extern crate nes;

use test::run_test_to_pc;

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
