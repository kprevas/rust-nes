#[derive(Default, Copy, Clone)]
pub struct ControllerState {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl ControllerState {
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