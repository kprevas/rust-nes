use std::cell::RefCell;
use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use piston_window::*;
use simple_error::SimpleResult;

use Commands;
use window::window_loop;

pub mod cartridge;
pub mod m68k;
pub mod vdp;
pub mod z80;

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
    command: Commands,
    cartridge: Box<[u8]>,
    save_path: PathBuf,
    mut window: PistonWindow<sdl2_window::Sdl2Window>,
) {
    if let Commands::Run {
        instrument_cpu,
        bench_mode,
        dump_vram,
        pause,
        ..
    } = command
    {
        window.set_size([320, 224]);
        let mut window = window.ups(60).bench_mode(bench_mode);

        let mut inputs = [::input::player_1_gen(), ::input::player_2_gen()];
        let record_path = save_path.with_extension("rcd");

        let vdp_bus = RefCell::new(vdp::bus::VdpBus::new(instrument_cpu));

        let vdp = vdp::Vdp::new(&vdp_bus, Some(&mut window), dump_vram, instrument_cpu);
        let mut cpu = m68k::Cpu::boot(&cartridge, Some(vdp), &vdp_bus, instrument_cpu);

        window_loop(
            window,
            &mut inputs,
            &record_path,
            &mut cpu,
            320.0,
            224.0,
            &Path::new("settings_gen.dat"),
            pause,
            instrument_cpu,
        );

        cpu.close();
    } else {
        panic!()
    }
}
