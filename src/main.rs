extern crate clap;
extern crate emu;
extern crate env_logger;

fn main() {
    env_logger::try_init().unwrap();
    emu::run();
}
