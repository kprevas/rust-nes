use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::convert::TryInto;

use gfx_device_gl::Device;
use image::{GenericImage, Rgba};
use num_integer::Integer;
use piston_window::*;
use triple_buffer::TripleBuffer;

use gen::vdp::bus::{
    Addr, AddrMode, AddrTarget, HorizontalScrollingMode, VdpBus, VerticalScrollingMode, WindowHPos,
    WindowVPos, WriteData,
};
use window::renderer::Renderer;

pub mod bus;

const BRIGHTNESS_VALS: [u8; 8] = [0, 52, 87, 116, 144, 172, 206, 255];

enum SpritePixel {
    Transparent,
    Shadow,
    Highlight,
    Color([u8; 4]),
}

#[allow(dead_code)]
pub struct Vdp<'a> {
    image_buffer: triple_buffer::Input<Box<[[u8; 4]; 71680]>>,
    renderer: Renderer,

    scanline: u16,
    dot: u16,

    hblank_counter: u16,

    vram: Box<[u8]>,
    cram: Box<[u8]>,
    vsram: Box<[u8]>,

    master_clock_ticks: u8,
    pixel_clock_tick: bool,

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
            scanline: 0,
            dot: 0,
            hblank_counter: 0,
            vram: vec![0; 0x10000].into_boxed_slice(),
            cram: vec![0; 0x80].into_boxed_slice(),
            vsram: vec![0; 0x50].into_boxed_slice(),
            master_clock_ticks: 0,
            pixel_clock_tick: false,
            bus,
        }
    }

    pub fn cpu_tick(&mut self, m68k_cartridge: &[u8], m68k_ram: &[u8]) {
        self.master_clock_ticks += 7;
        while self.master_clock_ticks > 4 {
            self.tick(m68k_cartridge, m68k_ram);
            self.master_clock_ticks -= 4;
        }
    }

    fn tick(&mut self, m68k_cartridge: &[u8], m68k_ram: &[u8]) {
        self.handle_bus_data(m68k_cartridge, m68k_ram);

        if self.pixel_clock_tick {
            self.tick_pixel();
        }
        self.pixel_clock_tick = !self.pixel_clock_tick;
    }

    fn handle_bus_data(&mut self, m68k_cartridge: &[u8], m68k_ram: &[u8]) {
        let mut bus = self.bus.borrow_mut();
        let write_data = bus.write_data.pop_front();
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
                        },
                        AddrTarget::CRAM => match data {
                            WriteData::Byte(val) => {
                                self.cram[addr] = val;
                            }
                            WriteData::Word(val) => {
                                self.cram[addr] = (val >> 8) as u8;
                                self.cram[addr + 1] = (val & 0xFF) as u8;
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
                        },
                    }
                    bus.increment_addr();
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
                    write_data,
                );
            }
            None => {}
        }
    }

    fn tick_pixel(&mut self) {
        let mut bus = self.bus.borrow_mut();
        let width = if bus.mode_4.h_40_wide_mode { 320 } else { 256 };

        if self.dot < width && self.scanline >= 11 && self.scanline - 11 < 224 {
            let mut pixel = None;
            let x = self.dot;
            let y = self.scanline - 11;

            let mut shadow = false;
            let mut highlight = false;

            let window_tile_index = (y / 8) * 64 + (x / 8);
            let window_tile_data_addr =
                (bus.window_nametable_addr + window_tile_index * 2) as usize;
            let window_tile_data = (self.vram[window_tile_data_addr] as u16) << 8
                | (self.vram[window_tile_data_addr + 1] as u16);
            let window_priority = (window_tile_data >> 15) & 0b1 > 0;

            let x_in_window = match bus.window_h_pos {
                WindowHPos::DrawToRight(window_width) => x > width - window_width as u16 * 8,
                WindowHPos::DrawToLeft(window_width) => x < window_width as u16 * 8,
            };
            let y_in_window = match bus.window_v_pos {
                WindowVPos::DrawToTop(window_height) => y < window_height as u16 * 8,
                WindowVPos::DrawToBottom(window_height) => y > 224 - window_height as u16 * 8,
            };
            let window_pixel = if x_in_window || y_in_window {
                self.get_pixel(x, y, window_tile_data, false, false)
            } else {
                None
            };

            let (plane_a_x, plane_a_y, plane_a_tile_data) = self.plane_scroll(
                x,
                y,
                bus.mode_3.vertical_scrolling_mode,
                bus.mode_3.horizontal_scrolling_mode,
                bus.plane_height,
                bus.plane_width,
                bus.horizontal_scroll_data_addr,
                bus.plane_a_nametable_addr,
                0,
            );

            let (plane_b_x, plane_b_y, plane_b_tile_data) = self.plane_scroll(
                x,
                y,
                bus.mode_3.vertical_scrolling_mode,
                bus.mode_3.horizontal_scrolling_mode,
                bus.plane_height,
                bus.plane_width,
                bus.horizontal_scroll_data_addr,
                bus.plane_b_nametable_addr,
                2,
            );

            let plane_a_priority = (plane_a_tile_data >> 15) & 0b1 > 0;
            let plane_b_priority = (plane_b_tile_data >> 15) & 0b1 > 0;
            if bus.mode_4.enable_shadow_highlight && !plane_a_priority && !plane_b_priority {
                shadow = true;
            }

            let (sprite_pixel, sprite_priority) = self.get_sprite_pixel(
                x,
                y,
                bus.sprite_table_addr as usize,
                bus.mode_4.enable_shadow_highlight,
                shadow,
            );

            let sprite_pixel = match sprite_pixel {
                SpritePixel::Transparent => None,
                SpritePixel::Shadow => {
                    shadow = true;
                    None
                }
                SpritePixel::Highlight => {
                    if shadow {
                        shadow = false;
                    } else {
                        highlight = true;
                    }
                    None
                }
                SpritePixel::Color(color) => Some(color),
            };

            let plane_a_pixel =
                self.get_pixel(plane_a_x, plane_a_y, plane_a_tile_data, shadow, highlight);
            let plane_b_pixel =
                self.get_pixel(plane_b_x, plane_b_y, plane_b_tile_data, shadow, highlight);

            if pixel.is_none() && window_priority {
                pixel = window_pixel;
            }
            if pixel.is_none() && sprite_priority {
                pixel = sprite_pixel;
            }
            if pixel.is_none() && plane_a_priority {
                pixel = plane_a_pixel;
            }
            if pixel.is_none() && plane_b_priority {
                pixel = plane_b_pixel;
            }
            if pixel.is_none() {
                pixel = window_pixel;
            }
            if pixel.is_none() {
                pixel = sprite_pixel;
            }
            if pixel.is_none() {
                pixel = plane_a_pixel;
            }
            if pixel.is_none() {
                pixel = plane_b_pixel;
            }
            if pixel.is_none() {
                pixel = Some(self.get_color(bus.bg_palette, bus.bg_color, false, false));
            }

            self.image_buffer.input_buffer()
                [y as usize * 320 as usize + ((320 - width) / 2) as usize + x as usize] =
                pixel.unwrap();
        }

        self.dot += 1;
        if self.dot == width {
            if bus.mode_1.enable_horizontal_interrupt {
                if self.hblank_counter == 0 {
                    bus.horizontal_interrupt = true;
                    self.hblank_counter = bus.horizontal_interrupt_counter;
                } else {
                    self.hblank_counter -= 1;
                }
            }
        } else if self.dot == width + 33 {
            self.dot = 0;
            self.scanline += 1;
            if self.scanline == 243 {
                if bus.mode_2.enable_vertical_interrupt {
                    bus.vertical_interrupt = true;
                }
                self.image_buffer.publish();
                let bg = self.get_color(bus.bg_palette, bus.bg_color, false, false);
                self.image_buffer.input_buffer().fill(bg);
            } else if self.scanline == 262 {
                self.scanline = 0;
            }
        }

        // TODO: match https://plutiedev.com/mirror/kabuto-hardware-notes#hv-counter
        bus.beam_vpos = self.scanline.max(243);
        bus.beam_hpos = self.dot.max(width);
    }

    fn plane_scroll(
        &mut self,
        x: u16,
        y: u16,
        v_scroll_mode: VerticalScrollingMode,
        h_scroll_mode: HorizontalScrollingMode,
        plane_height: u16,
        plane_width: u16,
        h_scroll_data_addr: u16,
        nametable_addr: u16,
        plane_offset: usize,
    ) -> (u16, u16, u16) {
        let v_scroll_index = match v_scroll_mode {
            VerticalScrollingMode::Column16Pixels => (x / 16 * 2 * 2) as usize,
            VerticalScrollingMode::FullScreen => 0,
        } + plane_offset;
        let v_scroll = i16::from_be_bytes(
            self.vsram[v_scroll_index..=v_scroll_index + 1]
                .try_into()
                .unwrap(),
        );

        let h_scroll_index = match h_scroll_mode {
            HorizontalScrollingMode::Row1Pixel => {
                (((y.wrapping_add_signed(v_scroll)) % plane_height) * 2 * 2) as usize
            }
            HorizontalScrollingMode::Row8Pixel => {
                (((y.wrapping_add_signed(v_scroll)) % plane_height) / 8 * 8 * 2 * 2) as usize
            }
            HorizontalScrollingMode::FullScreen => 0,
            HorizontalScrollingMode::Invalid => 0,
        } + plane_offset;
        let h_scroll = if let HorizontalScrollingMode::Invalid = h_scroll_mode {
            0
        } else {
            i16::from_be_bytes(
                self.vram[h_scroll_data_addr as usize + h_scroll_index
                    ..=h_scroll_data_addr as usize + h_scroll_index + 1]
                    .try_into()
                    .unwrap(),
            )
        };

        let x = (x.wrapping_add_signed(h_scroll)) % plane_width;
        let y = (y.wrapping_add_signed(v_scroll)) % plane_height;

        let tile_x = x / 8;
        let tile_y = y / 8;
        let tile_index = tile_y * (plane_width / 8) + tile_x;
        let tile_data_addr = (nametable_addr + tile_index * 2) as usize;
        let tile_data =
            (self.vram[tile_data_addr] as u16) << 8 | (self.vram[tile_data_addr + 1] as u16);

        (x, y, tile_data)
    }

    fn get_sprite_pixel(
        &self,
        x: u16,
        y: u16,
        sprite_table_addr: usize,
        enable_shadow_highlight: bool,
        shadow: bool,
    ) -> (SpritePixel, bool) {
        let x = x + 128;
        let y = y + 128;
        let mut sprite_index = 0;
        while {
            let sprite_addr = sprite_table_addr + sprite_index * 8;

            let sprite_y =
                (self.vram[sprite_addr] as u16) << 8 | (self.vram[sprite_addr + 1] as u16);
            let sprite_x =
                (self.vram[sprite_addr + 6] as u16) << 8 | (self.vram[sprite_addr + 7] as u16);
            let height = (self.vram[sprite_addr + 2] & 0b11) as u16 + 1;
            let width = ((self.vram[sprite_addr + 2] >> 2) & 0b11) as u16 + 1;
            let tile = ((self.vram[sprite_addr + 4] & 0b111) as u16) << 8
                | (self.vram[sprite_addr + 5] as u16);
            let high_priority = (self.vram[sprite_addr + 4] & 0b10000000) > 0;
            let flip_vertical = (self.vram[sprite_addr + 4] & 0b00010000) > 0;
            let flip_horizontal = (self.vram[sprite_addr + 4] & 0b00001000) > 0;
            let palette_line = (self.vram[sprite_addr + 4] >> 5) & 0b11;

            if sprite_y <= y
                && sprite_x <= x
                && sprite_y + 8 * height > y
                && sprite_x + 8 * width > x
            {
                let x_in_sprite = x - sprite_x;
                let y_in_sprite = y - sprite_y;
                let (mut x_tile, mut x_offset) = x_in_sprite.div_rem(&8);
                let (mut y_tile, mut y_offset) = y_in_sprite.div_rem(&8);
                if flip_vertical {
                    y_tile = height - 1 - y_tile;
                    y_offset = 7 - y_offset;
                }
                if flip_horizontal {
                    x_tile = width - 1 - x_tile;
                    x_offset = 7 - x_offset;
                }
                let tile_index = height * x_tile + y_tile;
                let tile_addr = (tile + tile_index) as usize * 0x20;
                let pixel_addr = tile_addr + (y_offset as usize * 8) / 2 + x_offset as usize / 2;
                let pixel_data = self.vram[pixel_addr];
                let palette_color = if x_offset % 2 == 1 {
                    pixel_data & 0xF
                } else {
                    pixel_data >> 4
                };
                if palette_color != 0 {
                    if enable_shadow_highlight && palette_line == 3 {
                        if palette_color == 14 {
                            return (SpritePixel::Highlight, high_priority);
                        }
                        if palette_color == 15 {
                            return (SpritePixel::Shadow, high_priority);
                        }
                    }
                    return (
                        SpritePixel::Color(self.get_color(
                            palette_line,
                            palette_color,
                            shadow,
                            false,
                        )),
                        high_priority,
                    );
                }
            }

            sprite_index = self.vram[sprite_addr + 3] as usize;
            sprite_index != 0
        } {}
        (SpritePixel::Transparent, false)
    }

    fn get_pixel(
        &self,
        x: u16,
        y: u16,
        tile_data: u16,
        shadow: bool,
        highlight: bool,
    ) -> Option<[u8; 4]> {
        let palette_line = ((tile_data >> 13) & 0b11) as u8;
        let v_flip = (tile_data >> 12) & 0b1 == 1;
        let h_flip = (tile_data >> 11) & 0b1 == 1;
        let tile_index = tile_data & 0x7FF;
        let tile_addr = tile_index as usize * 0x20;
        let tile_x = if h_flip { 7 - (x % 8) } else { x % 8 };
        let tile_y = if v_flip { 7 - (y % 8) } else { y % 8 };
        let pixel_addr = tile_addr + (tile_y as usize * 8) / 2 + tile_x as usize / 2;
        let pixel_data = self.vram[pixel_addr];
        let palette_color = if tile_x % 2 == 1 {
            pixel_data & 0xF
        } else {
            pixel_data >> 4
        };
        if palette_color == 0 {
            None
        } else {
            Some(self.get_color(palette_line, palette_color, shadow, highlight))
        }
    }

    fn get_color(
        &self,
        palette_line: u8,
        palette_index: u8,
        shadow: bool,
        highlight: bool,
    ) -> [u8; 4] {
        let index = ((palette_line * 16 + palette_index) * 2) as usize;
        let palette_val_high = self.cram[index];
        let palette_val_low = self.cram[index + 1];
        let r = BRIGHTNESS_VALS[((palette_val_low & 0xF) / 2) as usize];
        let g = BRIGHTNESS_VALS[((palette_val_low >> 4) / 2) as usize];
        let b = BRIGHTNESS_VALS[(palette_val_high / 2) as usize];
        if shadow {
            [r / 2, g / 2, b / 2, 0xff]
        } else if highlight {
            [
                r.saturating_mul(2),
                g.saturating_mul(2),
                b.saturating_mul(2),
                0xff,
            ]
        } else {
            [r, g, b, 0xff]
        }
    }

    pub fn render(
        &mut self,
        c: Context,
        texture_ctx: &mut G2dTextureContext,
        gl: &mut G2d,
        device: &mut Device,
    ) {
        self.renderer.render(c, texture_ctx, gl, device, 1.0);
    }

    pub fn close(&mut self) {
        self.image_buffer.publish();
        self.renderer.close();
    }
}
