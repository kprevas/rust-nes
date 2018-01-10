#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate clap;

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::BufWriter;

mod cpu;
mod cartridge;

fn main() {
    env_logger::init().unwrap();

    let matches = clap_app!(myapp =>
        (@subcommand disassemble =>
            (about: "disassemble a .nes file")
            (@arg INPUT: +required "the input file to use")
            (@arg OUTPUT: "the output file (stdout if not provided)")
        )
    ).get_matches();

    if let Some(matches) = matches.subcommand_matches("disassemble") {
        let input_file = matches.value_of("INPUT").unwrap();
        let output_path = matches.value_of("OUTPUT");
        let mut out = match output_path {
            Some(ref path) => Box::new(File::create(&Path::new(path)).unwrap()) as Box<Write>,
            None => Box::new(std::io::stdout()) as Box<Write>,
        };
        let cartridge = cartridge::read(File::open(input_file).as_mut().unwrap());
        cpu::disassembler::disassemble(cartridge, 0xc000, &mut out).unwrap();
    }
}
