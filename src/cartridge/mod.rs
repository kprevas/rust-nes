mod mapper0;

use std::io::prelude::*;
use simple_error::*;

enum NametableMirroring {
    Vertical,
    Horizontal,
}

pub struct Cartridge {
    pub cpu_bus: Box<CartridgeBus>,
    pub ppu_bus: Box<CartridgeBus>,
}

pub trait CartridgeBus {
    fn read_memory(&self, address: u16) -> u8;
    fn write_memory(&mut self, address: u16, value: u8);
    fn mirror_nametable(&self, address: u16) -> u16;
}

pub struct Header {
    prg_rom_blocks: u8,
    chr_rom_blocks: u8,
    prg_ram_blocks: u8,
    flags_6: u8,
    flags_7: u8,
    _flags_9: u8,
    _flags_10: u8,
}

pub fn read(src: &mut Read) -> SimpleResult<Cartridge> {
    let mut contents = Box::new([0; 0xbfe0]);
    src.read(contents.as_mut()).expect("error reading source");
    if contents[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
        return Err(SimpleError::new("Not a NES file."))
    }
    let header = Header {
        prg_rom_blocks: contents[4],
        chr_rom_blocks: contents[5],
        prg_ram_blocks: contents[8],
        flags_6: contents[6],
        flags_7: contents[7],
        _flags_9: contents[9],
        _flags_10: contents[10],
    };
    assert_eq!([0, 0, 0, 0, 0], contents[11..16]);
    // TODO check for trainer
    let prg_end = 16 + (u16::from(header.prg_rom_blocks) * 0x4000) as usize;
    let chr_end = prg_end + (u16::from(header.chr_rom_blocks) * 0x2000) as usize;
    let prg_rom = &contents[16..prg_end];
    let chr_rom = &contents[prg_end..chr_end];

    let mapper = (header.flags_6 >> 4) + (header.flags_7 & 0b11110000);
    info!("Using mapper {}", mapper);

    match mapper {
        0 => Ok(mapper0::read(&header, prg_rom, chr_rom)),
        _ => unimplemented!()
    }
}
