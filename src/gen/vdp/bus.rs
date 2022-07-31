struct Status {
    fifo_empty: bool,
    fifo_full: bool,
    vertical_interrupt: bool,
    sprite_limit: bool,
    sprite_overlap: bool,
    interlaced_odd_frame: bool,
    vblank: bool,
    hblank: bool,
    dma: bool,
    pal: bool,
}

impl Status {
    fn to_u16(&self) -> u16 {
        0b0011010000000000
            | if self.fifo_empty { 1 << 9 } else { 0 }
            | if self.fifo_full { 1 << 8 } else { 0 }
            | if self.vertical_interrupt { 1 << 7 } else { 0 }
            | if self.sprite_limit { 1 << 6 } else { 0 }
            | if self.sprite_overlap { 1 << 5 } else { 0 }
            | if self.interlaced_odd_frame { 1 << 4 } else { 0 }
            | if self.vblank { 1 << 3 } else { 0 }
            | if self.hblank { 1 << 2 } else { 0 }
            | if self.dma { 1 << 1 } else { 0 }
            | if self.pal { 1 << 1 } else { 0 }
    }
}

pub struct VdpBus {
    status: Status,
}

impl VdpBus {
    pub fn new() -> VdpBus {
        VdpBus {
            status: Status {
                fifo_empty: false,
                fifo_full: false,
                vertical_interrupt: false,
                sprite_limit: false,
                sprite_overlap: false,
                interlaced_odd_frame: false,
                vblank: false,
                hblank: false,
                dma: false,
                pal: false,
            },
        }
    }

    pub fn read_byte(&mut self, addr: u32) -> u8 {
        match addr {
            0xC004 | 0xC006 => (self.status.to_u16() >> 8) as u8,
            0xC005 | 0xC007 => (self.status.to_u16() & 0xFF) as u8,
            _ => panic!(),
        }
    }

    pub fn read_word(&mut self, addr: u32) -> u16 {
        match addr {
            0xC004 | 0xC006 => self.status.to_u16(),
            _ => panic!(),
        }
    }

    pub fn read_long(&mut self, addr: u32) -> u32 {
        match addr {
            0xC004 => ((self.status.to_u16() as u32) << 16) | (self.status.to_u16() as u32),
            _ => panic!(),
        }
    }

    pub fn write_byte(&mut self, addr: u32, _data: u8) {
        match addr {
            _ => panic!(),
        }
    }

    pub fn write_word(&mut self, addr: u32, _data: u16) {
        match addr {
            _ => panic!(),
        }
    }

    pub fn write_long(&mut self, addr: u32, _data: u32) {
        match addr {
            _ => panic!(),
        }
    }
}
