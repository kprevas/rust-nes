use apu::*;

const DUTY_CYCLES: [[bool; 8]; 4] = [
    [false, true, false, false, false, false, false, false],
    [false, true, true, false, false, false, false, false],
    [false, true, true, true, true, false, false, false],
    [true, false, false, true, true, true, true, true],
];

pub struct Pulse {
    curr_timer: u16,
    curr_cycle: usize,
    envelope_delay: u8,
    envelope_value: u8,
    length_written: bool,
    sweep_counter: u8,
}

impl Pulse {
    pub fn new() -> Pulse {
        Pulse {
            curr_timer: 0,
            curr_cycle: 0,
            envelope_delay: 0,
            envelope_value: 15,
            length_written: false,
            sweep_counter: 0,
        }
    }

    fn sweep_target_period(&self, ctrl_bus: &SquareCtrl) -> u16 {
        if ctrl_bus.sweep.enabled {
            let shift_amount = ctrl_bus.timer >> ctrl_bus.sweep.shift_count;
            if ctrl_bus.sweep.negate {
                ctrl_bus.timer - shift_amount - if ctrl_bus.sweep.ones_complement_adj { 1 } else { 0 }
            } else {
                ctrl_bus.timer + shift_amount
            }
        } else {
            ctrl_bus.timer
        }
    }

    pub fn tick(&mut self, ctrl_bus: &mut SquareCtrl) -> f32 {
        if !ctrl_bus.enabled {
            ctrl_bus.length_counter = 0;
        } else if let Some(length_counter) = ctrl_bus.length_counter_load.take() {
            ctrl_bus.length_counter = LENGTH_TABLE[length_counter as usize];
            self.length_written = true;
        }

        let tick_val;
        if ctrl_bus.length_counter > 0 && self.sweep_target_period(&ctrl_bus) <= 0x7FF && ctrl_bus.timer >= 8 {
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

    pub fn clock_length_and_sweep(&mut self, ctrl_bus: &mut SquareCtrl) {
        if !ctrl_bus.halt_flag_envelope_loop && ctrl_bus.length_counter > 0 {
            ctrl_bus.length_counter -= 1;
        }
        if self.sweep_counter == 0 || ctrl_bus.sweep.reload {
            if self.sweep_counter == 0 {
                if ctrl_bus.sweep.enabled {
                    let target_period = self.sweep_target_period(&ctrl_bus);
                    if target_period <= 0x7FF {
                        ctrl_bus.timer = target_period;
                    }
                }
            }
            self.sweep_counter = ctrl_bus.sweep.period;
            ctrl_bus.sweep.reload = false;
        } else {
            self.sweep_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self, ctrl_bus: &SquareCtrl) {
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