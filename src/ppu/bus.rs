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
    pub just_read: bool,
}

impl Status {
    fn to_u8(&self) -> u8 {
        0 + if self.sprite_overflow { 0b100000 } else { 0 }
            + if self.sprite_0_hit { 0b1000000 } else { 0 }
            + if self.vertical_blank { 0b10000000 } else { 0 }
    }
}

const DECAY_REGISTER_TTL: u32 = 1073863;

struct DecayRegister {
    pub masks: [u8; 8],
    pub value: u8,
    pub ttl: [u32; 8],
}

impl DecayRegister {
    fn new(masks: [u8; 8]) -> DecayRegister {
        DecayRegister {
            masks,
            value: 0,
            ttl: [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    fn tick(&mut self) {
        for i in 0..8 {
            if self.ttl[i] > 0 {
                self.ttl[i] -= 1;
                if self.ttl[i] == 0 {
                    self.value &= !(1 << i);
                }
            }
        }
    }

    fn write(&mut self, value: u8) {
        self.refresh_bits(value, 0xFF);
    }

    fn read(&mut self, addr: usize, value: u8, palette: bool) -> u8 {
        let mask = if palette { 0xC0 } else { self.masks[addr] };
        let inv_mask = !mask;
        self.refresh_bits(value, inv_mask);
        (self.value & mask) | (value & (inv_mask))
    }

    fn refresh_bits(&mut self, value: u8, mask: u8) {
        self.value = (self.value & !mask) | (value & mask);
        let mut mask = mask;
        for i in 0..8 {
            if mask & 1 > 0 {
                self.ttl[i] = DECAY_REGISTER_TTL;
            }
            mask >>= 1;
        }
    }
}

pub struct PpuBus {
    pub ctrl: Ctrl,
    pub mask: Mask,
    pub status: Status,
    pub oam_addr: u8,
    pub oam_data: Option<u8>,
    pub scroll: Option<u8>,
    pub addr_write: Option<u8>,
    pub data_write: Option<u8>,
    pub addr: u16,
    pub read_buffer: Option<u8>,
    pub palette_data: u8,
    pub first_write: bool,
    pub nmi_interrupt: bool,
    pub nmi_interrupt_age: u8,
    decay_register: DecayRegister,
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
                just_read: false,
            },
            oam_addr: 0,
            oam_data: None,
            scroll: None,
            addr_write: None,
            data_write: None,
            addr: 0,
            read_buffer: Some(0),
            palette_data: 0,
            first_write: false,
            nmi_interrupt: false,
            nmi_interrupt_age: 0,
            decay_register: DecayRegister::new(
                [0xFF, 0xFF, 0x1F, 0xFF, 0x00, 0xFF, 0xFF, 0x00]),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let addr = (addr % 8) as usize;
        self.decay_register.read(addr,
                                 match addr {
                                     0 => 0,
                                     1 => 0,
                                     2 => {
                                         let value = self.status.to_u8();
                                         self.status.vertical_blank = false;
                                         self.first_write = false;
                                         self.status.just_read = true;
                                         if self.nmi_interrupt_age < 2 {
                                             self.nmi_interrupt = false;
                                         }
                                         value
                                     }
                                     3 => self.oam_addr,
                                     4 => match self.oam_data {
                                         Some(value) => value,
                                         None => 0,
                                     },
                                     5 => 0,
                                     6 => 0,
                                     7 => {
                                         let buffer_data = self.read_buffer.take();
                                         if self.addr >= 0x3F00 {
                                             self.palette_data
                                         } else {
                                             buffer_data.unwrap()
                                         }
                                     },
                                     _ => panic!()
                                 },
                                 addr == 7 && self.addr >= 0x3F00)
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        let addr = (addr % 8) as usize;
        self.decay_register.write(value);
        match addr {
            0 => {
                self.ctrl = Ctrl::from_u8(value);
                if !self.ctrl.gen_nmi && self.nmi_interrupt_age < 2 {
                    self.nmi_interrupt = false;
                }
                if self.ctrl.gen_nmi && self.status.vertical_blank {
                    self.nmi_interrupt = true;
                    self.nmi_interrupt_age = 0;
                }
            }
            1 => self.mask = Mask::from_u8(value),
            2 => {}
            3 => self.oam_addr = value,
            4 => {
                self.oam_data = Some(value);
            }
            5 => {
                self.scroll = Some(value);
                self.first_write = !self.first_write;
            }
            6 => {
                self.addr_write = Some(value);
                self.first_write = !self.first_write;
            }
            7 => {
                self.data_write = Some(value);
            }
            _ => panic!()
        }
    }

    pub fn tick(&mut self) {
        self.decay_register.tick();
    }

    pub fn reset(&mut self) {
        self.ctrl = Ctrl::from_u8(0);
        self.mask = Mask::from_u8(0);
        self.scroll = None;
        self.addr_write = None;
        self.data_write = None;
    }
}
