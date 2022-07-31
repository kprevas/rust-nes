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

pub struct VdpBus {
    status: Status,
    beam_vpos: u16,
    beam_hpos: u16,
    interlace_mode: bool,
    // TODO put in mode register
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
            interlace_mode: false,
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

    pub fn write_word(&mut self, addr: u32, _data: u16) {
        match addr {
            0xC0004 => {
                // TODO: set register
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
