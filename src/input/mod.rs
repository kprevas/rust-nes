use piston_window::*;

pub struct ControllerState {
    keys: [Key;8],
    state: u8,
}

impl ControllerState {

    fn new(keys: [Key;8]) -> ControllerState {
        ControllerState {
            keys,
            state: 0,
        }
    }

    pub fn player_1() -> ControllerState {
        ControllerState::new([Key::J, Key::H, Key::Backslash, Key::Return, Key::Up, Key::Down, Key::Left, Key::Right])
    }

    pub fn player_2() -> ControllerState {
        ControllerState::new([Key::G, Key::F, Key::CapsLock, Key::Tab, Key::W, Key::S, Key::A, Key::D])
    }

    pub fn event(&mut self, event: &Event) -> bool {
        let prev_state = self.state;
        if let Some(Button::Keyboard(key_pressed)) = event.press_args() {
            for (i, key) in self.keys.iter().enumerate() {
                if *key == key_pressed {
                    self.state |= 1 << i;
                }
            }
        }

        if let Some(Button::Keyboard(key_released)) = event.release_args() {
            for (i, key) in self.keys.iter().enumerate() {
                if *key == key_released {
                    self.state &= !(1 << i);
                }
            }
        }
        self.state != prev_state
    }

    pub fn to_u8(&self) -> u8 {
        self.state
    }

    pub fn set_from_u8(&mut self, value: u8) {
        self.state = value;
    }
}