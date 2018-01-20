extern crate nes;

mod test;

use test::run_test;

#[test]
fn nes_test() {
    run_test(&mut include_bytes!("roms/nestest.nes").as_ref(), Some(0xc000), 0xc66e, &[(0x02, 0), (0x03, 0)]);
}
