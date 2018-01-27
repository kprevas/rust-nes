use apu::*;
use apu::bus::*;

pub const TIMER_VALUES: [u16; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

pub struct Noise {
    curr_timer: u16,
    shift_register: u16,
    envelope_delay: u8,
    envelope_value: u8,
    length_written: bool,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            curr_timer: 0,
            shift_register: 1,
            envelope_delay: 0,
            envelope_value: 15,
            length_written: false,
        }
    }

    pub fn tick(&mut self, ctrl_bus: &mut NoiseCtrl) -> f32 {
        if !ctrl_bus.enabled {
            ctrl_bus.length_counter = 0;
        } else if let Some(length_counter) = ctrl_bus.length_counter_load.take() {
            ctrl_bus.length_counter = LENGTH_TABLE[length_counter as usize];
            self.length_written = true;
        }

        let tick_val;
        if ctrl_bus.length_counter > 0 {
            if self.curr_timer == 0 {
                self.curr_timer = ctrl_bus.timer;
                let feedback_bit = if ctrl_bus.loop_noise { (self.shift_register & 0x40) >> 6 } else { (self.shift_register & 0x2) >> 1 };
                let feedback = (self.shift_register & 0x1) ^ feedback_bit;
                self.shift_register >>= 1;
                let feedback_applied = (self.shift_register & !(0x4000)) | (feedback << 14);
                self.shift_register = feedback_applied;
            } else {
                self.curr_timer -= 1;
            }
            tick_val = self.shift_register & 0x1 == 0;
        } else {
            tick_val = false;
        }
        if tick_val {
            if ctrl_bus.constant_volume {
                f32::from(ctrl_bus.envelope_param)
            } else {
                f32::from(self.envelope_value)
            }
        } else {
            0.0
        }
    }

    pub fn clock_length(&mut self, ctrl_bus: &mut NoiseCtrl) {
        if !ctrl_bus.halt_flag_envelope_loop && ctrl_bus.length_counter > 0 {
            ctrl_bus.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self, ctrl_bus: &NoiseCtrl) {
        if self.length_written {
            self.length_written = false;
            self.envelope_delay = ctrl_bus.envelope_param;
            self.envelope_value = 15;
        }
        if self.envelope_value == 0 {
            self.envelope_delay = ctrl_bus.envelope_param;
            if ctrl_bus.halt_flag_envelope_loop {
                self.envelope_value = 15;
            }
        } else if self.envelope_delay > 0 {
            self.envelope_delay -= 1;
        } else {
            self.envelope_delay = ctrl_bus.envelope_param;
            self.envelope_value -= 1;
        }
    }
}