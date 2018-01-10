use cartridge::Cartridge;
use cartridge::Header;

use std::cmp::max;

struct Mapper0 {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rom: Vec<u8>,
}

pub fn read(header: &Header, prg_rom: &[u8], chr_rom: &[u8]) -> Box<Cartridge> {
    Box::new(Mapper0 {
        prg_rom: prg_rom.to_vec(),
        prg_ram: vec![0; (u16::from(max(header.prg_ram_blocks, 1)) * 0x2000) as usize],
        chr_rom: chr_rom.to_vec(),
    })
}

impl Cartridge for Mapper0 {
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
}