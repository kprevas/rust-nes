use std::cell::RefCell;
use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use piston_window::*;
use simple_error::SimpleResult;

use window::window_loop;

pub mod cartridge;
pub mod m68k;
pub mod vdp;

pub fn load_cartridge(
    src: &mut dyn Read,
    save_data: Option<&mut dyn Read>,
) -> SimpleResult<Box<[u8]>> {
    cartridge::read(src, save_data)
}

pub fn disassemble(
    cartridge: Box<[u8]>,
    mut out: &mut Box<dyn Write>,
) -> Result<(), Box<dyn Error>> {
    m68k::disassembler::disassemble(cartridge, &mut out)
}

pub fn run(
    matches: &ArgMatches,
    cartridge: Box<[u8]>,
    save_path: PathBuf,
    mut window: PistonWindow<sdl2_window::Sdl2Window>,
) {
    window.set_size([320, 224]);
    let mut window = window
        .ups(60)
        .bench_mode(matches.is_present("bench_mode"));

    let mut inputs = [::input::player_1_gen(), ::input::player_2_gen()];
    let record_path = save_path.with_extension("rcd");

    let instrument_cpu = matches.is_present("instrument_cpu");

    let vdp_bus = RefCell::new(vdp::bus::VdpBus::new());

    let vdp = vdp::Vdp::new(&vdp_bus, Some(&mut window), matches.is_present("dump_vram"));
    let mut cpu = m68k::Cpu::boot(&cartridge, Some(vdp), &vdp_bus, instrument_cpu);

    window_loop(
        window,
        &mut inputs,
        &record_path,
        &mut cpu,
        320.0,
        224.0,
        &Path::new("settings_gen.dat"),
        matches.is_present("pause"),
    );

    cpu.close();
}
