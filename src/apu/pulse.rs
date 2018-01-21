use apu::*;

const DUTY_CYCLES: [[bool; 8]; 4] = [
    [false, true, false, false, false, false, false, false],
    [false, true, true, false, false, false, false, false],
    [false, true, true, true, true, false, false, false],
    [true, false, false, true, true, true, true, true],
];

pub struct Pulse {
    duty_cycle: usize,
    length_counter: u8,
    envelope: bool,
    envelope_divider: u8,
    timer: u16,
    curr_timer: u16,
    curr_cycle: usize,
    tick_buffer: Box<[bool]>,
    tick_buffer_ptr: usize,
    sample_buffer: Box<[f32]>,
    sample_buffer_ptr: usize,
}

impl Pulse {
    pub fn new() -> Pulse {
        Pulse {
            duty_cycle: 0,
            length_counter: 0,
            envelope: false,
            envelope_divider: 0,
            timer: 0,
            curr_timer: 0,
            curr_cycle: 0,
            tick_buffer: vec![false; TICKS_PER_SAMPLE as usize].into_boxed_slice(),
            tick_buffer_ptr: 0,
            sample_buffer: vec![0.0; SAMPLES_PER_FRAME as usize].into_boxed_slice(),
            sample_buffer_ptr: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.curr_timer == 0 {
            self.curr_timer = self.timer;
            if self.timer > 0 {
                self.curr_cycle += 1;
                self.curr_cycle %= 8;
                let tick_val = DUTY_CYCLES[self.duty_cycle][self.curr_cycle];
                self.tick_buffer[self.tick_buffer_ptr] = tick_val;
            }
        } else {
            self.curr_timer -= 1;
        }

        self.tick_buffer_ptr += 1;
        if self.tick_buffer_ptr == TICKS_PER_SAMPLE as usize {
            let avg = f32::from(self.tick_buffer.iter().fold(0u16, |a, &b| { a + if b { 1 } else { 0 } }))
                / f32::from(TICKS_PER_SAMPLE);
            self.sample_buffer[self.sample_buffer_ptr] = avg;
            self.sample_buffer_ptr += 1;
            self.tick_buffer_ptr = 0;
        }
    }

    pub fn on_frame(&mut self) {
        self.sample_buffer_ptr = 0;
    }

    pub fn sample_buffer(&self) -> &[f32] {
        &self.sample_buffer
    }
}