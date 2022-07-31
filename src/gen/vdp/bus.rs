enum AddrMode {
    Read,
    Write,
}

enum AddrTarget {
    VRAM,
    CRAM,
    VSRAM,
}

struct Addr {
    mode: AddrMode,
    target: AddrTarget,
    addr: u16,
    vram_to_vram: bool,
    dma: bool,
}

impl Addr {
    fn from_u32(val: u32) -> Addr {
        let addr = (((val & 0b11) << 14) | ((val >> 16) & 0x3FFF)) as u16;
        let vram_to_vram = ((val >> 6) & 0b1) == 1;
        let dma = ((val >> 7) & 0b1) == 1;
        let (mode, target) = match (((val >> 4) & 0b11) << 2) | (val >> 30) {
            0b0000 => (AddrMode::Read, AddrTarget::VRAM),
            0b0001 => (AddrMode::Write, AddrTarget::VRAM),
            0b1000 => (AddrMode::Read, AddrTarget::CRAM),
            0b0011 => (AddrMode::Write, AddrTarget::CRAM),
            0b0100 => (AddrMode::Read, AddrTarget::VSRAM),
            0b0101 => (AddrMode::Write, AddrTarget::VSRAM),
            _ => panic!(),
        };
        Addr {
            mode,
            target,
            addr,
            vram_to_vram,
            dma,
        }
    }
}

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

struct Mode1 {
    blank_leftmost_8: bool,
    enable_horizontal_interrupt: bool,
    use_high_color_bits: bool,
    freeze_hv_on_level_2_interrupt: bool,
    disable_display: bool,
}

impl Mode1 {
    fn from_u8(val: u8) -> Mode1 {
        Mode1 {
            blank_leftmost_8: (val & 0b00100000) > 0,
            enable_horizontal_interrupt: (val & 0b00010000) > 0,
            use_high_color_bits: (val & 0b00000100) > 0,
            freeze_hv_on_level_2_interrupt: (val & 0b00000010) > 0,
            disable_display: (val & 0b00000001) > 0,
        }
    }
}

struct Mode2 {
    use_128k_vram: bool,
    enable_display: bool,
    enable_vertical_interrupt: bool,
    enable_dma: bool,
    pal_mode: bool,
    mode_5: bool,
}

impl Mode2 {
    fn from_u8(val: u8) -> Mode2 {
        Mode2 {
            use_128k_vram: (val & 0b10000000) > 0,
            enable_display: (val & 0b01000000) > 0,
            enable_vertical_interrupt: (val & 0b00100000) > 0,
            enable_dma: (val & 0b00010000) > 0,
            pal_mode: (val & 0b00001000) > 0,
            mode_5: (val & 0b00000100) > 0,
        }
    }
}

enum VerticalScrollingMode { Column16Pixels, FullScreen }

enum HorizontalScrollingMode { Row1Pixel, Row8Pixel, FullScreen, Invalid }

struct Mode3 {
    enable_external_interrupt: bool,
    vertical_scrolling_mode: VerticalScrollingMode,
    horizontal_scrolling_mode: HorizontalScrollingMode,
}

impl Mode3 {
    fn from_u8(val: u8) -> Mode3 {
        Mode3 {
            enable_external_interrupt: (val & 0b00001000) > 0,
            vertical_scrolling_mode: if (val & 0b00000100) > 0 {
                VerticalScrollingMode::Column16Pixels
            } else {
                VerticalScrollingMode::FullScreen
            },
            horizontal_scrolling_mode: match val & 0b00000011 {
                0b00 => HorizontalScrollingMode::FullScreen,
                0b01 => HorizontalScrollingMode::Invalid,
                0b10 => HorizontalScrollingMode::Row8Pixel,
                0b11 => HorizontalScrollingMode::Row1Pixel,
                _ => panic!(),
            },
        }
    }
}

enum InterlaceMode { NoInterlace, InterlaceNormal, InterlaceDouble }

struct Mode4 {
    wide_mode: bool,
    freeze_hsync: bool,
    pixel_clock_signal_on_vsync: bool,
    enable_external_pixel_bus: bool,
    enable_shadow_highlight: bool,
    interlace_mode: InterlaceMode,
}

impl Mode4 {
    fn from_u8(val: u8) -> Mode4 {
        Mode4 {
            wide_mode: (val & 0b10000000) > 0,
            freeze_hsync: (val & 0b01000000) > 0,
            pixel_clock_signal_on_vsync: (val & 0b00100000) > 0,
            enable_external_pixel_bus: (val & 0b00010000) > 0,
            enable_shadow_highlight: (val & 0b00001000) > 0,
            interlace_mode: match (val & 0b00000110) >> 1 {
                0b00 | 0b10 => InterlaceMode::NoInterlace,
                0b01 => InterlaceMode::InterlaceNormal,
                0b11 => InterlaceMode::InterlaceDouble,
                _ => panic!()
            },
        }
    }
}

pub struct VdpBus {
    status: Status,
    beam_vpos: u16,
    beam_hpos: u16,
    mode_1: Mode1,
    mode_2: Mode2,
    mode_3: Mode3,
    mode_4: Mode4,
    addr: Addr,
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
            beam_vpos: 0,
            beam_hpos: 0,
            mode_1: Mode1 {
                blank_leftmost_8: false,
                enable_horizontal_interrupt: false,
                use_high_color_bits: false,
                freeze_hv_on_level_2_interrupt: false,
                disable_display: false,
            },
            mode_2: Mode2 {
                use_128k_vram: false,
                enable_display: false,
                enable_vertical_interrupt: false,
                enable_dma: false,
                pal_mode: false,
                mode_5: false,
            },
            mode_3: Mode3 {
                enable_external_interrupt: false,
                vertical_scrolling_mode: VerticalScrollingMode::Column16Pixels,
                horizontal_scrolling_mode: HorizontalScrollingMode::Row1Pixel,
            },
            mode_4: Mode4 {
                wide_mode: false,
                freeze_hsync: false,
                pixel_clock_signal_on_vsync: false,
                enable_external_pixel_bus: false,
                enable_shadow_highlight: false,
                interlace_mode: InterlaceMode::NoInterlace,
            },
            addr: Addr {
                mode: AddrMode::Read,
                target: AddrTarget::VRAM,
                addr: 0,
                vram_to_vram: false,
                dma: false,
            },
        }
    }

    pub fn read_byte(&mut self, addr: u32) -> u8 {
        match addr {
            0xC004 | 0xC006 | 0xC008 | 0xC00A | 0xC00C | 0xC00E => {
                (self.read_word(addr) >> 8) as u8
            }
            0xC005 | 0xC007 | 0xC009 | 0xC00B | 0xC00D | 0xC00F => {
                (self.read_word(addr - 1) & 0xFF) as u8
            }
            _ => panic!(),
        }
    }

    pub fn read_word(&mut self, addr: u32) -> u16 {
        match addr {
            0xC004 | 0xC006 => self.status.to_u16(),
            0xC008 | 0xC00A | 0xC00C | 0xC00E => {
                if self.interlace_mode {
                    ((self.beam_vpos >> 1) << 9)
                        | (self.beam_vpos & 0b100000000)
                        | ((self.beam_hpos >> 1) & 0xFF)
                } else {
                    (self.beam_vpos << 8) | ((self.beam_hpos >> 1) & 0xFF)
                }
            }
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
            0xC0004 | 0xC0005 => panic!(), // TODO: allowed?
            _ => panic!(),
        }
    }

    pub fn write_word(&mut self, addr: u32, data: u16) {
        match addr {
            0xC0004 => match (data >> 8) | 0b11111 {
                0x00 => self.mode_1 = Mode1::from_u8((data & 0xFF) as u8),
                0x01 => self.mode_2 = Mode2::from_u8((data & 0xFF) as u8),
                0x0B => self.mode_3 = Mode3::from_u8((data & 0xFF) as u8),
                0x0C => self.mode_4 = Mode4::from_u8((data & 0xFF) as u8),
                _ => panic!(),
            }
            0xC0011 | 0xC0013 | 0xC0015 | 0xC0017 => {} // TODO: PSG
            0xC001C | 0xC001E => {}                     // TODO: debug register
            _ => panic!(),
        }
    }

    pub fn write_long(&mut self, addr: u32, data: u32) {
        match addr {
            0xC0004 => {
                if data >> 14 == 0b10 {
                    self.write_word(addr, (data >> 16) as u16);
                    self.write_word(addr, (data & 0xFFFF) as u16);
                } else {
                    self.addr = Addr::from_u32(data);
                }
            }
            _ => panic!(),
        }
    }
}
