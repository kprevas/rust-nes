extern crate nes;

use test::run_test_to_pc;

mod test;

#[test]
fn test_1_frame_basics_test() {
    run_test_to_pc(&mut include_bytes!("roms/vbl_nmi_timing/1.frame_basics.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_2_vbl_timing_test() {
    run_test_to_pc(&mut include_bytes!("roms/vbl_nmi_timing/2.vbl_timing.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_3_even_odd_frames_test() {
    run_test_to_pc(&mut include_bytes!("roms/vbl_nmi_timing/3.even_odd_frames.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_4_vbl_clear_timing_test() {
    run_test_to_pc(&mut include_bytes!("roms/vbl_nmi_timing/4.vbl_clear_timing.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_5_nmi_suppression_test() {
    run_test_to_pc(&mut include_bytes!("roms/vbl_nmi_timing/5.nmi_suppression.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_6_nmi_disable_test() {
    run_test_to_pc(&mut include_bytes!("roms/vbl_nmi_timing/6.nmi_disable.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn test_7_nmi_timing_test() {
    run_test_to_pc(&mut include_bytes!("roms/vbl_nmi_timing/7.nmi_timing.nes").as_ref(),
                   None, 0xe01d, &[(0xf8, 1)]);
}
