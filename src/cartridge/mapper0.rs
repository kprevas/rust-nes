use cartridge::Cartridge;
use cartridge::CartridgeBus;
use cartridge::Header;
use cartridge::NametableMirroring;
use cartridge::NametableMirroring::*;

use std::cmp::max;

struct Mapper0Cpu {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
}

struct Mapper0Ppu {
    chr_rom: Vec<u8>,
    mirroring: NametableMirroring,
    uses_chr_ram: bool,
}

pub fn read(header: &Header, prg_rom: &[u8], chr_rom: &[u8]) -> Cartridge {
    let uses_chr_ram = chr_rom.len() == 0;
    Cartridge {
        cpu_bus: Box::new(Mapper0Cpu {
            prg_rom: prg_rom.to_vec(),
            prg_ram: vec![0; (u16::from(max(header.prg_ram_blocks, 1)) * 0x2000) as usize],
        }),
        ppu_bus: Box::new(Mapper0Ppu {
            chr_rom: if uses_chr_ram { vec!(0; 0x2000) } else { chr_rom.to_vec() },
            mirroring: if header.flags_6 & 0x1 == 1 { Vertical } else { Horizontal },
            uses_chr_ram,
        }),
    }
}

impl CartridgeBus for Mapper0Cpu {
    fn read_memory(&self, address: u16) -> u8 {
        match address {
            0x6000 ... 0x7FFF => self.prg_ram[(address - 0x6000) as usize],
            0x8000 ... 0xBFFF => self.prg_rom[(address - 0x8000) as usize],
            0xC000 ... 0xFFFF =>
                if self.prg_rom.len() <= 0x4000 {
                    self.prg_rom[(address - 0xC000) as usize]
                } else {
                    self.prg_rom[(address - 0x8000) as usize]
                }
            _ => panic!("bad memory read 0x{:04X}", address),
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x6000 ... 0x7FFF => self.prg_ram[(address - 0x6000) as usize] = value,
            _ => panic!("bad memory write 0x{:04X}", address),
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        address
    }
}

impl CartridgeBus for Mapper0Ppu {
    fn read_memory(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x1FFF => self.chr_rom[address as usize],
            _ => panic!("bad memory read 0x{:04X}", address),
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        if self.uses_chr_ram {
            match address {
                0x0000 ... 0x1FFF => self.chr_rom[address as usize] = value,
                _ => panic!("bad memory write 0x{:04X}", address),
            }
        } else {
            panic!("bad memory write 0x{:04X}", address);
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        match address {
            0x2000 ... 0x23FF => address - 0x2000,
            0x2400 ... 0x27FF => match self.mirroring {
                Vertical => address - 0x2000,
                Horizontal => address - 0x2400,
            },
            0x2800 ... 0x2BFF => match self.mirroring {
                Vertical => address - 0x2800,
                Horizontal => address - 0x2400,
            },
            0x2C00 ... 0x2FFF => address - 0x2800,
            _ => panic!("Bad nametable mirror request {:04X}", address),
        }
    }
}
