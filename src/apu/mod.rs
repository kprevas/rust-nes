extern crate sample;
extern crate portaudio;
extern crate rb;
extern crate time;

pub mod bus;
mod pulse;
mod triangle;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use self::rb::{SpscRb, RB, Producer, RbProducer, RbConsumer, RbInspector};
use self::portaudio::*;
use self::pulse::*;
use self::triangle::*;

use self::bus::*;

const CHANNELS: i32 = 1;
const FRAMES: u32 = 512;
const TARGET_HZ: f64 = 44_100.0;
const TICKS_PER_SAMPLE: usize = 20;

const LENGTH_TABLE: [u8; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06,
    0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16,
    0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub type OutputStream = Stream<NonBlocking, Output<f32>>;

pub struct Apu<'a> {
    pulse_1: Pulse,
    pulse_2: Pulse,
    triangle: Triangle,
    frame_counter: u16,
    output_buffer: Producer<f32>,
    stream: OutputStream,
    buffer: Box<VecDeque<f32>>,
    bus: &'a RefCell<ApuBus>,
}

impl<'a> Apu<'a> {
    pub fn new(bus: &RefCell<ApuBus>, underrun: Arc<AtomicBool>) -> Result<Apu, Error> {
        let pa = PortAudio::new()?;
        let settings = pa.default_output_stream_settings::<f32>(CHANNELS, TARGET_HZ, FRAMES)?;

        let buffer = SpscRb::new((FRAMES * 5) as usize);
        let (buffer_producer, buffer_consumer) = (buffer.producer(), buffer.consumer());
        let inspector = buffer;

        let callback = move |OutputStreamCallbackArgs { buffer, frames, .. }| {
            underrun.store(inspector.count() < frames, Ordering::Relaxed);
            buffer_consumer.read_blocking(buffer);
            Continue
        };
        let mut stream = pa.open_non_blocking_stream(settings, callback)?;
        stream.start()?;

        Ok(Apu {
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
            triangle: Triangle::new(),
            frame_counter: 0,
            output_buffer: buffer_producer,
            stream,
            buffer: Box::new(VecDeque::with_capacity(TICKS_PER_SAMPLE + 1)),
            bus,
        })
    }

    fn clock_envelope(&mut self, bus: &mut ApuBus) {
        self.pulse_1.clock_envelope(&bus.pulse_1);
        self.pulse_2.clock_envelope(&bus.pulse_2);
        self.triangle.clock_linear_counter(&mut bus.triangle);
    }

    fn clock_length_and_sweep(&mut self, bus: &mut ApuBus) {
        self.pulse_1.clock_length_and_sweep(&mut bus.pulse_1);
        self.pulse_2.clock_length_and_sweep(&mut bus.pulse_2);
        self.triangle.clock_length(&mut bus.triangle);
    }

    pub fn tick(&mut self) {
        self.frame_counter += 1;
        let mut bus = self.bus.borrow_mut();
        match self.frame_counter {
            3729 => {
                self.clock_envelope(&mut bus);
            },
            7457 => {
                self.clock_envelope(&mut bus);
                self.clock_length_and_sweep(&mut bus);
            },
            11186 => {
                self.clock_envelope(&mut bus);
            },
            14915 => {
                if !bus.frame_mode {
                    self.clock_envelope(&mut bus);
                    self.clock_length_and_sweep(&mut bus);
                    self.frame_counter = 0;
                    if !bus.irq_inhibit {
                        bus.irq_interrupt = true;
                    }
                }
            },
            18641 => {
                self.clock_envelope(&mut bus);
                self.clock_length_and_sweep(&mut bus);
                self.frame_counter = 0;
            },
            _ => (),
        }

        let pulse_1 = self.pulse_1.tick(&mut bus.pulse_1);
        let pulse_2 = self.pulse_2.tick(&mut bus.pulse_2);
        let triangle = self.triangle.tick(&mut bus.triangle);
        self.buffer.push_back((pulse_1 + pulse_2) * 0.00752 + triangle * 0.00851);
        if self.buffer.len() >= TICKS_PER_SAMPLE {
            let mut sum = 0.0;
            for _ in 0..TICKS_PER_SAMPLE {
                sum += self.buffer.pop_front().unwrap();
            }
            let avg = sum / (TICKS_PER_SAMPLE as f32);
            self.output_buffer.write_blocking(&[avg]);
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.stream.abort()?;
        Ok(())
    }
}