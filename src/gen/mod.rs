use std::cell::RefCell;
use std::fs::File;
use std::path::PathBuf;

use clap::ArgMatches;
use nfd::Response;
use piston_window::*;

use window::window_loop;

pub mod cartridge;
pub mod m68k;
pub mod vdp;

fn load_cartridge(matches: &ArgMatches) -> Option<(Box<[u8]>, PathBuf)> {
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

pub fn run(matches: &ArgMatches) {
    let window: PistonWindow<sdl2_window::Sdl2Window> =
        WindowSettings::new("gen", [320, 224]).build().unwrap();
    let mut window = window
        .ups(60)
        .ups_reset(0)
        .bench_mode(matches.is_present("bench_mode"));

    let cartridge;
    let save_path;
    if let Some((c, s)) = load_cartridge(matches) {
        cartridge = c;
        save_path = s;
    } else {
        return;
    }

    let mut inputs = [::input::player_1_gen(), ::input::player_2_gen()];
    let record_path = save_path.with_extension("rcd");

    let instrument_cpu = matches.is_present("instrument_cpu");

    let vdp_bus = RefCell::new(vdp::bus::VdpBus::new());

    let vdp = vdp::Vdp::new(&vdp_bus, Some(&mut window));
    let mut cpu = m68k::Cpu::boot(&cartridge, Some(vdp), &vdp_bus, instrument_cpu);

    window_loop(window, &mut inputs, &record_path, &mut cpu, 320.0, 224.0);

    cpu.close();
}
