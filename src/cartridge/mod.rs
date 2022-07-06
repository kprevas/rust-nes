use std::io::prelude::*;
use std::io::Result;

use bytes::Buf;
use simple_error::*;

mod mapper0;
mod mapper1;
mod mapper3;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
enum NametableMirroring {
    Vertical,
    Horizontal,
    SingleScreen,
    _FourScreen,
    _Diagonal,
    _LShaped,
    _ThreeScreenVertical,
    _ThreeScreenHorizontal,
    _ThreeScreenDiagonal,
    _SingleScreenFixed,
}

pub struct Cartridge {
    pub cpu_bus: Box<dyn CartridgeBus>,
    pub ppu_bus: Box<dyn CartridgeBus>,
}

pub trait CartridgeBus {
    fn read_memory(&self, address: u16, open_bus: u8) -> u8;
    fn write_memory(&mut self, address: u16, value: u8, cpu_cycle: u64);
    fn mirror_nametable(&self, address: u16) -> u16;
    fn save_to_battery(&self, out: &mut dyn Write) -> Result<usize>;
    fn load_from_battery(&mut self, inp: &mut dyn Read) -> Result<usize>;
    fn save_state(&self, out: &mut Vec<u8>);
    fn load_state(&mut self, state: &mut dyn Buf);
}

#[derive(Debug)]
pub struct Header {
    prg_rom_blocks: u8,
    chr_rom_blocks: u8,
    prg_ram_blocks: u8,
    flags_6: u8,
    flags_7: u8,
    _flags_9: u8,
    _flags_10: u8,

    mirroring: NametableMirroring,
    battery_save: bool,
}

pub fn read(src: &mut dyn Read, save_data: Option<&mut dyn Read>) -> SimpleResult<Cartridge> {
    let mut contents = Vec::new();
    src.read_to_end(&mut contents).expect("error reading source");
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
        mirroring: if contents[6] & 0b1 > 0 { NametableMirroring::Vertical } else { NametableMirroring::Horizontal },
        battery_save: contents[6] & 0b10 > 0,
    };
    info!(target: "cartridge", "header: {:?}", header);
    assert_eq!([0, 0, 0, 0, 0], contents[11..16]);
    // TODO check for trainer
    let prg_end = 16 + (u32::from(header.prg_rom_blocks) * 0x4000) as usize;
    let chr_end = prg_end + (u32::from(header.chr_rom_blocks) * 0x2000) as usize;
    let prg_rom = &contents[16..prg_end];
    let chr_rom = &contents[prg_end..chr_end];

    let mapper = (header.flags_6 >> 4) + (header.flags_7 & 0b11110000);
    info!(target: "cartridge", "Using mapper {}", mapper);

    let mut cartridge = match mapper {
        0 => Ok(mapper0::read(&header, prg_rom, chr_rom)),
        1 => Ok(mapper1::read(&header, prg_rom, chr_rom)),
        3 => Ok(mapper3::read(&header, prg_rom, chr_rom)),
        _ => unimplemented!()
    };

    if let Ok(ref mut cartridge) = cartridge {
        if let Some(save_data) = save_data {
            let bytes = cartridge.cpu_bus.load_from_battery(save_data)
                .map_err(|io_error| SimpleError::new(io_error.to_string()))?;
            info!(target: "cartridge", "{} bytes loaded", bytes);
        }
    }

    cartridge
}
