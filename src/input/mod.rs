use piston_window::*;
use piston_window::Button::*;

pub struct ControllerState {
    buttons: [Button; 8],
    state: u8,
}

impl ControllerState {
    fn new(buttons: [Button; 8]) -> ControllerState {
        ControllerState {
            buttons,
            state: 0,
        }
    }

    pub fn player_1() -> ControllerState {
        ControllerState::new([
            Keyboard(Key::J),
            Keyboard(Key::H),
            Keyboard(Key::Backslash),
            Keyboard(Key::Return),
            Keyboard(Key::Up),
            Keyboard(Key::Down),
            Keyboard(Key::Left),
            Keyboard(Key::Right),
        ])
    }

    pub fn player_2() -> ControllerState {
        ControllerState::new([
            Keyboard(Key::G),
            Keyboard(Key::F),
            Keyboard(Key::CapsLock),
            Keyboard(Key::Tab),
            Keyboard(Key::W),
            Keyboard(Key::S),
            Keyboard(Key::A),
            Keyboard(Key::D),
        ])
    }

    pub fn event(&mut self, event: &Event) -> bool {
        let prev_state = self.state;
        if let Some(button_pressed) = event.press_args() {
            for (i, button) in self.buttons.iter().enumerate() {
                if *button == button_pressed {
                    self.state |= 1 << i;
                }
            }
        }

        if let Some(button_released) = event.release_args() {
            for (i, button) in self.buttons.iter().enumerate() {
                if *button == button_released {
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

    pub fn buttons(&self) -> [Button; 8] {
        self.buttons.clone()
    }

    pub fn set_buttons(&mut self, buttons: &[Button]) {
        for i in 0..8 {
            self.buttons[i] = buttons[i];
        }
    }
}