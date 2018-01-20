extern crate env_logger;

use std::cell::RefCell;
use std::io::Read;

use nes::cartridge;
use nes::cpu::*;
use nes::ppu::*;
use nes::ppu::bus::*;
use nes::input::ControllerState;

const CPU_PER_PPU: f32 = 3.0;

pub fn run_test(rom: &mut Read, pc_start: Option<u16>, pc_end: u16, assert: &[(u16, u8)]) {
    let _ = env_logger::init();
    let ppu_bus = RefCell::new(PpuBus::new());
    let mut cartridge = cartridge::read(rom).unwrap();
    let mut ppu = Ppu::new(&mut cartridge.ppu_bus, &ppu_bus, None);
    let mut cpu = Cpu::boot(&mut cartridge.cpu_bus, &ppu_bus);
    let inputs = ControllerState::default();

    if let Some(pc_start) = pc_start {
        cpu.setup_for_test(0x24, pc_start);
    }

    let mut cpu_dots: f32 = 0.0;
    let mut cpu_cycles: u32 = 0;
    while cpu.pc_for_test() != pc_end && cpu_cycles < 900000 {
        if cpu_dots <= 0.0 {
            cpu.tick(true, inputs);
            cpu_dots += CPU_PER_PPU;
            cpu_cycles += 1;
        } else {
            cpu_dots -= 1.0;
        }
        ppu.tick(true, None);
    }

    assert_ne!(900000, cpu_cycles);
    for &(addr, val) in assert {
        assert_eq!(val, cpu.read_memory(addr), "0x{:02X}", cpu.read_memory(addr));
    }
}