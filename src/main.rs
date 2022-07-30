#[macro_use]
extern crate clap;
extern crate emu;
extern crate env_logger;

fn main() {
    env_logger::try_init().unwrap();

    let matches = clap_app!(myapp =>
        (@subcommand disassemble =>
            (about: "disassemble a .nes file")
            (@arg INPUT: "the input file to use")
            (@arg OUTPUT: -o "the output file (stdout if not provided)")
        )
        (@subcommand run =>
            (about: "run a .nes file")
            (@arg INPUT: "the input file to use")
            (@arg instrument_cpu: -c "instruments CPU")
            (@arg instrument_ppu: -p "instruments PPU")
            (@arg bench_mode: -b "runs in benchmark mode")
        )
        (@subcommand gen =>
            (about: "run a .md file")
            (@arg INPUT: "the input file to use")
            (@arg instrument_cpu: -c "instruments CPU")
            (@arg bench_mode: -b "runs in benchmark mode")
        )
    )
        .get_matches();
    emu::run(matches);
}
