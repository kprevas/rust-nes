use std::cmp::max;
use std::io::prelude::*;
use std::io::Result;

use bytes::*;

use nes::cartridge::Cartridge;
use nes::cartridge::CartridgeBus;
use nes::cartridge::Header;
use nes::cartridge::NametableMirroring;
use nes::cartridge::NametableMirroring::*;

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
            chr_rom: if uses_chr_ram {
                vec![0; 0x2000]
            } else {
                chr_rom.to_vec()
            },
            mirroring: header.mirroring,
            uses_chr_ram,
        }),
    }
}

impl CartridgeBus for Mapper0Cpu {
    fn read_memory(&self, address: u16, open_bus: u8) -> u8 {
        match address {
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize],
            0x8000..=0xBFFF => self.prg_rom[(address - 0x8000) as usize],
            0xC000..=0xFFFF => {
                if self.prg_rom.len() <= 0x4000 {
                    self.prg_rom[(address - 0xC000) as usize]
                } else {
                    self.prg_rom[(address - 0x8000) as usize]
                }
            }
            _ => open_bus,
        }
    }

    fn write_memory(&mut self, address: u16, value: u8, _cpu_cycle: u64) {
        match address {
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize] = value,
            _ => (),
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        address
    }

    fn save_to_battery(&self, _out: &mut dyn Write) -> Result<usize> {
        Ok(0)
    }

    fn load_from_battery(&mut self, _inp: &mut dyn Read) -> Result<usize> {
        unimplemented!();
    }

    fn save_state(&self, out: &mut Vec<u8>) {
        out.put_slice(&self.prg_ram);
    }

    fn load_state(&mut self, state: &mut dyn Buf) {
        state.copy_to_slice(&mut self.prg_ram);
    }
}

impl CartridgeBus for Mapper0Ppu {
    fn read_memory(&self, address: u16, open_bus: u8) -> u8 {
        match address {
            0x0000..=0x1FFF => self.chr_rom[address as usize],
            _ => open_bus,
        }
    }

    fn write_memory(&mut self, address: u16, value: u8, _cpu_cycle: u64) {
        if self.uses_chr_ram {
            match address {
                0x0000..=0x1FFF => self.chr_rom[address as usize] = value,
                _ => (),
            }
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        match address {
            0x2000..=0x23FF => address - 0x2000,
            0x2400..=0x27FF => match self.mirroring {
                Vertical => address - 0x2000,
                Horizontal => address - 0x2400,
                _ => unimplemented!(),
            },
            0x2800..=0x2BFF => match self.mirroring {
                Vertical => address - 0x2800,
                Horizontal => address - 0x2400,
                _ => unimplemented!(),
            },
            0x2C00..=0x2FFF => address - 0x2800,
            _ => panic!("Bad nametable mirror request {:04X}", address),
        }
    }

    fn save_to_battery(&self, _out: &mut dyn Write) -> Result<usize> {
        unimplemented!();
    }

    fn load_from_battery(&mut self, _inp: &mut dyn Read) -> Result<usize> {
        unimplemented!();
    }

    fn save_state(&self, out: &mut Vec<u8>) {
        if self.uses_chr_ram {
            out.put_slice(&self.chr_rom);
        }
    }

    fn load_state(&mut self, state: &mut dyn Buf) {
        if self.uses_chr_ram {
            state.copy_to_slice(&mut self.chr_rom);
        }
    }
}
