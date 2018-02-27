extern crate nes;

use test::run_test_to_pc;
use test::run_test_until_memory_matches;

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

#[test]
fn test_ppu_open_bus() {
    run_test_until_memory_matches(&mut include_bytes!("roms/ppu_open_bus/ppu_open_bus.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn test_oam_read() {
    run_test_until_memory_matches(&mut include_bytes!("roms/oam_read/oam_read.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn test_oam_stress() {
    run_test_until_memory_matches(&mut include_bytes!("roms/oam_stress/oam_stress.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}
