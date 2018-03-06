extern crate clap;
extern crate find_folder;
extern crate hex_slice;
extern crate image;
#[macro_use]
extern crate log;
extern crate nfd;
extern crate piston_window;
extern crate portaudio;
extern crate simple_error;

use cartridge::Cartridge;
use clap::ArgMatches;
use nfd::Response;
use piston_window::*;
use portaudio::PortAudio;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub mod apu;
pub mod cpu;
pub mod cartridge;
pub mod input;
pub mod ppu;

pub fn run(matches: clap::ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("disassemble") {
        let output_path = matches.value_of("OUTPUT");
        let mut out = match output_path {
            Some(ref path) => Box::new(File::create(&Path::new(path)).unwrap()) as Box<Write>,
            None => Box::new(std::io::stdout()) as Box<Write>,
        };
        let cartridge = if let Some(cartridge) = load_cartridge(matches) { cartridge } else { return };
        cpu::disassembler::disassemble(cartridge.cpu_bus, 0xc000, &mut out).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let window: PistonWindow = WindowSettings::new(
            "nes",
            [293, 240],
        )
            .exit_on_esc(true)
            .build()
            .unwrap();
        let mut window = window.ups(60).ups_reset(0);

        let mut cartridge = if let Some(cartridge) = load_cartridge(matches) { cartridge } else { return };

        let instrument_cpu = matches.is_present("instrument_cpu");
        let instrument_ppu = matches.is_present("instrument_ppu");

        let mut inputs: input::ControllerState = Default::default();
        let mut reset = false;

        let ppu_bus = RefCell::new(ppu::bus::PpuBus::new());
        let apu_bus = RefCell::new(apu::bus::ApuBus::new());

        let ppu = ppu::Ppu::new(&mut cartridge.ppu_bus, &ppu_bus, Some(&mut window), instrument_ppu);
        let apu = apu::Apu::new(&apu_bus, Some(PortAudio::new().unwrap())).unwrap();

        let mut cpu = cpu::Cpu::boot(&mut cartridge.cpu_bus, ppu, &ppu_bus, apu, &apu_bus, instrument_cpu);

        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("src").unwrap();
        let ref font = assets.join("VeraMono.ttf");
        let factory = window.factory.clone();
        let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();

        while let Some(e) = window.next() {
            inputs.event(&e, &mut reset);

            if let Some(u) = e.update_args() {
                if reset {
                    reset = false;
                    cpu.reset(true);
                }
                cpu.do_frame(u.dt, inputs);
            }

            if let Some(_r) = e.render_args() {
                window.draw_2d(&e, |c, gl| {
                    cpu.render(c, gl, &mut glyphs);
                });
            }

            if let Some(_c) = e.close_args() {
                cpu.close();
            }
        }
    }
}

fn load_cartridge(matches: &ArgMatches) -> Option<Cartridge> {
    let cartridge: cartridge::Cartridge = loop {
        let input_file = match matches.value_of("INPUT") {
            Some(i) => Some(PathBuf::from(i)),
            None => {
                match nfd::open_file_dialog(None, None).unwrap() {
                    Response::Okay(p) => Some(PathBuf::from(p)),
                    Response::OkayMultiple(v) => Some(PathBuf::from(&v[0])),
                    Response::Cancel => None,
                }
            }
        };
        if let Some(input_file) = input_file {
            match cartridge::read(File::open(input_file).as_mut().unwrap()) {
                Ok(c) => break c,
                Err(e) => if matches.is_present("INPUT") {
                    panic!(e);
                },
            };
        } else {
            return None;
        }
    };
    Some(cartridge)
}
