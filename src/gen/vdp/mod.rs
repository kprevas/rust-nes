use std::cell::RefCell;
use std::convert::TryInto;

use gen::vdp::bus::{Addr, AddrMode, AddrTarget, VdpBus, WriteData};

pub mod bus;

#[allow(dead_code)]
pub struct Vdp<'a> {
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

    pub fn tick(&mut self) {
        let mut bus = self.bus.borrow_mut();
        let write_data = bus.write_data.take();
        match &bus.addr {
            Addr {
                mode: AddrMode::Read,
                target,
                addr,
                ..
            } => match target {
                AddrTarget::VRAM => {
                    bus.read_data = u32::from_le_bytes(
                        self.vram[*addr as usize..(*addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
                AddrTarget::CRAM => {
                    bus.read_data = u32::from_le_bytes(
                        self.cram[*addr as usize..(*addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
                AddrTarget::VSRAM => {
                    bus.read_data = u32::from_le_bytes(
                        self.vsram[*addr as usize..(*addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
            },
            Addr {
                mode: AddrMode::Write,
                target,
                addr,
                ..
            } => {
                let addr = *addr as usize;
                if let Some(data) = write_data {
                    match target {
                        AddrTarget::VRAM => {
                            match data {
                                WriteData::Byte(val) => {
                                    self.vram[addr] = val;
                                }
                                WriteData::Word(val) => {
                                    self.vram[addr] = (val >> 8) as u8;
                                    self.vram[addr + 1] = (val & 0xFF) as u8;
                                }
                                WriteData::Long(val) => {
                                    self.vram[addr] = (val >> 24) as u8;
                                    self.vram[addr + 1] = ((val >> 16) & 0xFF) as u8;
                                    self.vram[addr + 2] = ((val >> 8) & 0xFF) as u8;
                                    self.vram[addr + 3] = (val & 0xFF) as u8;
                                }
                            }
                        }
                        AddrTarget::CRAM => {
                            match data {
                                WriteData::Byte(val) => {
                                    self.cram[addr] = val;
                                }
                                WriteData::Word(val) => {
                                    self.cram[addr] = (val >> 8) as u8;
                                    self.cram[addr + 1] = (val & 0xFF) as u8;
                                }
                                WriteData::Long(val) => {
                                    self.cram[addr] = (val >> 24) as u8;
                                    self.cram[addr + 1] = ((val >> 16) & 0xFF) as u8;
                                    self.cram[addr + 2] = ((val >> 8) & 0xFF) as u8;
                                    self.cram[addr + 3] = (val & 0xFF) as u8;
                                }
                            }
                        }
                        AddrTarget::VSRAM => {
                            match data {
                                WriteData::Byte(val) => {
                                    self.vsram[addr] = val;
                                }
                                WriteData::Word(val) => {
                                    self.vsram[addr] = (val >> 8) as u8;
                                    self.vsram[addr + 1] = (val & 0xFF) as u8;
                                }
                                WriteData::Long(val) => {
                                    self.vsram[addr] = (val >> 24) as u8;
                                    self.vsram[addr + 1] = ((val >> 16) & 0xFF) as u8;
                                    self.vsram[addr + 2] = ((val >> 8) & 0xFF) as u8;
                                    self.vsram[addr + 3] = (val & 0xFF) as u8;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
