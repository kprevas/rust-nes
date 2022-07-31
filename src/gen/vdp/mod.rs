use std::cell::RefCell;

use gen::vdp::bus::VdpBus;

pub mod bus;

struct Vdp<'a> {
    vram: Box<[u8]>,
    cram: Box<[u8]>,
    vsram: Box<[u8]>,

    bus: &'a RefCell<VdpBus>,
}

impl<'a> Vdp<'a> {
    pub fn new(bus: &RefCell<VdpBus>) -> Vdp {
        Vdp {
            vram: vec![0; 0x10000].into_boxed_slice(),
            cram: vec![0; 0x80].into_boxed_slice(),
            vsram: vec![0; 0x50].into_boxed_slice(),
            bus,
        }
    }
}