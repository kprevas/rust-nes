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

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::ArgMatches;

pub mod nes;
pub mod control;
pub mod input;
pub mod gen;
pub mod menu;
pub mod record;

pub fn run(matches: ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("disassemble") {
        let output_path = matches.value_of("OUTPUT");
        let mut out = match output_path {
            Some(ref path) => Box::new(File::create(&Path::new(path)).unwrap()) as Box<dyn Write>,
            None => Box::new(std::io::stdout()) as Box<dyn Write>,
        };
        nes::disasssemble(matches, &mut out);
    } else if let Some(matches) = matches.subcommand_matches("run") {
        nes::run(matches);
    } else if let Some(matches) = matches.subcommand_matches("gen") {
        gen::run(matches);
    }
}