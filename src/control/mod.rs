extern crate array_init;

use std::mem::swap;

use piston_window::*;

use window::Cpu;

use super::record::Recorder;

const SAVE_KEYS: [Key; 10] = [
    Key::F1,
    Key::F2,
    Key::F3,
    Key::F4,
    Key::F5,
    Key::F6,
    Key::F7,
    Key::F8,
    Key::F9,
    Key::F10,
];

pub struct Control<const B: usize> {
    states: [Vec<u8>; 10],
    left_shift_state: bool,
    right_shift_state: bool,
    left_ctrl_state: bool,
    right_ctrl_state: bool,
    pub reset: bool,
    pub pause: bool,
    pub step: bool,
    pub input_overlay: bool,
    pub render_layers: usize,
}

impl<const B: usize> Control<B> {
    pub fn new() -> Control<B> {
        Control {
            states: array_init::array_init(|_i| Vec::new()),
            left_shift_state: false,
            right_shift_state: false,
            left_ctrl_state: false,
            right_ctrl_state: false,
            reset: false,
            pause: false,
            step: false,
            input_overlay: false,
            render_layers: 0,
        }
    }

    pub fn event(
        &mut self,
        event: &Event,
        cpu: &mut dyn Cpu,
        recorder: &mut Recorder<B>,
        frame_count: u32,
    ) {
        if let Some(Button::Keyboard(key_pressed)) = event.press_args() {
            self.process_modifier_keys(key_pressed, true);
            for (i, key) in SAVE_KEYS.iter().enumerate() {
                if *key == key_pressed {
                    if self.left_shift_state || self.right_shift_state {
                        let mut vec = Vec::new();
                        swap(&mut self.states[i], &mut vec);
                        cpu.load_state(&mut vec.as_slice());
                        swap(&mut self.states[i], &mut vec);
                    } else {
                        cpu.save_state(&mut self.states[i]);
                    }
                }
            }
            if key_pressed == Key::R && (self.left_ctrl_state || self.right_ctrl_state) {
                self.reset = true;
            }
            if key_pressed == Key::S && (self.left_ctrl_state || self.right_ctrl_state) {
                recorder.toggle(frame_count);
            }
            if key_pressed == Key::P
                && (self.left_ctrl_state || self.right_ctrl_state)
                && (self.left_shift_state || self.right_shift_state)
            {
                self.pause = !self.pause;
            } else if key_pressed == Key::P && (self.left_ctrl_state || self.right_ctrl_state) {
                recorder.toggle_playback(frame_count);
            }
            if key_pressed == Key::I && (self.left_ctrl_state || self.right_ctrl_state) {
                self.input_overlay = !self.input_overlay;
            }
            if key_pressed == Key::Equals {
                cpu.increase_speed();
            }
            if key_pressed == Key::Minus {
                cpu.decrease_speed();
            }
            if key_pressed == Key::LeftBracket {
                self.render_layers = self.render_layers.wrapping_sub(1);
            }
            if key_pressed == Key::RightBracket {
                self.render_layers = self.render_layers.wrapping_add(1);
            }
            if self.pause && key_pressed == Key::Space {
                self.step = true;
            }
        }

        if let Some(Button::Keyboard(key_released)) = event.release_args() {
            self.process_modifier_keys(key_released, false);
        }
    }

    fn process_modifier_keys(&mut self, key_pressed: Key, state: bool) {
        match key_pressed {
            Key::RShift => self.right_shift_state = state,
            Key::LShift => self.left_shift_state = state,
            Key::RCtrl => self.right_ctrl_state = state,
            Key::LCtrl => self.left_ctrl_state = state,
            _ => (),
        }
    }
}
