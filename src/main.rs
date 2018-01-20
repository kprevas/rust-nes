#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
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

mod cpu;
mod cartridge;
mod input;
mod ppu;
mod test;

const CPU_PER_PPU: f32 = 3.0;

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
            (@arg INPUT: "the input file to use")
            (@arg instrument_cpu: -c "instruments CPU")
            (@arg instrument_ppu: -p "instruments PPU")
            (@arg time_frame: -t "logs frame timing")
            (@arg step: -s "frame-by-frames step with spacebar")
            (@arg dump_vram: -v "dumps vram")
        )
    ).get_matches();

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
        let mut window: PistonWindow = WindowSettings::new(
            "nes",
            [293, 240],
        )
            .exit_on_esc(true)
            .build()
            .unwrap();

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
        let step = matches.is_present("step");
        let dump_vram = matches.is_present("dump_vram");

        let mut inputs: input::ControllerState = Default::default();

        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("src").unwrap();
        let ref font = assets.join("VeraMono.ttf");
        let factory = window.factory.clone();
        let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();

        let ppu_bus = RefCell::new(ppu::bus::PpuBus::new());

        let mut ppu = ppu::Ppu::new(&mut cartridge.ppu_bus, &ppu_bus, Some(&mut window));

        let mut cpu = cpu::Cpu::boot(&mut cartridge.cpu_bus, &ppu_bus);

        let mut settings = EventSettings::new();
        settings.ups = 60;
        settings.ups_reset = 0;
        let mut cpu_dots = 0f32;
        while let Some(e) = window.next() {
            if let Some(Button::Keyboard(key)) = e.press_args() {
                if step && key == Key::Space {
                    do_frame(&mut window, &mut cpu, &mut ppu, &mut cpu_dots, inputs, instrument_cpu, instrument_ppu, time_frame)
                }
                match key {
                    Key::A => inputs.b = true,
                    Key::S => inputs.a = true,
                    Key::Tab => inputs.select = true,
                    Key::Return => inputs.start = true,
                    Key::Up => inputs.up = true,
                    Key::Down => inputs.down = true,
                    Key::Left => inputs.left = true,
                    Key::Right => inputs.right = true,
                    _ => (),
                }
            }

            if let Some(Button::Keyboard(key)) = e.release_args() {
                match key {
                    Key::A => inputs.b = false,
                    Key::S => inputs.a = false,
                    Key::Tab => inputs.select = false,
                    Key::Return => inputs.start = false,
                    Key::Up => inputs.up = false,
                    Key::Down => inputs.down = false,
                    Key::Left => inputs.left = false,
                    Key::Right => inputs.right = false,
                    _ => (),
                }
            }

            if let Some(_u) = e.update_args() {
                if !step {
                    do_frame(&mut window, &mut cpu, &mut ppu, &mut cpu_dots, inputs, instrument_cpu, instrument_ppu, time_frame)
                }
            }

            if let Some(_r) = e.render_args() {
                window.draw_2d(&e, |c, gl| {
                    ppu.render(c, gl);
                    if dump_vram {
                        ppu.dump_ram(c, gl, &mut glyphs);
                    }
                });
            }
        }
    }
}

fn do_frame(window: &mut PistonWindow,
            cpu: &mut cpu::Cpu,
            ppu: &mut ppu::Ppu,
            cpu_dots: &mut f32,
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
        } else {
            *cpu_dots -= 1.0;
        }
        ppu.tick(instrument_ppu, Some(&mut window.encoder));
    }
    if time_frame {
        debug!("frame took {}", start_time.to(time::PreciseTime::now()));
    }
}
