use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use piston_window::*;
use portaudio::PortAudio;
use simple_error::SimpleResult;

use nes::cartridge::Cartridge;
use window::window_loop;

pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod ppu;

pub fn load_cartridge(
    src: &mut dyn Read,
    save_data: Option<&mut dyn Read>,
) -> SimpleResult<Cartridge> {
    cartridge::read(src, save_data)
}

pub fn disassemble(
    cartridge: Cartridge,
    mut out: &mut Box<dyn Write>,
) -> Result<(), Box<dyn Error>> {
    cpu::disassembler::disassemble(cartridge.cpu_bus, 0x8000, &mut out)
}

pub fn run(
    matches: &ArgMatches,
    mut cartridge: Cartridge,
    save_path: PathBuf,
    mut window: PistonWindow<sdl2_window::Sdl2Window>,
) {
    window.set_size([293, 240]);
    let mut window = window
        .ups(60)
        .ups_reset(0)
        .bench_mode(matches.is_present("bench_mode"));
    let instrument_cpu = matches.is_present("instrument_cpu");
    let instrument_ppu = matches.is_present("instrument_ppu");

    let mut inputs = [::input::player_1_nes(), ::input::player_2_nes()];
    let record_path = save_path.with_extension("rcd");

    let ppu_bus = RefCell::new(ppu::bus::PpuBus::new());
    let apu_bus = RefCell::new(apu::bus::ApuBus::new());

    let ppu = ppu::Ppu::new(
        &mut cartridge.ppu_bus,
        &ppu_bus,
        Some(&mut window),
        instrument_ppu,
    );
    let apu = apu::Apu::new(&apu_bus, Some(PortAudio::new().unwrap())).unwrap();

    let mut cpu = cpu::Cpu::boot(
        &mut cartridge.cpu_bus,
        ppu,
        &ppu_bus,
        apu,
        &apu_bus,
        instrument_cpu,
    );

    window_loop(
        window,
        &mut inputs,
        &record_path,
        &mut cpu,
        293.0,
        240.0,
        &Path::new("settings_nes.dat"),
        matches.is_present("pause"),
    );

    cpu.close();
    let mut save: Vec<u8> = Vec::new();
    cpu.save_to_battery(&mut save).unwrap();
    if save.len() > 0 {
        File::create(save_path.as_path())
            .unwrap()
            .write(save.as_slice())
            .unwrap();
    }
}
