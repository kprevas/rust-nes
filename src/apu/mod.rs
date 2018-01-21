extern crate sample;
extern crate portaudio;
extern crate rb;

mod pulse;

use self::rb::{SpscRb, RB, Producer, RbProducer, RbConsumer};

use self::portaudio::*;
use self::pulse::*;

const CHANNELS: i32 = 1;
const FRAMES: u32 = 64;
const SAMPLE_HZ: f64 = 44_100.0;
const TICKS_PER_SAMPLE: u8 = (4_194_304 / 44_100) as u8;
const SAMPLES_PER_FRAME: u16 = 44_100 / 60;

pub type OutputStream = Stream<NonBlocking, Output<f32>>;

pub struct Apu {
    pulse_1: Pulse,
    pulse_2: Pulse,
    buffer: Producer<f32>,
    stream: OutputStream,
    frame_buffer: Vec<f32>,
}

impl Apu {
    pub fn new() -> Result<Apu, Error> {
        let pa = PortAudio::new()?;
        let settings = pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_HZ, FRAMES)?;

        let buffer = SpscRb::new((SAMPLES_PER_FRAME * 2) as usize);
        let (buffer_producer, buffer_consumer) = (buffer.producer(), buffer.consumer());

        let callback = move |OutputStreamCallbackArgs { buffer, .. }| {
            buffer_consumer.read_blocking(buffer);
            Continue
        };
        let mut stream = pa.open_non_blocking_stream(settings, callback)?;
        stream.start()?;

        Ok(Apu {
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
            buffer: buffer_producer,
            stream,
            frame_buffer: vec!(0.0; SAMPLES_PER_FRAME as usize),
        })
    }

    pub fn tick(&mut self) {
        self.pulse_1.tick();
    }

    pub fn do_frame(&mut self) {
        {
            let pulse_1 = self.pulse_1.sample_buffer();
            let pulse_2 = self.pulse_2.sample_buffer();
            for i in 0..SAMPLES_PER_FRAME as usize {
                let val = (pulse_1[i] + pulse_2[i]) * 0.00752;
                self.frame_buffer[i] = val;
            }
        }
        self.buffer.write_blocking(self.frame_buffer.as_slice());

        self.pulse_1.on_frame();
    }
}