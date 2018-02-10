use apu::*;

pub struct Triangle {
    linear_counter: u8,
    timer_tick: u16,
    timer_phase: u8,
}

impl Triangle {
    pub fn new() -> Triangle {
        Triangle {
            linear_counter: 0,
            timer_tick: 0,
            timer_phase: 0,
        }
    }

    pub fn tick(&mut self, ctrl_bus: &mut TriangleCtrl) -> f32 {
        if !ctrl_bus.enabled {
            ctrl_bus.length_counter = 0;
        } else if let Some(length_counter) = ctrl_bus.length_counter_load.take() {
            ctrl_bus.length_counter = LENGTH_TABLE[length_counter as usize];
        }

        if ctrl_bus.length_counter > 0 && self.linear_counter > 0 {
            if self.timer_tick >= ctrl_bus.timer + 1 {
                self.timer_tick -= ctrl_bus.timer + 1;
                self.timer_phase += 1;
                self.timer_phase %= 32;
            }
            self.timer_tick += 2;
            if self.timer_phase < 16 {
                15.0 - f32::from(self.timer_phase)
            } else {
                f32::from(self.timer_phase) - 16.0
            }
        } else {
            0.0
        }
    }

    pub fn clock_length(&mut self, ctrl_bus: &mut TriangleCtrl) {
        if !ctrl_bus.control_flag && ctrl_bus.length_counter > 0 {
            ctrl_bus.length_counter -= 1;
        }
    }

    pub fn clock_linear_counter(&mut self, ctrl_bus: &mut TriangleCtrl) {
        if ctrl_bus.linear_counter_reload {
            self.linear_counter = ctrl_bus.reload_value;
        } else {
            if self.linear_counter > 0 {
                self.linear_counter -= 1;
            }
        }
        if !ctrl_bus.control_flag {
            ctrl_bus.linear_counter_reload = false;
        }
    }
}