use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
    pub input: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    // disassemble a ROM file
    Disassemble {
        // the output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    // load and run a ROM
    Run {
        // instruments CPU
        #[arg(short = 'c')]
        instrument_cpu: bool,
        // instruments NES PPU
        #[arg(long = "ppu")]
        instrument_ppu: bool,
        // runs in benchmark mode
        #[arg(short = 'b')]
        bench_mode: bool,
        // displays VRAM dump
        #[arg(short = 'v')]
        dump_vram: bool,
        // starts paused
        #[arg(short = 'p')]
        pause: bool,
    },
}
