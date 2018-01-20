pub struct Ctrl {
    pub nametable_select: Option<u8>,
    pub address_increment_vertical: bool,
    pub sprite_pattern_table_high: bool,
    pub bgd_pattern_table_high: bool,
    pub sprite_size_large: bool,
    pub ppu_output_to_ext: bool,
    pub gen_nmi: bool,
}

impl Ctrl {
    fn from_u8(value: u8) -> Ctrl {
        Ctrl {
            nametable_select: Some(value & 0b11),
            address_increment_vertical: value & 0b100 > 0,
            sprite_pattern_table_high: value & 0b1000 > 0,
            bgd_pattern_table_high: value & 0b10000 > 0,
            sprite_size_large: value & 0b100000 > 0,
            ppu_output_to_ext: value & 0b1000000 > 0,
            gen_nmi: value & 0b10000000 > 0,
        }
    }
}

pub struct Mask {
    pub grayscale: bool,
    pub show_bgd_left8: bool,
    pub show_sprite_left8: bool,
    pub show_bgd: bool,
    pub show_sprite: bool,
    pub red_emphasis: bool,
    pub green_emphasis: bool,
    pub blue_emphasis: bool,
}

impl Mask {
    fn from_u8(value: u8) -> Mask {
        Mask {
            grayscale: value & 0b1 > 0,
            show_bgd_left8: value & 0b10 > 0,
            show_sprite_left8: value & 0b100 > 0,
            show_bgd: value & 0b1000 > 0,
            show_sprite: value & 0b10000 > 0,
            red_emphasis: value & 0b100000 > 0,
            green_emphasis: value & 0b1000000 > 0,
            blue_emphasis: value & 0b10000000 > 0,
        }
    }
}

pub struct Status {
    pub sprite_overflow: bool,
    pub sprite_0_hit: bool,
    pub vertical_blank: bool,
}

impl Status {
    fn to_u8(&self) -> u8 {
        0 + if self.sprite_overflow { 0b100000 } else { 0 }
            + if self.sprite_0_hit { 0b1000000 } else { 0 }
            + if self.vertical_blank { 0b10000000 } else { 0 }
    }
}

pub struct PpuBus {
    pub ctrl: Ctrl,
    pub mask: Mask,
    pub status: Status,
    pub oam_addr: u8,
    pub oam_data: Option<u8>,
    pub scroll: Option<u8>,
    pub addr: Option<u8>,
    pub data: Option<u8>,
    pub first_write: bool,
    pub nmi_interrupt: bool,
}

impl PpuBus {
    pub fn new() -> PpuBus {
        PpuBus {
            ctrl: Ctrl {
                nametable_select: None,
                address_increment_vertical: false,
                sprite_pattern_table_high: false,
                bgd_pattern_table_high: false,
                sprite_size_large: false,
                ppu_output_to_ext: false,
                gen_nmi: false,
            },
            mask: Mask {
                grayscale: false,
                show_bgd_left8: false,
                show_sprite_left8: false,
                show_bgd: false,
                show_sprite: false,
                red_emphasis: false,
                green_emphasis: false,
                blue_emphasis: false,
            },
            status: Status {
                sprite_overflow: false,
                sprite_0_hit: false,
                vertical_blank: false,
            },
            oam_addr: 0,
            oam_data: None,
            scroll: None,
            addr: None,
            data: None,
            first_write: false,
            nmi_interrupt: false,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr % 8 {
            0 => {
                debug!("tried to read from PPU control register");
                0
            }
            1 => {
                debug!("tried to read from PPU mask register");
                0
            }
            2 => {
                let value = self.status.to_u8();
                self.status.vertical_blank = false;
                self.first_write = false;
                value
            }
            3 => self.oam_addr,
            4 => match self.oam_data {
                Some(value) => value,
                None => {
                    debug!("tried to read OAM data when none present");
                    0
                }
            },
            5 => {
                debug!("tried to read from PPU scroll register");
                0
            }
            6 => {
                debug!("tried to read from PPU address register");
                0
            }
            7 => match self.data {
                Some(value) => value,
                None => {
                    debug!("tried to read PPU data when none present");
                    0
                }
            },
            _ => panic!()
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr % 8 {
            0 => self.ctrl = Ctrl::from_u8(value),
            1 => self.mask = Mask::from_u8(value),
            2 => debug!("tried to write to PPU status register"),
            3 => self.oam_addr = value,
            4 => {
                self.oam_data = Some(value);
            }
            5 => {
                self.scroll = Some(value);
                self.first_write = !self.first_write;
            }
            6 => {
                self.addr = Some(value);
                self.first_write = !self.first_write;
            }
            7 => {
                self.data = Some(value);
            }
            _ => panic!()
        }
    }
}
