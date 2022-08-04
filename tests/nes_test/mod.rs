extern crate env_logger;
extern crate piston_window;

use std::cell::RefCell;
use std::io::Read;

use emu::input::{player_1_nes, player_2_nes};
use emu::nes::apu::*;
use emu::nes::apu::bus::*;
use emu::nes::cartridge;
use emu::nes::cpu::*;
use emu::nes::ppu::*;
use emu::nes::ppu::bus::*;
use emu::window::Cpu;

use self::piston_window::*;

pub fn run_test_to_pc(
    rom: &mut dyn Read,
    pc_start: Option<u16>,
    pc_end: u16,
    assert: &[(u16, u8)],
) {
    run_test(
        rom,
        pc_start,
        &mut |cpu| cpu.pc_for_test() == pc_end,
        None,
        &mut |cpu| {
            for &(addr, val) in assert {
                assert_eq!(
                    val,
                    cpu.read_memory_no_tick(addr),
                    "0x{:02X}",
                    cpu.read_memory_no_tick(addr)
                );
            }
        },
    );
}

pub fn run_test_to_pc_and_check_accumulator(
    rom: &mut dyn Read,
    pc_start: Option<u16>,
    pc_end: u16,
    expected_accumulator: u8,
) {
    run_test(
        rom,
        pc_start,
        &mut |cpu| cpu.pc_for_test() == pc_end,
        None,
        &mut |cpu| {
            assert_eq!(
                expected_accumulator,
                cpu.a_for_test(),
                "0x{:02X}",
                cpu.a_for_test()
            );
        },
    );
}

pub fn run_test_to_success_or_fail_pc(
    rom: &mut dyn Read,
    pc_start: Option<u16>,
    pc_success: u16,
    pc_fail: u16,
    error_code_addr: u16,
) {
    run_test(
        rom,
        pc_start,
        &mut |cpu| cpu.pc_for_test() == pc_success || cpu.pc_for_test() == pc_fail,
        None,
        &mut |cpu| {
            if cpu.pc_for_test() == pc_fail {
                assert_eq!(0, cpu.read_memory_no_tick(error_code_addr));
            }
        },
    );
}

pub fn run_test_until_memory_matches(
    rom: &mut dyn Read,
    valid_signal_addr: u16,
    valid_signal_val: &[u8],
    status_addr: u16,
    running_status: u8,
    reset_status: u8,
    assert: &[(u16, u8)],
) {
    run_test(
        rom,
        None,
        &mut |cpu| {
            let mut output = valid_signal_addr;
            for val in valid_signal_val {
                if *val != cpu.read_memory_no_tick(output) {
                    return false;
                }
                output += 1;
            }
            let status = cpu.read_memory_no_tick(status_addr);
            status != running_status && status != reset_status
        },
        Some((status_addr, running_status, reset_status)),
        &mut |cpu| {
            for &(addr, val) in assert {
                assert_eq!(
                    val,
                    cpu.read_memory_no_tick(addr),
                    "0x{:02X}",
                    cpu.read_memory_no_tick(addr)
                );
            }
        },
    );
}

fn run_test(
    rom: &mut dyn Read,
    pc_start: Option<u16>,
    terminate_condition: &mut dyn FnMut(&mut Cpu) -> bool,
    status: Option<(u16, u8, u8)>,
    assert: &mut dyn FnMut(&mut Cpu),
) {
    let _ = env_logger::try_init();
    let ppu_bus = RefCell::new(PpuBus::new());
    let apu_bus = RefCell::new(ApuBus::new());
    let mut cartridge = cartridge::read(rom, None).unwrap();
    let ppu = Ppu::new::<NoWindow>(&mut cartridge.ppu_bus, &ppu_bus, None, true);
    let apu = Apu::new(&apu_bus, None).unwrap();
    let mut cpu = Cpu::boot(&mut cartridge.cpu_bus, ppu, &ppu_bus, apu, &apu_bus, true);
    let inputs = [player_1_nes(), player_2_nes()];

    if let Some(pc_start) = pc_start {
        cpu.setup_for_test(0x24, pc_start);
    }

    let mut reset_delay = 0;
    let mut did_reset = false;
    while !terminate_condition(&mut cpu) {
        cpu.next_operation(&inputs);
        if let Some((addr, running, reset)) = status {
            if reset_delay > 0 {
                reset_delay -= 1;
                if reset_delay == 0 {
                    cpu.reset(true);
                }
            } else if cpu.read_memory_no_tick(addr) == running && did_reset {
                did_reset = false;
            } else if cpu.read_memory_no_tick(addr) == reset && !did_reset {
                did_reset = true;
                reset_delay = 20_000;
            }
        }
    }

    assert(&mut cpu);
}
