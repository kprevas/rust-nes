use cartridge::Cartridge;
use cartridge::CartridgeBus;
use cartridge::Header;
use cartridge::NametableMirroring;
use cartridge::NametableMirroring::*;
use std::cell::Cell;
use std::rc::Rc;

struct Mapper3Cpu {
    prg_rom: Vec<u8>,
    chr_bank: Rc<Cell<usize>>,
}

struct Mapper3Ppu {
    chr_rom: Vec<u8>,
    mirroring: NametableMirroring,
    chr_bank: Rc<Cell<usize>>,
}

pub fn read(header: &Header, prg_rom: &[u8], chr_rom: &[u8]) -> Cartridge {
    let chr_bank = Rc::new(Cell::new(0usize));
    Cartridge {
        cpu_bus: Box::new(Mapper3Cpu {
            prg_rom: prg_rom.to_vec(),
            chr_bank: chr_bank.clone(),
        }),
        ppu_bus: Box::new(Mapper3Ppu {
            chr_rom: chr_rom.to_vec(),
            mirroring: if header.flags_6 & 0x1 == 1 { Vertical } else { Horizontal },
            chr_bank: chr_bank.clone(),
        }),
    }
}

impl CartridgeBus for Mapper3Cpu {
    fn read_memory(&self, address: u16, open_bus: u8) -> u8 {
        match address {
            0x8000 ... 0xBFFF => self.prg_rom[(address - 0x8000) as usize],
            0xC000 ... 0xFFFF =>
                if self.prg_rom.len() <= 0x4000 {
                    self.prg_rom[(address - 0xC000) as usize]
                } else {
                    self.prg_rom[(address - 0x8000) as usize]
                }
            _ => open_bus,
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x8000 ... 0xFFFF => {
                self.chr_bank.replace((value & 0x3) as usize);
            }
            _ => panic!("bad memory write 0x{:04X}", address),
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        address
    }
}

impl CartridgeBus for Mapper3Ppu {
    fn read_memory(&self, address: u16, open_bus: u8) -> u8 {
        match address {
            0x0000 ... 0x1FFF => self.chr_rom[(self.chr_bank.get() * 0x2000) + address as usize],
            _ => open_bus,
        }
    }

    fn write_memory(&mut self, address: u16, _value: u8) {
        panic!("bad memory write 0x{:04X}", address);
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        match address {
            0x2000 ... 0x23FF => address - 0x2000,
            0x2400 ... 0x27FF => match self.mirroring {
                Vertical => address - 0x2000,
                Horizontal => address - 0x2400,
                _ => unimplemented!(),
            },
            0x2800 ... 0x2BFF => match self.mirroring {
                Vertical => address - 0x2800,
                Horizontal => address - 0x2400,
                _ => unimplemented!(),
            },
            0x2C00 ... 0x2FFF => address - 0x2800,
            _ => panic!("Bad nametable mirror request {:04X}", address),
        }
    }
}
