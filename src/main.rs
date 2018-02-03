extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate nes;

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
        )
    ).get_matches();
    nes::run(matches);
}

