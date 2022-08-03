use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::convert::TryInto;

use image::{GenericImage, Rgba};
use piston_window::*;
use triple_buffer::TripleBuffer;

use gen::vdp::bus::{Addr, AddrMode, AddrTarget, VdpBus, WriteData};
use gfx_device_gl::Device;
use window::renderer::Renderer;

pub mod bus;

const BRIGHTNESS_VALS: [u8; 8] = [0, 52, 87, 116, 144, 172, 206, 255];

#[allow(dead_code)]
pub struct Vdp<'a> {
    image_buffer: triple_buffer::Input<Box<[[u8; 4]; 71680]>>,
    renderer: Renderer,

    vram: Box<[u8]>,
    cram: Box<[u8]>,
    vsram: Box<[u8]>,

    bus: &'a RefCell<VdpBus>,
}

impl<'a> Vdp<'a> {
    pub fn new<'b, W: Window>(
        bus: &'b RefCell<VdpBus>,
        window: Option<&mut PistonWindow<W>>,
    ) -> Vdp<'b> {
        let (image_buffer, image_buffer_out) =
            TripleBuffer::new(&Box::new([[0u8; 4]; 71680])).split();
        let renderer = Renderer::new(window, image_buffer_out, 320, |image_buffer_out, image| {
            let pixels = image_buffer_out.output_buffer();
            let mut dot = 0;
            let mut scanline = 0;
            for color in pixels.iter() {
                image.put_pixel(dot, scanline, Rgba(*color));
                dot += 1;
                if dot == 320 {
                    dot = 0;
                    scanline += 1;
                }
            }
        });

        Vdp {
            image_buffer,
            renderer,
            vram: vec![0; 0x10000].into_boxed_slice(),
            cram: vec![0; 0x80].into_boxed_slice(),
            vsram: vec![0; 0x50].into_boxed_slice(),
            bus,
        }
    }

    pub fn tick(&mut self, m68k_cartridge: &[u8], m68k_ram: &[u8]) {
        let mut bus = self.bus.borrow_mut();
        let write_data = bus.write_data.take();
        match bus.addr {
            Some(Addr {
                     mode: AddrMode::Read,
                     target,
                     addr,
                     ..
                 }) => match target {
                AddrTarget::VRAM => {
                    bus.read_data = u32::from_le_bytes(
                        self.vram[addr as usize..(addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
                AddrTarget::CRAM => {
                    bus.read_data = u32::from_le_bytes(
                        self.cram[addr as usize..(addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
                AddrTarget::VSRAM => {
                    bus.read_data = u32::from_le_bytes(
                        self.vsram[addr as usize..(addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
            },
            Some(Addr {
                     mode: AddrMode::Write,
                     target,
                     addr,
                     dma: false,
                     ..
                 }) => {
                let addr = addr as usize;
                if let Some(data) = write_data {
                    match target {
                        AddrTarget::VRAM => match data {
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
                        },
                        AddrTarget::CRAM => match data {
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
                        },
                        AddrTarget::VSRAM => match data {
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
                        },
                    }
                }
            }
            Some(Addr {
                     mode: AddrMode::Write,
                     target,
                     dma: true,
                     ..
                 }) => {
                // TODO DMA shouldn't happen instantaneously
                bus.dma(
                    m68k_cartridge,
                    m68k_ram,
                    match target {
                        AddrTarget::VRAM => self.vram.borrow_mut(),
                        AddrTarget::CRAM => self.cram.borrow_mut(),
                        AddrTarget::VSRAM => self.vsram.borrow_mut(),
                    },
                );
            }
            None => {}
        }
    }

    fn get_color(&self, palette_line: u8, palette_index: u8) -> Rgba<u8> {
        let index = ((palette_line * 16 + palette_index) * 2) as usize;
        let palette_val_high = self.cram[index];
        let palette_val_low = self.cram[index + 1];
        Rgba([
            BRIGHTNESS_VALS[(palette_val_high / 2) as usize],
            BRIGHTNESS_VALS[((palette_val_low >> 4) / 2) as usize],
            BRIGHTNESS_VALS[((palette_val_low & 0xF) / 2) as usize],
            0xff,
        ])
    }

    pub fn render(
        &mut self,
        c: Context,
        texture_ctx: &mut G2dTextureContext,
        gl: &mut G2d,
        device: &mut Device,
    ) {
        self.renderer.render(c, texture_ctx, gl, device);
    }

    pub fn close(&mut self) {
        self.image_buffer.publish();
        self.renderer.close();
    }
}
