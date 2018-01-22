use apu::*;
use apu::bus::*;

const DUTY_CYCLES: [[bool; 8]; 4] = [
    [false, true, false, false, false, false, false, false],
    [false, true, true, false, false, false, false, false],
    [false, true, true, true, true, false, false, false],
    [true, false, false, true, true, true, true, true],
];

pub struct Pulse {
    curr_timer: u16,
    curr_cycle: usize,
    tick_buffer: Box<[f32]>,
    tick_buffer_ptr: usize,
    sample_buffer: Box<[f32]>,
    sample_buffer_ptr: usize,
    length_counter: u8,
    envelope_delay: u8,
    envelope_value: u8,
}

impl Pulse {
    pub fn new() -> Pulse {
        Pulse {
            curr_timer: 0,
            curr_cycle: 0,
            tick_buffer: vec![0.0; TICKS_PER_SAMPLE as usize].into_boxed_slice(),
            tick_buffer_ptr: 0,
            sample_buffer: vec![0.0; SAMPLES_PER_FRAME as usize].into_boxed_slice(),
            sample_buffer_ptr: 0,
            length_counter: 0,
            envelope_delay: 0,
            envelope_value: 15,
        }
    }

    pub fn tick(&mut self, ctrl_bus: &mut ChannelCtrl) {
        if !ctrl_bus.enabled {
            self.length_counter = 0;
        } else if let Some(length_counter) = ctrl_bus.length_counter_load.take() {
            self.length_counter = LENGTH_TABLE[length_counter as usize];
            self.envelope_delay = ctrl_bus.envelope_param;
            self.envelope_value = 15;
        }

        let tick_val;
        if self.length_counter > 0 {
            if self.curr_timer == 0 {
                self.curr_timer = ctrl_bus.timer;
                if ctrl_bus.timer >= 8 {
                    self.curr_cycle += 1;
                    self.curr_cycle %= 8;
                }
            } else {
                self.curr_timer -= 1;
            }
            tick_val = DUTY_CYCLES[ctrl_bus.duty_cycle][self.curr_cycle];
        } else {
            tick_val = false;
        }
        self.tick_buffer[self.tick_buffer_ptr] = if tick_val {
            if ctrl_bus.constant_volume {
                f32::from(ctrl_bus.envelope_param)
            } else {
                f32::from(self.envelope_value)
            }
        } else {
            0.0
        };

        self.tick_buffer_ptr += 1;
        if self.tick_buffer_ptr == TICKS_PER_SAMPLE as usize {
            let avg = self.tick_buffer.iter().fold(0.0f32, |a, &b| { a + b })
                / f32::from(TICKS_PER_SAMPLE);
            self.sample_buffer[self.sample_buffer_ptr] = avg;
            self.sample_buffer_ptr += 1;
            self.tick_buffer_ptr = 0;
        }
    }

    pub fn clock_length(&mut self, ctrl_bus: &ChannelCtrl) {
        if !ctrl_bus.halt_flag_envelope_loop && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self, ctrl_bus: &ChannelCtrl) {
        if self.envelope_value == 0 {
            if ctrl_bus.halt_flag_envelope_loop {
                self.envelope_value = ctrl_bus.envelope_param;
            }
        } else if self.envelope_delay > 0 {
            self.envelope_delay -= 1;
        } else {
            self.envelope_value -= 1;
        }
    }

    pub fn on_frame(&mut self) {
        self.sample_buffer_ptr = 0;
    }

    pub fn sample_buffer(&self) -> &[f32] {
        &self.sample_buffer
    }
}