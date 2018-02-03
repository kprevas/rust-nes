#[macro_use]
extern crate log;
extern crate clap;
extern crate piston_window;
extern crate image;
extern crate time;
extern crate find_folder;
extern crate hex_slice;
extern crate nfd;
extern crate simple_error;

use std::cell::RefCell;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

use piston_window::*;

use nfd::Response;

pub mod apu;
pub mod cpu;
pub mod cartridge;
pub mod input;
pub mod ppu;

const CPU_PER_PPU: f32 = 3.0;
const APU_PER_PPU: f32 = CPU_PER_PPU * 2.0;

pub fn run(matches: clap::ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("disassemble") {
        let input_file = matches.value_of("INPUT").unwrap();
        let output_path = matches.value_of("OUTPUT");
        let mut out = match output_path {
            Some(ref path) => Box::new(File::create(&Path::new(path)).unwrap()) as Box<Write>,
            None => Box::new(std::io::stdout()) as Box<Write>,
        };
        let cartridge = cartridge::read(File::open(input_file).as_mut().unwrap()).unwrap();
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

        let mut cartridge: cartridge::Cartridge = loop {
            let input_file = match matches.value_of("INPUT") {
                Some(i) => PathBuf::from(i),
                None => {
                    match nfd::open_file_dialog(None, None).unwrap() {
                        Response::Okay(p) => PathBuf::from(p),
                        Response::OkayMultiple(v) => PathBuf::from(&v[0]),
                        Response::Cancel => return,
                    }
                }
            };
            match cartridge::read(File::open(input_file).as_mut().unwrap()) {
                Ok(c) => break c,
                Err(e) => if matches.is_present("INPUT") {
                    panic!(e);
                },
            };
        };

        let instrument_cpu = matches.is_present("instrument_cpu");
        let instrument_ppu = matches.is_present("instrument_ppu");
        let time_frame = matches.is_present("time_frame");
        let dump_vram = matches.is_present("dump_vram");

        let mut inputs: input::ControllerState = Default::default();
        let mut reset = false;

        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("src").unwrap();
        let ref font = assets.join("VeraMono.ttf");
        let factory = window.factory.clone();
        let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();

        let ppu_bus = RefCell::new(ppu::bus::PpuBus::new());
        let apu_bus = RefCell::new(apu::bus::ApuBus::new());

        let mut ppu = ppu::Ppu::new(&mut cartridge.ppu_bus, &ppu_bus, Some(&mut window));

        let mut cpu = cpu::Cpu::boot(&mut cartridge.cpu_bus, &ppu_bus, &apu_bus);

        let mut apu = apu::Apu::new(&apu_bus).unwrap();

        let mut cpu_dots = 0.0;
        let mut apu_dots = 0.0;

        while let Some(e) = window.next() {
            inputs.event(&e, &mut reset);

            if let Some(_u) = e.update_args() {
                if reset {
                    reset = false;
                    cpu.reset();
                }
                do_frame(&mut window, &mut cpu, &mut ppu, &mut apu, &mut cpu_dots, &mut apu_dots, inputs, instrument_cpu, instrument_ppu, time_frame);
            }

            if let Some(_r) = e.render_args() {
                window.draw_2d(&e, |c, gl| {
                    ppu.render(c, gl);
                    if dump_vram {
                        ppu.dump_ram(c, gl, &mut glyphs);
                    }
                });
            }

            if let Some(_c) = e.close_args() {
                apu.close().unwrap();
            }
        }
    }
}

fn do_frame(window: &mut PistonWindow,
            cpu: &mut cpu::Cpu,
            ppu: &mut ppu::Ppu,
            apu: &mut apu::Apu,
            cpu_dots: &mut f32,
            apu_dots: &mut f32,
            inputs: input::ControllerState,
            instrument_cpu: bool,
            instrument_ppu: bool,
            time_frame: bool) -> () {
    let start_time = time::PreciseTime::now();
    let dots = ppu.dots_per_frame();
    for _ in 0..dots {
        if *cpu_dots <= 0.0 {
            cpu.tick(instrument_cpu, inputs);
            *cpu_dots += CPU_PER_PPU;
        }
        *cpu_dots -= 1.0;
        if *apu_dots <= 0.0 {
            apu.tick(&mut |addr| { cpu.read_memory(addr) });
            *apu_dots += APU_PER_PPU;
        }
        *apu_dots -= 1.0;
        ppu.tick(instrument_ppu, Some(&mut window.encoder));
    }
    if time_frame {
        debug!(target: "timing", "frame took {}", start_time.to(time::PreciseTime::now()));
    }
}
