extern crate nes;

use test::run_test_to_pc;

mod test;

#[test]
fn nes_test() {
    run_test_to_pc(&mut include_bytes!("roms/nestest/nestest.nes").as_ref(),
                   Some(0xc000), 0xc66e, &[(0x02, 0), (0x03, 0)]);
}
