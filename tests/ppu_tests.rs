extern crate emu;

use test::run_test_to_pc;
use test::run_test_until_memory_matches;

mod test;

#[test]
fn test_1_frame_basics_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/vbl_nmi_timing/1.frame_basics.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_2_vbl_timing_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/vbl_nmi_timing/2.vbl_timing.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_3_even_odd_frames_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/vbl_nmi_timing/3.even_odd_frames.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_4_vbl_clear_timing_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/vbl_nmi_timing/4.vbl_clear_timing.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_5_nmi_suppression_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/vbl_nmi_timing/5.nmi_suppression.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_6_nmi_disable_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/vbl_nmi_timing/6.nmi_disable.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_7_nmi_timing_test() {
    run_test_to_pc(
        &mut include_bytes!("roms/vbl_nmi_timing/7.nmi_timing.nes").as_ref(),
        None,
        0xe01d,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_ppu_open_bus() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_open_bus/ppu_open_bus.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_oam_read() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/oam_read/oam_read.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_oam_stress() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/oam_stress/oam_stress.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_01_vbl_basics() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/01-vbl_basics.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_02_vbl_set_time() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/02-vbl_set_time.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_03_vbl_clear_time() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/03-vbl_clear_time.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_04_nmi_control() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/04-nmi_control.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_05_nmi_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/05-nmi_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_06_suppression() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/06-suppression.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_07_nmi_on_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/07-nmi_on_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_08_nmi_off_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/08-nmi_off_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_09_even_odd_frames() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/09-even_odd_frames.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_10_even_odd_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/ppu_vbl_nmi/10-even_odd_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn test_sprite_hit_01_basics() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/01.basics.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_02_alignment() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/02.alignment.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_03_corners() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/03.corners.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_04_flip() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/04.flip.nes").as_ref(),
        None,
        0xe5b6,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_05_left_clip() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/05.left_clip.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_06_right_edge() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/06.right_edge.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_07_screen_bottom() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/07.screen_bottom.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_08_double_height() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/08.double_height.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_09_timing_basics() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/09.timing_basics.nes").as_ref(),
        None,
        0xe64c,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_10_timing_order() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/10.timing_order.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_hit_11_edge_timing() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_hit_tests/10.timing_order.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_overflow_1_basics() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_overflow_tests/1.Basics.nes").as_ref(),
        None,
        0xe55a,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_overflow_2_details() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_overflow_tests/2.Details.nes").as_ref(),
        None,
        0xe635,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_overflow_3_timing() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_overflow_tests/3.Timing.nes").as_ref(),
        None,
        0xe5f0,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_overflow_4_obscure() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_overflow_tests/4.Obscure.nes").as_ref(),
        None,
        0xe5f0,
        &[(0xf8, 1)],
    );
}

#[test]
fn test_sprite_overflow_5_emulator() {
    run_test_to_pc(
        &mut include_bytes!("roms/sprite_overflow_tests/5.Emulator.nes").as_ref(),
        None,
        0xe5f0,
        &[(0xf8, 1)],
    );
}
