#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate image;

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};

mod cpu;
mod cartridge;
mod ppu;

fn main() {
    env_logger::init().unwrap();

    let matches = clap_app!(myapp =>
        (@subcommand disassemble =>
            (about: "disassemble a .nes file")
            (@arg INPUT: +required "the input file to use")
            (@arg OUTPUT: "the output file (stdout if not provided)")
        )
        (@subcommand run =>
            (about: "run a .nes file")
            (@arg INPUT: +required "the input file to use")
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
        cpu::disassembler::disassemble(cartridge.cpu_bus, 0xc000, &mut out).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let input_file = matches.value_of("INPUT").unwrap();
        let mut cartridge = cartridge::read(File::open(input_file).as_mut().unwrap());

        let opengl = OpenGL::V3_2;

        let mut window: Window = WindowSettings::new(
            "nes",
            [256, 240],
        )
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
        let mut gl = GlGraphics::new(opengl);

        let mut ppu = ppu::Ppu::new(&mut cartridge.ppu_bus);

        let cpu = cpu::Cpu::boot(&mut cartridge.cpu_bus);

        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut window) {
            if let Some(r) = e.render_args() {
                ppu.render(&mut gl, r);
            }

            if let Some(u) = e.update_args() {}
        }
    }
}
