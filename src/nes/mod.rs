use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::ArgMatches;
use nfd::Response;
use piston_window::*;
use portaudio::PortAudio;

use nes::cartridge::Cartridge;
use window::window_loop;

pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod ppu;

fn load_cartridge(matches: &ArgMatches) -> Option<(Cartridge, PathBuf)> {
    let mut save_path: PathBuf;
    let cartridge: Cartridge = loop {
        let input_file = match matches.value_of("INPUT") {
            Some(i) => Some(PathBuf::from(i)),
            None => match nfd::open_file_dialog(None, None).unwrap() {
                Response::Okay(p) => Some(PathBuf::from(p)),
                Response::OkayMultiple(v) => Some(PathBuf::from(&v[0])),
                Response::Cancel => None,
            },
        };
        if let Some(input_file) = input_file {
            save_path = PathBuf::from(".")
                .join(input_file.file_name().unwrap())
                .with_extension("sav");
            match cartridge::read(
                File::open(input_file).as_mut().unwrap(),
                match File::open(save_path.as_path()) {
                    Ok(ref mut file) => Some(file),
                    Err(_) => None,
                },
            ) {
                Ok(c) => break c,
                Err(e) => {
                    if matches.is_present("INPUT") {
                        panic!("{}", e);
                    }
                }
            };
        } else {
            return None;
        }
    };
    Some((cartridge, save_path))
}

pub fn disasssemble(matches: &ArgMatches, mut out: &mut Box<dyn Write>) {
    let cartridge = if let Some((cartridge, _)) = load_cartridge(matches) {
        cartridge
    } else {
        return;
    };
    cpu::disassembler::disassemble(cartridge.cpu_bus, 0x8000, &mut out).unwrap();
}

pub fn run(matches: &ArgMatches) {
    let window: PistonWindow<sdl2_window::Sdl2Window> =
        WindowSettings::new("nes", [293, 240]).build().unwrap();
    let mut window = window
        .ups(60)
        .ups_reset(0)
        .bench_mode(matches.is_present("bench_mode"));

    let mut cartridge;
    let save_path;
    if let Some((c, s)) = load_cartridge(matches) {
        cartridge = c;
        save_path = s;
    } else {
        return;
    }

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

    window_loop(window, &mut inputs, &record_path, &mut cpu, 293.0, 240.0);

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
