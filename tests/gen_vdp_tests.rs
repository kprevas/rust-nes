extern crate emu;
extern crate piston;

use std::cell::RefCell;
use std::fs::File;
use std::path::Path;

use piston::NoWindow;

use emu::gen;
use emu::gen::{m68k, vdp};
use emu::window::Cpu;

#[test]
fn test_fifo_buffer_size() {
    run_vdp_test(0x30C);
}

#[test]
fn test_separate_fifo_read_write_buffer() {
    run_vdp_test(0x316);
}

#[test]
fn test_dma_transfer_using_fifo() {
    run_vdp_test(0x320);
}

#[test]
fn test_dma_fill_fifo_usage() {
    run_vdp_test(0x32A);
}

#[test]
#[ignore]
fn test_fifo_write_to_invalid_target() {
    run_vdp_test(0x334);
}

#[test]
#[ignore]
fn test_read_target_01100() {
    run_vdp_test(0x33E);
}

#[test]
fn test_vram_byteswapping() {
    run_vdp_test(0x348);
}

#[test]
fn test_cram_byteswapping() {
    run_vdp_test(0x352);
}

#[test]
fn test_vsram_byteswapping() {
    run_vdp_test(0x35C);
}

#[test]
#[ignore]
fn test_partial_cp_writes() {
    run_vdp_test(0x36E);
}

#[test]
fn test_register_write_bit_13_masked() {
    run_vdp_test(0x378);
}

#[test]
#[ignore]
fn test_mode_4_register_masked() {
    run_vdp_test(0x382);
}

#[test]
#[ignore]
fn test_register_writes_affect_code_register() {
    run_vdp_test(0x38C);
}

#[test]
fn test_cp_write_pending_reset() {
    run_vdp_test(0x396);
}

#[test]
fn test_read_target_switching() {
    run_vdp_test(0x3A0);
}

#[test]
#[ignore]
fn test_fifo_wait_states() {
    run_vdp_test(0x3AA);
}

#[test]
#[ignore]
fn test_hv_counter_latch() {
    run_vdp_test(0x3BC);
}

#[test]
fn test_blanking_flags() {
    run_vdp_test(0x3C6);
}

#[test]
#[ignore]
fn test_dma_transfer_bus_lock() {
    run_vdp_test(0x3D0);
}

#[test]
fn test_dma_transfer_source_address_wrapping() {
    run_vdp_test(0x3E2);
}

#[test]
fn test_dma_transfer_to_vram_wrapping() {
    run_vdp_test(0x3EC);
}

#[test]
fn test_dma_transfer_to_cram_wrapping() {
    run_vdp_test(0x3FA);
}

#[test]
fn test_dma_transfer_to_vsram_wrapping() {
    run_vdp_test(0x404);
}

#[test]
#[ignore]
fn test_dma_transfer_length_reg_update() {
    run_vdp_test(0x40E);
}

#[test]
#[ignore]
fn test_dma_fill_length_reg_update() {
    run_vdp_test(0x418);
}

#[test]
#[ignore]
fn test_dma_copy_length_reg_update() {
    run_vdp_test(0x422);
}

#[test]
#[ignore]
fn test_dma_transfer_source_reg_update() {
    run_vdp_test(0x42C);
}

#[test]
fn test_dma_fill_source_reg_update() {
    run_vdp_test(0x436);
}

#[test]
#[ignore]
fn test_dma_copy_source_reg_update() {
    run_vdp_test(0x440);
}

#[test]
#[ignore]
fn test_fifo_full_dma_transfer() {
    run_vdp_test(0x452);
}

#[test]
#[ignore]
fn test_dma_fill_data_port_writes_vram() {
    run_vdp_test(0x45C);
}

#[test]
#[ignore]
fn test_dma_fill_data_port_writes_cram() {
    run_vdp_test(0x466);
}

#[test]
#[ignore]
fn test_dma_fill_data_port_writes_vsram() {
    run_vdp_test(0x470);
}

#[test]
#[ignore]
fn test_dma_fill_control_port_writes() {
    run_vdp_test(0x47A);
}

#[test]
#[ignore]
fn test_dma_busy_flag_dma_transfer() {
    run_vdp_test(0x490);
}

#[test]
#[ignore]
fn test_dma_busy_flag_dma_fill() {
    run_vdp_test(0x49A);
}

#[test]
#[ignore]
fn test_dma_busy_flag_dma_copy() {
    run_vdp_test(0x4A4);
}

#[test]
#[ignore]
fn test_dma_busy_flag_dma_enable_toggle_fill() {
    run_vdp_test(0x4AE);
}

#[test]
#[ignore]
fn test_dma_busy_flag_dma_enable_toggle_copy() {
    run_vdp_test(0x4B8);
}

#[test]
#[ignore]
fn test_dma_busy_flag_dma_disabled_fill() {
    run_vdp_test(0x4C2);
}

#[test]
#[ignore]
fn test_dma_busy_flag_dma_disabled_copy() {
    run_vdp_test(0x4CC);
}

#[test]
fn test_dma_transfer_to_vram_inc_0() {
    run_vdp_test(0x4DE);
}

#[test]
fn test_dma_transfer_to_cram_inc_0() {
    run_vdp_test(0x4F4);
}

#[test]
fn test_dma_transfer_to_vsram_inc_0() {
    run_vdp_test(0x50A);
}

#[test]
fn test_dma_transfer_to_vram_inc_1() {
    run_vdp_test(0x520);
}

#[test]
fn test_dma_transfer_to_cram_inc_1() {
    run_vdp_test(0x536);
}

#[test]
fn test_dma_transfer_to_vsram_inc_1() {
    run_vdp_test(0x54C);
}

#[test]
fn test_dma_transfer_to_vram_inc_2() {
    run_vdp_test(0x562);
}

#[test]
fn test_dma_transfer_to_cram_inc_2() {
    run_vdp_test(0x578);
}

#[test]
fn test_dma_transfer_to_vsram_inc_2() {
    run_vdp_test(0x58E);
}

#[test]
fn test_dma_transfer_to_vram_inc_3() {
    run_vdp_test(0x5AC);
}

#[test]
fn test_dma_transfer_to_cram_inc_3() {
    run_vdp_test(0x5C2);
}

#[test]
fn test_dma_transfer_to_vsram_inc_3() {
    run_vdp_test(0x5D8);
}

#[test]
fn test_dma_transfer_to_vram_inc_4() {
    run_vdp_test(0x5EE);
}

#[test]
fn test_dma_transfer_to_cram_inc_4() {
    run_vdp_test(0x604);
}

#[test]
fn test_dma_transfer_to_vsram_inc_4() {
    run_vdp_test(0x61A);
}

#[test]
fn test_dma_transfer_to_vram_cd4_1_inc_0() {
    run_vdp_test(0x638);
}

#[test]
fn test_dma_transfer_to_cram_cd4_1_inc_0() {
    run_vdp_test(0x64E);
}

#[test]
fn test_dma_transfer_to_vsram_cd4_1_inc_0() {
    run_vdp_test(0x664);
}

#[test]
fn test_dma_transfer_to_vram_cd4_1_inc_1() {
    run_vdp_test(0x67A);
}

#[test]
fn test_dma_transfer_to_cram_cd4_1_inc_1() {
    run_vdp_test(0x690);
}

#[test]
fn test_dma_transfer_to_vsram_cd4_1_inc_1() {
    run_vdp_test(0x6A6);
}

#[test]
fn test_dma_transfer_to_vram_cd4_1_inc_2() {
    run_vdp_test(0x6BC);
}

#[test]
fn test_dma_transfer_to_cram_cd4_1_inc_2() {
    run_vdp_test(0x6D2);
}

#[test]
fn test_dma_transfer_to_vsram_cd4_1_inc_2() {
    run_vdp_test(0x6E8);
}

#[test]
fn test_dma_transfer_to_vram_cd4_1_inc_3() {
    run_vdp_test(0x706);
}

#[test]
fn test_dma_transfer_to_cram_cd4_1_inc_3() {
    run_vdp_test(0x71C);
}

#[test]
fn test_dma_transfer_to_vsram_cd4_1_inc_3() {
    run_vdp_test(0x732);
}

#[test]
fn test_dma_transfer_to_vram_cd4_1_inc_4() {
    run_vdp_test(0x748);
}

#[test]
fn test_dma_transfer_to_cram_cd4_1_inc_4() {
    run_vdp_test(0x75E);
}

#[test]
fn test_dma_transfer_to_vsram_cd4_1_inc_4() {
    run_vdp_test(0x774);
}

#[test]
fn test_dma_fill_to_vram_inc_0() {
    run_vdp_test(0x0792)
}

#[test]
fn test_dma_fill_to_cram_inc_0() {
    run_vdp_test(0x07A8)
}

#[test]
fn test_dma_fill_to_vsram_inc_0() {
    run_vdp_test(0x07BE)
}

#[test]
fn test_dma_fill_to_vram_inc_1() {
    run_vdp_test(0x07DC)
}

#[test]
fn test_dma_fill_to_cram_inc_1() {
    run_vdp_test(0x07F2)
}

#[test]
fn test_dma_fill_to_vsram_inc_1() {
    run_vdp_test(0x0808)
}

#[test]
fn test_dma_fill_to_vram_inc_2() {
    run_vdp_test(0x0826)
}

#[test]
fn test_dma_fill_to_cram_inc_2() {
    run_vdp_test(0x083C)
}

#[test]
fn test_dma_fill_to_vsram_inc_2() {
    run_vdp_test(0x0852)
}

#[test]
fn test_dma_fill_to_vram_inc_4() {
    run_vdp_test(0x0870)
}

#[test]
fn test_dma_fill_to_cram_inc_4() {
    run_vdp_test(0x0886)
}

#[test]
fn test_dma_fill_to_vsram_inc_4() {
    run_vdp_test(0x089C)
}

#[test]
fn test_dma_fill_to_vram_cd4_1_inc_0() {
    run_vdp_test(0x08BA)
}

#[test]
fn test_dma_fill_to_cram_cd4_1_inc_0() {
    run_vdp_test(0x08D0)
}

#[test]
fn test_dma_fill_to_vsram_cd4_1_inc_0() {
    run_vdp_test(0x08E6)
}

#[test]
fn test_dma_fill_to_vram_cd4_1_inc_1() {
    run_vdp_test(0x0904)
}

#[test]
fn test_dma_fill_to_cram_cd4_1_inc_1() {
    run_vdp_test(0x091A)
}

#[test]
fn test_dma_fill_to_vsram_cd4_1_inc_1() {
    run_vdp_test(0x0930)
}

#[test]
fn test_dma_fill_to_vram_cd4_1_inc_2() {
    run_vdp_test(0x094E)
}

#[test]
fn test_dma_fill_to_cram_cd4_1_inc_2() {
    run_vdp_test(0x0964)
}

#[test]
fn test_dma_fill_to_vsram_cd4_1_inc_2() {
    run_vdp_test(0x097A)
}

#[test]
fn test_dma_fill_to_vram_cd4_1_inc_4() {
    run_vdp_test(0x0998)
}

#[test]
fn test_dma_fill_to_cram_cd4_1_inc_4() {
    run_vdp_test(0x09AE)
}

#[test]
fn test_dma_fill_to_vsram_cd4_1_inc_4() {
    run_vdp_test(0x09C4)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_inc_0() {
    run_vdp_test(0x09E2)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_inc_1() {
    run_vdp_test(0x0A02)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_inc_2() {
    run_vdp_test(0x0A22)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_inc_4() {
    run_vdp_test(0x0A42)
}

#[test]
#[ignore]
fn test_dma_copy_8000_to_8002_for_0a() {
    run_vdp_test(0x0A62)
}

#[test]
#[ignore]
fn test_dma_copy_8000_to_8001_for_0a() {
    run_vdp_test(0x0A82)
}

#[test]
#[ignore]
fn test_dma_copy_8001_to_8003_for_0a() {
    run_vdp_test(0x0AA2)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_for_09() {
    run_vdp_test(0x0ACA)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8001_for_09() {
    run_vdp_test(0x0AEA)
}

#[test]
#[ignore]
fn test_dma_copy_9001_to_8000_for_09() {
    run_vdp_test(0x0B0A)
}

#[test]
#[ignore]
fn test_dma_copy_9001_to_8001_for_09() {
    run_vdp_test(0x0B2A)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_for_0a() {
    run_vdp_test(0x0B4A)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8001_for_0a() {
    run_vdp_test(0x0B6A)
}

#[test]
#[ignore]
fn test_dma_copy_9001_to_8000_for_0a() {
    run_vdp_test(0x0B8A)
}

#[test]
#[ignore]
fn test_dma_copy_9001_to_8001_for_0a() {
    run_vdp_test(0x0BAA)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_0000() {
    run_vdp_test(0x0BD2)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_0001() {
    run_vdp_test(0x0BF2)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_0011() {
    run_vdp_test(0x0C12)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_0100() {
    run_vdp_test(0x0C32)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_0101() {
    run_vdp_test(0x0C52)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_0111() {
    run_vdp_test(0x0C72)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_1000() {
    run_vdp_test(0x0C9A)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_1001() {
    run_vdp_test(0x0CBA)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_1011() {
    run_vdp_test(0x0CDA)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_1100() {
    run_vdp_test(0x0CFA)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_1101() {
    run_vdp_test(0x0D1A)
}

#[test]
#[ignore]
fn test_dma_copy_9000_to_8000_cd03_1111() {
    run_vdp_test(0x0D3A)
}

fn run_vdp_test(start_addr: u32) {
    let _ = env_logger::try_init();
    let cartridge = gen::load_cartridge(
        File::open(&Path::new("tests/gen_vdp/VDPFIFOTesting.bin"))
            .as_mut()
            .unwrap(),
        None,
    ).unwrap();
    let vdp_bus = RefCell::new(vdp::bus::VdpBus::new(false));
    let vdp = vdp::Vdp::new::<NoWindow>(&vdp_bus, None, false, false);
    let mut cpu = m68k::Cpu::boot(&cartridge, Some(vdp), &vdp_bus, false);

    cpu.reset(false);
    while cpu.pc_for_test() != 0x30C {
        cpu.next_operation(&[emu::input::player_1_gen(), emu::input::player_2_gen()]);
    }
    cpu.set_pc(start_addr);
    while cpu.pc_for_test() != 0xE4C {
        cpu.next_operation(&[emu::input::player_1_gen(), emu::input::player_2_gen()]);
    }
    if cpu.peek_ram(0xFFFF12) == 1 {
        let mut expected = vec![];
        let mut actual = vec![];
        for i in (cpu.a_for_test(2)..cpu.a_for_test(1)).step_by(2) {
            expected.push(format!("{:04X}", cpu.peek_ram(i)));
        }
        for i in (cpu.a_for_test(1)..cpu.a_for_test(0)).step_by(2) {
            actual.push(format!("{:04X}", cpu.peek_ram(i)));
        }
        assert_eq!(expected, actual);
    }
}
