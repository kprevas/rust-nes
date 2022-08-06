#![feature(mixed_integer_ops)]

extern crate bincode;
extern crate bytes;
extern crate clap;
extern crate core;
extern crate dasp;
extern crate find_folder;
extern crate gfx_device_gl;
extern crate graphics;
extern crate hex_slice;
extern crate image;
#[macro_use]
extern crate log;
extern crate num_integer;
extern crate num_traits;
extern crate piston_window;
extern crate portaudio;
extern crate rfd;
extern crate sdl2_window;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate simple_error;
extern crate triple_buffer;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use piston_window::{PistonWindow, WindowSettings};

use rom::Rom;

pub mod control;
pub mod gen;
pub mod input;
pub mod menu;
pub mod nes;
pub mod record;
pub mod rom;
pub mod window;

pub fn run(matches: ArgMatches) {
    let window: PistonWindow<sdl2_window::Sdl2Window> =
        WindowSettings::new("emu", [300, 300]).build().unwrap();

    let mut save_path = None;
    let rom: Option<Rom> = loop {
        let input_file = match matches.value_of("INPUT") {
            Some(i) => Some(PathBuf::from(i)),
            None => rfd::FileDialog::new().pick_file(),
        };
        if let Some(input_file) = input_file {
            save_path = Some(
                PathBuf::from(".")
                    .join(input_file.file_name().unwrap())
                    .with_extension("sav"),
            );
            let nes = nes::load_cartridge(
                File::open(&input_file).as_mut().unwrap(),
                match File::open(save_path.as_ref().unwrap().as_path()) {
                    Ok(ref mut file) => Some(file),
                    Err(_) => None,
                },
            );
            if let Ok(cartridge) = nes {
                break Some(Rom::Nes(cartridge));
            };
            let gen = gen::load_cartridge(
                File::open(&input_file).as_mut().unwrap(),
                match File::open(save_path.as_ref().unwrap().as_path()) {
                    Ok(ref mut file) => Some(file),
                    Err(_) => None,
                },
            );
            if let Ok(cartridge) = gen {
                break Some(Rom::Genesis(cartridge));
            };
            if matches.is_present("INPUT") {
                break None;
            }
        } else {
            break None;
        }
    };
    let rom = match rom {
        None => panic!("Couldn't load ROM"),
        Some(rom) => rom,
    };
    let save_path = match save_path {
        None => panic!("Couldn't create save data"),
        Some(save_path) => save_path,
    };

    if let Some(matches) = matches.subcommand_matches("disassemble") {
        let output_path = matches.value_of("OUTPUT");
        let mut out = match output_path {
            Some(ref path) => Box::new(File::create(&Path::new(path)).unwrap()) as Box<dyn Write>,
            None => Box::new(std::io::stdout()) as Box<dyn Write>,
        };
        match rom {
            Rom::Nes(cartridge) => nes::disassemble(cartridge, &mut out).unwrap(),
            Rom::Genesis(cartridge) => gen::disassemble(cartridge, &mut out).unwrap(),
        }
    } else if let Some(matches) = matches.subcommand_matches("run") {
        match rom {
            Rom::Nes(cartridge) => nes::run(matches, cartridge, save_path, window),
            Rom::Genesis(cartridge) => gen::run(matches, cartridge, save_path, window),
        }
    }
}
