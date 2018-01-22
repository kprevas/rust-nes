use apu::*;
use apu::bus::*;

pub struct Triangle {
    tick_buffer: Box<[f32]>,
    tick_buffer_ptr: usize,
    sample_buffer: Box<[f32]>,
    sample_buffer_ptr: usize,
    length_counter: u8,
    linear_counter: u8,
    timer_tick: u16,
    timer_phase: u8,
}

impl Triangle {
    pub fn new() -> Triangle {
        Triangle {
            tick_buffer: vec![0.0; TICKS_PER_SAMPLE as usize].into_boxed_slice(),
            tick_buffer_ptr: 0,
            sample_buffer: vec![0.0; SAMPLES_PER_FRAME as usize].into_boxed_slice(),
            sample_buffer_ptr: 0,
            length_counter: 0,
            linear_counter: 0,
            timer_tick: 0,
            timer_phase: 0,
        }
    }

    pub fn tick(&mut self, ctrl_bus: &mut TriangleCtrl) {
        if !ctrl_bus.enabled {
            self.length_counter = 0;
        } else if let Some(length_counter) = ctrl_bus.length_counter_load.take() {
            self.length_counter = LENGTH_TABLE[length_counter as usize];
        }

        let tick_val;
        if self.length_counter > 0 && self.linear_counter > 0 {
            if self.timer_tick >= ctrl_bus.timer {
                self.timer_tick = 0;
                self.timer_phase += 1;
                self.timer_phase %= 32;
            } else {
                self.timer_tick += 2;
            }
            tick_val = if self.timer_phase < 16 { 15.0 - f32::from(self.timer_phase) } else { f32::from(self.timer_phase) - 16.0 }
        } else {
            tick_val = 0.0;
        }
        self.tick_buffer[self.tick_buffer_ptr] = tick_val;

        self.tick_buffer_ptr += 1;
        if self.tick_buffer_ptr == TICKS_PER_SAMPLE as usize {
            let avg = self.tick_buffer.iter().fold(0.0f32, |a, &b| { a + b })
                / f32::from(TICKS_PER_SAMPLE);
            self.sample_buffer[self.sample_buffer_ptr] = avg;
            self.sample_buffer_ptr += 1;
            self.tick_buffer_ptr = 0;
        }
    }

    pub fn clock_length(&mut self, ctrl_bus: &mut TriangleCtrl) {
        if !ctrl_bus.control_flag && self.length_counter > 0 {
            self.length_counter -= 1;
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

    pub fn on_frame(&mut self) {
        self.sample_buffer_ptr = 0;
    }

    pub fn sample_buffer(&self) -> &[f32] {
        &self.sample_buffer
    }
}