use piston_window::*;

#[derive(Default, Copy, Clone)]
pub struct ControllerState {
    a: bool,
    b: bool,
    select: bool,
    start: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl ControllerState {
    pub fn event(&mut self, event: &Event, reset: &mut bool) {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::A => self.b = true,
                Key::S => self.a = true,
                Key::Tab => self.select = true,
                Key::Return => self.start = true,
                Key::Up => self.up = true,
                Key::Down => self.down = true,
                Key::Left => self.left = true,
                Key::Right => self.right = true,

                Key::R => *reset = true,
                _ => (),
            }
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            match key {
                Key::A => self.b = false,
                Key::S => self.a = false,
                Key::Tab => self.select = false,
                Key::Return => self.start = false,
                Key::Up => self.up = false,
                Key::Down => self.down = false,
                Key::Left => self.left = false,
                Key::Right => self.right = false,
                _ => (),
            }
        }
    }

    pub fn to_u8(&self) -> u8 {
        (if self.right { 1 << 7 } else { 0 })
            + (if self.left { 1 << 6 } else { 0 })
            + (if self.down { 1 << 5 } else { 0 })
            + (if self.up { 1 << 4 } else { 0 })
            + (if self.start { 1 << 3 } else { 0 })
            + (if self.select { 1 << 2 } else { 0 })
            + (if self.b { 1 << 1 } else { 0 })
            + (if self.a { 1 << 0 } else { 0 })
    }
}