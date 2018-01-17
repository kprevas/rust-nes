#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate piston_window;
extern crate image;
extern crate time;

use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use piston_window::*;

mod cpu;
mod cartridge;
mod ppu;

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
            (@arg INPUT: +required "the input file to use")
            (@arg instrument_cpu: -c "instruments CPU")
            (@arg instrument_ppu: -p "instruments PPU")
            (@arg time_frame: -t "logs frame timing")
            (@arg step: -s "frame-by-frames step with spacebar")
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
        let instrument_cpu = matches.is_present("instrument_cpu");
        let instrument_ppu = matches.is_present("instrument_ppu");
        let time_frame = matches.is_present("time_frame");
        let step = matches.is_present("step");
        let mut cartridge = cartridge::read(File::open(input_file).as_mut().unwrap());

        let mut window: PistonWindow = WindowSettings::new(
            "nes",
            [256, 240],
        )
            .exit_on_esc(true)
            .build()
            .unwrap();

        let ppu_bus = RefCell::new(ppu::bus::PpuBus::new());

        let mut ppu = ppu::Ppu::new(&mut cartridge.ppu_bus, &ppu_bus, &mut window);

        let mut cpu = cpu::Cpu::boot(&mut cartridge.cpu_bus, &ppu_bus);

        let mut settings = EventSettings::new();
        settings.ups = 60;
        settings.ups_reset = 0;
        let mut cpu_dots = 0f32;
        while let Some(e) = window.next() {
            if let Some(Button::Keyboard(key)) = e.press_args() {
                if step && key == Key::Space {
                    do_frame(&mut window, &mut cpu, &mut ppu, &mut cpu_dots, instrument_cpu, instrument_ppu, time_frame)
                }
            }

            if let Some(u) = e.update_args() {
                if !step {
                    do_frame(&mut window, &mut cpu, &mut ppu, &mut cpu_dots, instrument_cpu, instrument_ppu, time_frame)
                }
            }

            if let Some(r) = e.render_args() {
                window.draw_2d(&e, |c, gl| ppu.render(c, gl));
            }
        }
    }
}

fn do_frame(window: &mut PistonWindow,
            cpu: &mut cpu::Cpu,
            ppu: &mut ppu::Ppu,
            cpu_dots: &mut f32,
            instrument_cpu: bool,
            instrument_ppu: bool,
            time_frame: bool) -> () {
    let start_time = time::PreciseTime::now();
    let dots = ppu.dots_per_frame();
    for _ in 0..dots {
        if *cpu_dots <= 0.0 {
            cpu.tick(instrument_cpu);
            *cpu_dots += CPU_PER_PPU;
        } else {
            *cpu_dots -= 1.0;
        }
        ppu.tick(instrument_ppu, &mut window.encoder);
    }
    if time_frame {
        debug!("frame took {}", start_time.to(time::PreciseTime::now()));
    }
}
