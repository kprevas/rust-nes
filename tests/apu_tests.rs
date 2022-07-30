extern crate emu;

use test::run_test_until_memory_matches;

mod test;

#[test]
fn apu_reset_4015_cleared() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_reset/4015_cleared.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_reset_4017_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_reset/4017_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_reset_4017_written() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_reset/4017_written.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_reset_irq_flag_cleared() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_reset/irq_flag_cleared.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn len_ctrs_enabled() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_reset/len_ctrs_enabled.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn works_immediately() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_reset/works_immediately.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_1_len_ctr() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/1-len_ctr.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_2_len_table() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/2-len_table.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_3_irq_flag() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/3-irq_flag.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_4_jitter() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/4-jitter.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_5_len_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/5-len_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_6_irq_flag_timing() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/6-irq_flag_timing.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_7_dmc_basics() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/7-dmc_basics.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}

#[test]
fn apu_test_8_dmc_rates() {
    run_test_until_memory_matches(
        &mut include_bytes!("roms/apu_test/8-dmc_rates.nes").as_ref(),
        0x6001,
        &[0xde, 0xb0, 0x61],
        0x6000,
        0x80,
        0x81,
        &[(0x6000, 0)],
    );
}
