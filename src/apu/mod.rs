extern crate dasp;
extern crate portaudio;
extern crate rb;
extern crate time;

use std::cell::RefCell;

use bincode::{deserialize_from, serialize};
use bytes::*;

use cartridge::CartridgeBus;

use self::bus::*;
use self::dmc::*;
use self::noise::*;
use self::portaudio::*;
use self::pulse::*;
use self::rb::{Producer, RB, RbConsumer, RbInspector, RbProducer, SpscRb};
use self::triangle::*;

pub mod bus;
mod dmc;
mod noise;
mod pulse;
mod triangle;

const CHANNELS: i32 = 1;
const TARGET_HZ: f64 = 44_100.0;
const TICKS_PER_SAMPLE: f64 = 20.2922108844;
const APPROX_TICKS_PER_FRAME: usize = 14915;
const MAX_BUFFER_FRAMES: usize = 3;

const LENGTH_TABLE: [u8; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub type OutputStream = Stream<NonBlocking, Output<f32>>;

pub struct Apu<'a> {
    pulse_1: Pulse,
    pulse_2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    frame_counter: i32,
    apu_tick: bool,
    output_buffer: Producer<f32>,
    stream: Option<OutputStream>,
    bus: &'a RefCell<ApuBus>,
}

impl<'a> Apu<'a> {
    pub fn new(bus: &RefCell<ApuBus>, pa: Option<PortAudio>) -> Result<Apu, Error> {
        let buffer = SpscRb::new(500_000);
        let (buffer_producer, buffer_consumer) = (buffer.producer(), buffer.consumer());

        let mut resample_data = Box::new(vec![0.0; 20_000]);
        let inspector = buffer;

        let callback = move |OutputStreamCallbackArgs { buffer, frames, .. }| {
            let ticks = TICKS_PER_SAMPLE * frames as f64;
            let ticks_to_read;
            if inspector.count() > APPROX_TICKS_PER_FRAME {
                ticks_to_read = inspector.count().min(ticks.floor() as usize);
            } else {
                ticks_to_read = inspector.count().min(ticks.ceil() as usize);
            }
            while inspector.count() > APPROX_TICKS_PER_FRAME * MAX_BUFFER_FRAMES {
                buffer_consumer.skip(APPROX_TICKS_PER_FRAME).unwrap();
            }
            buffer_consumer.read_blocking(&mut resample_data[0..ticks_to_read]);
            let ticks_per_sample = ((ticks_to_read as f32) / (frames as f32)).floor() as i16;
            let mut buffer_ptr = 0;
            let mut sum = 0.0;
            let mut ticks = 0;
            for tick_val in resample_data.iter().take(ticks_to_read) {
                sum += tick_val;
                ticks += 1;
                if ticks >= ticks_per_sample {
                    buffer[buffer_ptr] = sum / (ticks as f32);
                    buffer_ptr += 1;
                    if buffer_ptr >= frames {
                        break;
                    }
                    sum = 0.0;
                    ticks = 0;
                }
            }
            while buffer_ptr < frames {
                buffer[buffer_ptr] = 0.0;
                buffer_ptr += 1;
            }
            Continue
        };
        let stream = pa.map(|pa| {
            let settings = pa
                .default_output_stream_settings::<f32>(
                    CHANNELS,
                    TARGET_HZ,
                    FRAMES_PER_BUFFER_UNSPECIFIED,
                )
                .unwrap();
            let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();
            stream.start().unwrap();
            stream
        });

        Ok(Apu {
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            frame_counter: 0,
            apu_tick: false,
            output_buffer: buffer_producer,
            stream,
            bus,
        })
    }

    fn clock_envelope(&mut self, bus: &mut ApuBus) {
        self.pulse_1.clock_envelope(&bus.pulse_1);
        self.pulse_2.clock_envelope(&bus.pulse_2);
        self.triangle.clock_linear_counter(&mut bus.triangle);
        self.noise.clock_envelope(&bus.noise);
    }

    fn clock_length_and_sweep(&mut self, bus: &mut ApuBus) {
        self.pulse_1.clock_length_and_sweep(&mut bus.pulse_1);
        self.pulse_2.clock_length_and_sweep(&mut bus.pulse_2);
        self.triangle.clock_length(&mut bus.triangle);
        self.noise.clock_length(&mut bus.noise);
    }

    pub fn tick(&mut self, cartridge: &Box<dyn CartridgeBus>) {
        let mut bus = self.bus.borrow_mut();
        self.frame_counter += 1;
        if bus.frame_mode_written {
            if bus.frame_mode {
                self.clock_envelope(&mut bus);
                self.clock_length_and_sweep(&mut bus);
            }
            self.frame_counter = if self.apu_tick { -2 } else { -3 };
            bus.frame_mode_written = false;
        }
        match self.frame_counter {
            7457 => {
                self.clock_envelope(&mut bus);
            }
            14913 => {
                self.clock_envelope(&mut bus);
                self.clock_length_and_sweep(&mut bus);
            }
            22371 => {
                self.clock_envelope(&mut bus);
            }
            29828 => {
                if !bus.frame_mode {
                    if !bus.frame_irq_inhibit {
                        bus.frame_interrupt = true;
                    }
                }
            }
            29829 => {
                if !bus.frame_mode {
                    if !bus.frame_irq_inhibit {
                        bus.frame_interrupt = true;
                    }
                    self.clock_envelope(&mut bus);
                    self.clock_length_and_sweep(&mut bus);
                }
            }
            29830 => {
                if !bus.frame_mode {
                    if !bus.frame_irq_inhibit {
                        bus.frame_interrupt = true;
                    }
                    self.frame_counter = 0;
                }
            }
            37281 => {
                self.clock_envelope(&mut bus);
                self.clock_length_and_sweep(&mut bus);
                self.frame_counter = -1;
            }
            _ => (),
        }

        if self.apu_tick {
            let pulse_1 = self.pulse_1.tick(&mut bus.pulse_1);
            let pulse_2 = self.pulse_2.tick(&mut bus.pulse_2);
            let triangle = self.triangle.tick(&mut bus.triangle);
            let noise = self.noise.tick(&mut bus.noise);
            let dmc = self.dmc.tick(&mut bus, cartridge);
            if self.stream.is_some() {
                self.output_buffer
                    .write_blocking(&[(pulse_1 + pulse_2) * 0.00752
                        + triangle * 0.00851
                        + noise * 0.00494
                        + dmc * 0.00335]);
            }
        }

        let apu_tick = !self.apu_tick;
        self.apu_tick = apu_tick;
    }

    pub fn close(&mut self) {
        if let Some(ref mut stream) = self.stream {
            stream.abort().unwrap();
        }
    }

    pub fn save_state(&self, out: &mut Vec<u8>) {
        out.put_slice(&serialize(&self.pulse_1).unwrap());
        out.put_slice(&serialize(&self.pulse_2).unwrap());
        out.put_slice(&serialize(&self.triangle).unwrap());
        out.put_slice(&serialize(&self.noise).unwrap());
        out.put_slice(&serialize(&self.dmc).unwrap());
        out.put_i32(self.frame_counter);
        out.put_u8(if self.apu_tick { 1 } else { 0 });
    }

    pub fn load_state(&mut self, state: &mut dyn Buf) {
        self.pulse_1 = deserialize_from(state.reader()).unwrap();
        self.pulse_2 = deserialize_from(state.reader()).unwrap();
        self.triangle = deserialize_from(state.reader()).unwrap();
        self.noise = deserialize_from(state.reader()).unwrap();
        self.dmc = deserialize_from(state.reader()).unwrap();
        self.frame_counter = state.get_i32();
        self.apu_tick = state.get_u8() == 1;
    }

    pub fn instrumentation_short(&self) -> String {
        format!("{}", self.frame_counter)
    }
}
