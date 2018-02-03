extern crate env_logger;

use std::cell::RefCell;
use std::io::Read;

use nes::cartridge;
use nes::cpu::*;
use nes::ppu::*;
use nes::apu::*;
use nes::ppu::bus::*;
use nes::apu::bus::*;
use nes::input::ControllerState;

pub fn run_test(rom: &mut Read, pc_start: Option<u16>, pc_end: u16, assert: &[(u16, u8)]) {
    let _ = env_logger::init();
    let ppu_bus = RefCell::new(PpuBus::new());
    let apu_bus = RefCell::new(ApuBus::new());
    let mut cartridge = cartridge::read(rom).unwrap();
    let ppu = Ppu::new(&mut cartridge.ppu_bus, &ppu_bus, None, true);
    let apu = Apu::new(&apu_bus).unwrap();
    let mut cpu = Cpu::boot(&mut cartridge.cpu_bus, ppu, &ppu_bus, apu, &apu_bus, true);
    let inputs = ControllerState::default();

    if let Some(pc_start) = pc_start {
        cpu.setup_for_test(0x24, pc_start);
    }

    while cpu.pc_for_test() != pc_end {
        cpu.next_operation(inputs);
    }

    for &(addr, val) in assert {
        assert_eq!(val, cpu.read_memory(addr), "0x{:02X}", cpu.read_memory(addr));
    }
}