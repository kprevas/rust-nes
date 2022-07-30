extern crate emu;

use test::run_test_to_pc;
use test::run_test_to_pc_and_check_accumulator;
use test::run_test_to_success_or_fail_pc;
use test::run_test_until_memory_matches;

mod test;

#[test]
fn nes_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/nestest/nestest.nes").as_ref(),
        Some(0xc000),
        0xc66e,
        &[(0x02, 0), (0x03, 0)],
    );
}

#[test]
fn test_branch_timing_1_branch_basics() {
    run_test_to_pc(
        &mut include_bytes!("roms/branch_timing_tests/1.Branch_Basics.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_branch_timing_2_backward_branch() {
    run_test_to_pc(
        &mut include_bytes!("roms/branch_timing_tests/2.Backward_Branch.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_branch_timing_3_forward_branch() {
    run_test_to_pc(
        &mut include_bytes!("roms/branch_timing_tests/3.Forward_Branch.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_dummy_reads() {
    run_test_to_success_or_fail_pc(
        &mut include_bytes!("roms/cpu_dummy_reads/cpu_dummy_reads.nes").as_ref(),
        None,
        0xe36d,
        0xe372,
        0x16,
    );
}

#[test]
fn test_cpu_exec_space_apu() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_exec_space/test_cpu_exec_space_apu.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_cpu_exec_space_ppuio() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_exec_space/test_cpu_exec_space_ppuio.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_interrupts_1_cli_latency() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_interrupts/1-cli_latency.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_interrupts_2_nmi_and_brk() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_interrupts/2-nmi_and_brk.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_interrupts_3_nmi_and_irq() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_interrupts/3-nmi_and_irq.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_interrupts_4_irq_and_dma() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_interrupts/4-irq_and_dma.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_interrupts_5_branch_delays_irq() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_interrupts/5-branch_delays_irq.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_01_implied() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/01-implied.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_02_immediate() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/02-immediate.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_03_zero_page() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/03-zero_page.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_04_zp_xy() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/04-zp_xy.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_05_absolute() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/05-absolute.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_06_abs_xy() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/06-abs_xy.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_07_ind_x() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/07-ind_x.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_08_ind_y() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/08-ind_y.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_09_branches() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/09-branches.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_10_stack() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/10-stack.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_11_jmp_jsr() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/11-jmp_jsr.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_12_rts() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/12-rts.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_13_rti() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/13-rti.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_14_brk() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/14-brk.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_15_special() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_test-v3/15-special.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_cpu_timing() {
    run_test_to_pc_and_check_accumulator(
        &mut include_bytes!("roms/cpu_timing_test6/cpu_timing_test.nes").as_ref(),
        None,
        0xe970,
        0x02,
    )
}

#[test]
fn test_reset_ram_after_reset() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_reset/ram_after_reset.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_reset_registers() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/cpu_reset/registers.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_timing/1-instr_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_branch_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_timing/2-branch_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_misc_abs_x_wrap() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_misc/01-abs_x_wrap.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_misc_branch_wrap() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_misc/02-branch_wrap.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_misc_dummy_reads() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_misc/03-dummy_reads.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_instr_misc_dummy_reads_apu() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/instr_misc/04-dummy_reads_apu.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}
