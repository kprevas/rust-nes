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
        let cartridge = if let Some((cartridge, _)) = load_cartridge(matches) { cartridge } else { return };
        cpu::disassembler::disassemble(cartridge.cpu_bus, 0x8000, &mut out).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let window: PistonWindow = WindowSettings::new(
            "nes",
            [293, 240],
        )
            .build()
            .unwrap();
        let mut window = window.ups(60).ups_reset(0);

        let mut cartridge;
        let save_path;
        if let Some((c, s)) = load_cartridge(matches) {
            cartridge = c;
            save_path = s;
        } else {
            return
        }

        let instrument_cpu = matches.is_present("instrument_cpu");
        let instrument_ppu = matches.is_present("instrument_ppu");

        let mut inputs = [input::ControllerState::player_1(), input::ControllerState::player_2()];
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

        let mut scale = 1.0;
        let mut x_trans = 0.0;
        let mut y_trans = 0.0;

        while let Some(e) = window.next() {
            inputs[0].event(&e);
            inputs[1].event(&e);
            if let Some(Button::Keyboard(Key::R)) = e.press_args() {
                reset = true;
            }

            if let Some(u) = e.update_args() {
                if reset {
                    reset = false;
                    cpu.reset(true);
                }
                cpu.do_frame(u.dt, &inputs);
            }

            if let Some(_r) = e.render_args() {
                window.draw_2d(&e, |c, gl| {
                    cpu.render(c.trans(x_trans, y_trans).scale(scale, scale), gl, &mut glyphs);
                });
            }

            if let Some(r) = e.resize_args() {
                let width = r[0] as f64;
                let height = r[1] as f64;
                let x_scale = width / 293.0;
                let y_scale = height / 240.0;
                scale = x_scale.min(y_scale);
                x_trans = (width - 293.0 * scale) / 2.0;
                y_trans = (height - 240.0 * scale) / 2.0;
            }

            if let Some(_c) = e.close_args() {
                cpu.close();
                let mut save: Vec<u8> = Vec::new();
                cpu.save_to_battery(&mut save).unwrap();
                if save.len() > 0 {
                    File::create(save_path.as_path()).unwrap().write(save.as_slice()).unwrap();
                }
            }
        }
    }
}

fn load_cartridge(matches: &ArgMatches) -> Option<(Cartridge, PathBuf)> {
    let mut save_path: PathBuf;
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
            save_path = PathBuf::from(".").join(input_file.file_name().unwrap()).with_extension("sav");
            match cartridge::read(File::open(input_file).as_mut().unwrap(),
                                  match File::open(save_path.as_path()) {
                                      Ok(ref mut file) => Some(file),
                                      Err(_) => None,
                                  }) {
                Ok(c) => break c,
                Err(e) => if matches.is_present("INPUT") {
                    panic!(e);
                },
            };
        } else {
            return None;
        }
    };
    Some((cartridge, save_path))
}
