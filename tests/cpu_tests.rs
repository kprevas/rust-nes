extern crate nes;

use test::run_test_to_pc;
use test::run_test_to_success_or_fail_pc;
use test::run_test_until_memory_matches;

mod test;

#[test]
fn nes_test() {
    run_test_to_pc(&mut include_bytes!("roms/nestest/nestest.nes").as_ref(),
                   Some(0xc000), 0xc66e, &[(0x02, 0), (0x03, 0)]);
}

#[test]
fn test_branch_timing_1_branch_basics() {
    run_test_to_pc(&mut include_bytes!("roms/branch_timing_tests/1.Branch_Basics.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_branch_timing_2_backward_branch() {
    run_test_to_pc(&mut include_bytes!("roms/branch_timing_tests/2.Backward_Branch.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_branch_timing_3_forward_branch() {
    run_test_to_pc(&mut include_bytes!("roms/branch_timing_tests/3.Forward_Branch.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_dummy_reads() {
    run_test_to_success_or_fail_pc(&mut include_bytes!("roms/cpu_dummy_reads/cpu_dummy_reads.nes").as_ref(),
                                   None, 0xe36d, 0xe372, 0x16);
}

#[test]
fn test_cpu_exec_space_apu() {
    run_test_until_memory_matches(&mut include_bytes!("roms/cpu_exec_space/test_cpu_exec_space_apu.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn test_cpu_exec_space_ppuio() {
    run_test_until_memory_matches(&mut include_bytes!("roms/cpu_exec_space/test_cpu_exec_space_ppuio.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}
