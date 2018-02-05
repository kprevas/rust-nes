extern crate nes;

mod test;

use test::run_test;

#[test]
fn frame_basics_test() {
    run_test(&mut include_bytes!("roms/1.frame_basics.nes").as_ref(), None, 0xe01d, &[(0xf8, 1)]);
}

#[test]
fn vbl_timing_test() {
    run_test(&mut include_bytes!("roms/2.vbl_timing.nes").as_ref(), None, 0xe01d, &[(0xf8, 1)]);
}
