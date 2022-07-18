#![feature(mixed_integer_ops)]

extern crate bincode;
extern crate bytes;
extern crate clap;
extern crate core;
extern crate dasp;
extern crate find_folder;
extern crate hex_slice;
extern crate image;
#[macro_use]
extern crate log;
extern crate nfd;
extern crate num_integer;
extern crate num_traits;
extern crate piston_window;
extern crate portaudio;
extern crate sdl2_window;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate simple_error;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use nfd::Response;
use piston_window::*;
use portaudio::PortAudio;

use cartridge::Cartridge;
use menu::NES_CONTROLS;

pub mod apu;
pub mod cartridge;
pub mod control;
pub mod cpu;
pub mod input;
pub mod m68k;
pub mod menu;
pub mod ppu;
pub mod record;
pub mod md_cartridge;

pub fn run(matches: ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("disassemble") {
        let output_path = matches.value_of("OUTPUT");
        let mut out = match output_path {
            Some(ref path) => Box::new(File::create(&Path::new(path)).unwrap()) as Box<dyn Write>,
            None => Box::new(std::io::stdout()) as Box<dyn Write>,
        };
        let cartridge = if let Some((cartridge, _)) = load_cartridge(matches) {
            cartridge
        } else {
            return;
        };
        cpu::disassembler::disassemble(cartridge.cpu_bus, 0x8000, &mut out).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("run") {
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

        let mut inputs = [input::player_1_nes(), input::player_2_nes()];
        let mut reset = false;
        let mut input_overlay = false;

        let mut frame_count = 0u32;
        let record_path = save_path.with_extension("rcd");
        let mut recorder = record::Recorder::new(&record_path);

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

        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("src")
            .unwrap();
        let ref font = assets.join("VeraMono.ttf");
        let mut glyphs = Glyphs::new(
            font,
            window.create_texture_context(),
            TextureSettings::new(),
        )
            .unwrap();
        let mut texture_ctx = window.create_texture_context();

        let mut scale = 1.0;
        let mut x_trans = 0.0;
        let mut y_trans = 0.0;

        let mut control = control::Control::new();

        let mut input_changed = false;

        let mut menu = menu::Menu::new(NES_CONTROLS, &inputs);
        menu.update_controls(&mut inputs);

        while let Some(e) = window.next() {
            let menu_handled = menu.event(&e);
            if !menu_handled {
                input_changed |= inputs[0].event(&e);
                input_changed |= inputs[1].event(&e);
                control.event(
                    &e,
                    &mut cpu,
                    &mut reset,
                    &mut input_overlay,
                    &mut recorder,
                    frame_count,
                );
            } else {
                menu.update_controls(&mut inputs);
            }

            if let Some(u) = e.update_args() {
                if reset {
                    reset = false;
                    cpu.reset(true);
                }
                if input_changed {
                    recorder.input_changed(&inputs, frame_count);
                    input_changed = false;
                }
                recorder.set_frame_inputs(&mut inputs, frame_count);
                cpu.do_frame(u.dt, &inputs);
                frame_count += 1;
            }

            if let Some(_r) = e.render_args() {
                window.draw_2d(&e, |c, gl, _| {
                    let trans = c.trans(x_trans, y_trans).scale(scale, scale);
                    cpu.render(trans, &mut texture_ctx, gl, &mut glyphs);
                    recorder.render_overlay(c, gl);
                    if input_overlay {
                        inputs[0].render_overlay(trans.trans(10.0, 230.0), gl, &mut glyphs);
                        inputs[1].render_overlay(trans.trans(170.0, 230.0), gl, &mut glyphs);
                    }
                    menu.render(trans, gl, &mut glyphs);
                });
            }

            if let Some(r) = e.resize_args() {
                let width = r.draw_size[0] as f64;
                let height = r.draw_size[1] as f64;
                let x_scale = width / 293.0;
                let y_scale = height / 240.0;
                scale = x_scale.min(y_scale);
                x_trans = (width - 293.0 * scale) / 2.0;
                y_trans = (height - 240.0 * scale) / 2.0;
            }
        }

        cpu.close();
        let mut save: Vec<u8> = Vec::new();
        cpu.save_to_battery(&mut save).unwrap();
        if save.len() > 0 {
            File::create(save_path.as_path())
                .unwrap()
                .write(save.as_slice())
                .unwrap();
        }
        recorder.stop();
        menu.save_settings();
    } else if let Some(matches) = matches.subcommand_matches("m68k") {
        let window: PistonWindow<sdl2_window::Sdl2Window> =
            WindowSettings::new("gen", [320, 224]).build().unwrap();
        let mut window = window
            .ups(60)
            .ups_reset(0)
            .bench_mode(matches.is_present("bench_mode"));

        let mut reset = false;

        let mut frame_count = 0u32;

        let mut inputs = [input::player_1_nes(), input::player_2_nes()];

        let instrument_cpu = matches.is_present("instrument_cpu");

        let mut cartridge;
        let save_path;
        if let Some((c, s)) = load_md_cartridge(matches) {
            cartridge = c;
            save_path = s;
        } else {
            return;
        }

        let mut cpu = m68k::Cpu::boot(&cartridge, instrument_cpu);

        while let Some(e) = window.next() {
            if let Some(u) = e.update_args() {
                if reset {
                    reset = false;
                    cpu.reset(true);
                }
                cpu.do_frame(u.dt, &inputs);
                frame_count += 1;
            }
        }

        cpu.close();
    }
}

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

fn load_md_cartridge(matches: &ArgMatches) -> Option<(Box<[u8]>, PathBuf)> {
    let mut save_path: PathBuf;
    let cartridge: Box<[u8]> = loop {
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
            match md_cartridge::read(
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