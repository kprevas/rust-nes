extern crate nes;

use test::run_test_until_memory_matches;

mod test;

#[test]
fn apu_reset_4015_cleared() {
    run_test_until_memory_matches(&mut include_bytes!("roms/apu_reset/4015_cleared.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn apu_reset_4017_timing() {
    run_test_until_memory_matches(&mut include_bytes!("roms/apu_reset/4017_timing.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn apu_reset_4017_written() {
    run_test_until_memory_matches(&mut include_bytes!("roms/apu_reset/4017_written.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn apu_reset_irq_flag_cleared() {
    run_test_until_memory_matches(&mut include_bytes!("roms/apu_reset/irq_flag_cleared.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn len_ctrs_enabled() {
    run_test_until_memory_matches(&mut include_bytes!("roms/apu_reset/len_ctrs_enabled.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn works_immediately() {
    run_test_until_memory_matches(&mut include_bytes!("roms/apu_reset/works_immediately.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}

#[test]
fn apu_test() {
    run_test_until_memory_matches(&mut include_bytes!("roms/apu_test/apu_test.nes").as_ref(),
                                  0x6001,
                                  &[0xde, 0xb0, 0x61],
                                  0x6000,
                                  0x80,
                                  0x81,
                                  &[(0x6000, 0)]);
}
