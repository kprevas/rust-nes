extern crate array_init;

use piston_window::*;
use bytes::IntoBuf;
use std::mem::swap;

const SAVE_KEYS: [Key; 10] = [Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6, Key::F7, Key::F8, Key::F9, Key::F10];

pub struct SaveStates {
    states: [Vec<u8>; 10],
    left_shift_state: bool,
    right_shift_state: bool,
}

impl SaveStates {
    pub fn new() -> SaveStates {
        SaveStates {
            states: array_init::array_init(|_i| Vec::new()),
            left_shift_state: false,
            right_shift_state: false,
        }
    }

    pub fn event(&mut self, event: &Event, cpu: &mut super::cpu::Cpu) {
        if let Some(Button::Keyboard(key_pressed)) = event.press_args() {
            if key_pressed == Key::RShift {
                self.right_shift_state = true;
            } else if key_pressed == Key::LShift {
                self.left_shift_state = true;
            }
            for (i, key) in SAVE_KEYS.iter().enumerate() {
                if *key == key_pressed {
                    if self.left_shift_state || self.right_shift_state {
                        let mut vec = Vec::new();
                        swap(&mut self.states[i], &mut vec);
                        let mut cursor = vec.into_buf();
                        cpu.load_state(&mut cursor);
                        vec = cursor.into_inner();
                        swap(&mut self.states[i], &mut vec);
                    } else {
                        cpu.save_state(&mut self.states[i]);
                    }
                }
            }
        }

        if let Some(Button::Keyboard(key_released)) = event.release_args() {
            if key_released == Key::RShift {
                self.right_shift_state = false;
            } else if key_released == Key::LShift {
                self.left_shift_state = false;
            }
        }
    }
}