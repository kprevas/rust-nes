use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::convert::TryInto;

use gfx_device_gl::Device;
use image::{GenericImage, Rgba};
use num_integer::Integer;
use piston_window::*;
use triple_buffer::triple_buffer;

use gen::vdp::bus::{
    Addr, AddrMode, AddrTarget, HorizontalScrollingMode, Status, VdpBus, VerticalScrollingMode,
    WindowHPos, WindowVPos, WriteData,
};
use window::renderer::Renderer;

pub mod bus;

const BRIGHTNESS_VALS: [u8; 8] = [0, 52, 87, 116, 144, 172, 206, 255];
const BRIGHTNESS_VALS_SHADOW: [u8; 8] = [0, 29, 52, 70, 87, 101, 116, 130];
const BRIGHTNESS_VALS_HIGHLIGHT: [u8; 8] = [130, 144, 158, 172, 187, 206, 228, 255];

const WINDOW_PRIORITY: usize = 7;
const SPRITE_PRIORITY: usize = 6;
const PLANE_A_PRIORITY: usize = 5;
const PLANE_B_PRIORITY: usize = 4;
const WINDOW: usize = 3;
const SPRITE: usize = 2;
const PLANE_A: usize = 1;
const PLANE_B: usize = 0;

#[derive(Copy, Clone)]
enum SpritePixel {
    Transparent,
    Shadow,
    Highlight,
    Color { palette_line: u8, palette_color: u8 },
}

#[derive(Debug, Default, Copy, Clone)]
struct Sprite {
    y: u16,
    x: u16,
    height: u16,
    width: u16,
    tile: u16,
    high_priority: bool,
    flip_vertical: bool,
    flip_horizontal: bool,
    palette_line: u8,
    next: usize,
}

#[allow(dead_code)]
pub struct Vdp<'a> {
    image_buffers: [triple_buffer::Input<Box<[[u8; 4]; 71680]>>; 8],
    renderer: Renderer<8>,

    scanline: u16,
    dot: u16,
    h_counter: u16,
    v_counter: u16,

    sprite_line_buffer: [(SpritePixel, bool); 320],
    dot_overflow: bool,
    prev_line_dot_overflow: bool,

    hblank_counter: u16,

    vram: Box<[u8]>,
    cram: Box<[u8]>,
    vsram: Box<[u8]>,

    master_clock_ticks: u8,
    pixel_clock_tick: bool,

    bus: &'a RefCell<VdpBus>,

    dump_mode: bool,
    instrumented: bool,
}

impl<'a> Vdp<'a> {
    pub fn new<'b, W: Window>(
        bus: &'b RefCell<VdpBus>,
        window: Option<&mut PistonWindow<W>>,
        dump_mode: bool,
        instrumented: bool,
    ) -> Vdp<'b> {
        let (buf0, buf0_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let (buf1, buf1_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let (buf2, buf2_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let (buf3, buf3_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let (buf4, buf4_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let (buf5, buf5_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let (buf6, buf6_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let (buf7, buf7_out) = triple_buffer(&Box::new([[0u8; 4]; 71680]));
        let image_buffers = [buf0, buf1, buf2, buf3, buf4, buf5, buf6, buf7];
        let image_buffer_outs = [
            buf0_out, buf1_out, buf2_out, buf3_out, buf4_out, buf5_out, buf6_out, buf7_out,
        ];
        let renderer = Renderer::new(window, image_buffer_outs, 320, |image_buffer_out, image| {
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
            image_buffers,
            renderer,
            scanline: 0,
            dot: 0,
            h_counter: 0,
            v_counter: 0,
            sprite_line_buffer: [(SpritePixel::Transparent, false); 320],
            dot_overflow: false,
            prev_line_dot_overflow: false,
            hblank_counter: 0,
            vram: vec![0; 0x10000].into_boxed_slice(),
            cram: vec![0; 0x80].into_boxed_slice(),
            vsram: vec![0; 0x50].into_boxed_slice(),
            master_clock_ticks: 0,
            pixel_clock_tick: false,
            bus,
            dump_mode,
            instrumented,
        }
    }

    pub fn tick(&mut self, m68k_cartridge: &[u8], m68k_ram: &[u8]) {
        self.master_clock_ticks += 1;
        while self.master_clock_ticks > 4 {
            self.do_tick(m68k_cartridge, m68k_ram);
            self.master_clock_ticks -= 4;
        }
    }

    fn do_tick(&mut self, m68k_cartridge: &[u8], m68k_ram: &[u8]) {
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
                    bus.read_data = u32::from_be_bytes(
                        self.vram[addr as usize..(addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
                AddrTarget::CRAM => {
                    bus.read_data = u32::from_be_bytes(
                        self.cram[addr as usize..(addr + 4) as usize]
                            .try_into()
                            .unwrap(),
                    );
                }
                AddrTarget::VSRAM => {
                    bus.read_data = u32::from_be_bytes(
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
        let max_sprites_per_line = if bus.mode_4.h_40_wide_mode { 20 } else { 16 };
        let max_sprites_per_frame = if bus.mode_4.h_40_wide_mode { 80 } else { 64 };
        let active_display_h = if bus.mode_4.h_40_wide_mode { 26 } else { 24 };
        let active_display_h_end = if bus.mode_4.h_40_wide_mode { 345 } else { 279 };

        if self.h_counter >= active_display_h
            && self.h_counter <= active_display_h_end
            && self.dot < width
            && self.scanline < 224
        {
            let x = self.dot;
            let y = self.scanline;

            if self.dump_mode {
                self.draw_dump_pixel(x, y, width);
            } else {
                let i = y as usize * 320 as usize + ((320 - width) / 2) as usize + x as usize;

                let mut shadow = false;
                let mut highlight = false;

                let window_tile_index = (y / 8) * 64 + (x / 8);
                let window_tile_data_addr =
                    (bus.window_nametable_addr + window_tile_index * 2) as usize;
                let window_tile_data = (self.vram[window_tile_data_addr] as u16) << 8
                    | (self.vram[window_tile_data_addr + 1] as u16);
                let window_priority = (window_tile_data >> 15) & 0b1 > 0;

                let x_in_window = match bus.window_h_pos {
                    WindowHPos::DrawToRight(window_base) => x > window_base as u16 * 8,
                    WindowHPos::DrawToLeft(window_base) => x < window_base as u16 * 8,
                };
                let y_in_window = match bus.window_v_pos {
                    WindowVPos::DrawToTop(window_base) => y < window_base as u16 * 8,
                    WindowVPos::DrawToBottom(window_base) => y > window_base as u16 * 8,
                };

                self.image_buffers[WINDOW_PRIORITY].input_buffer()[i] =
                    if window_priority && (x_in_window || y_in_window) {
                        self.get_pixel(x, y, window_tile_data, false, false)
                            .unwrap_or([0, 0, 0, 0])
                    } else {
                        [0, 0, 0, 0]
                    };
                self.image_buffers[WINDOW].input_buffer()[i] =
                    if !window_priority && (x_in_window || y_in_window) {
                        self.get_pixel(x, y, window_tile_data, false, false)
                            .unwrap_or([0, 0, 0, 0])
                    } else {
                        [0, 0, 0, 0]
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

                let (sprite_pixel, sprite_priority) = self.sprite_line_buffer[x as usize];

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
                    SpritePixel::Color {
                        palette_line,
                        palette_color,
                    } => Some(self.get_color(palette_line, palette_color, shadow, highlight)),
                };
                self.image_buffers[SPRITE_PRIORITY].input_buffer()[i] = if sprite_priority {
                    sprite_pixel.unwrap_or([0, 0, 0, 0])
                } else {
                    [0, 0, 0, 0]
                };
                self.image_buffers[SPRITE].input_buffer()[i] = if !sprite_priority {
                    sprite_pixel.unwrap_or([0, 0, 0, 0])
                } else {
                    [0, 0, 0, 0]
                };

                let plane_a_pixel =
                    self.get_pixel(plane_a_x, plane_a_y, plane_a_tile_data, shadow, highlight);
                self.image_buffers[PLANE_A_PRIORITY].input_buffer()[i] =
                    if x_in_window || y_in_window {
                        [0, 0, 0, 0]
                    } else if plane_a_priority {
                        plane_a_pixel.unwrap_or([0, 0, 0, 0])
                    } else {
                        [0, 0, 0, 0]
                    };
                self.image_buffers[PLANE_A].input_buffer()[i] = if x_in_window || y_in_window {
                    [0, 0, 0, 0]
                } else if !plane_a_priority {
                    plane_a_pixel.unwrap_or([0, 0, 0, 0])
                } else {
                    [0, 0, 0, 0]
                };

                let plane_b_pixel =
                    self.get_pixel(plane_b_x, plane_b_y, plane_b_tile_data, shadow, highlight);
                self.image_buffers[PLANE_B_PRIORITY].input_buffer()[i] = if plane_b_priority {
                    plane_b_pixel.unwrap_or([0, 0, 0, 0])
                } else {
                    [0, 0, 0, 0]
                };
                self.image_buffers[PLANE_B].input_buffer()[i] = if !plane_b_priority {
                    plane_b_pixel.unwrap_or([0, 0, 0, 0])
                } else {
                    [0, 0, 0, 0]
                };
            }
        }

        self.h_counter += 1;
        if self.h_counter > active_display_h && self.h_counter <= active_display_h_end {
            self.dot += 1;
        }
        if self.h_counter == if bus.mode_4.h_40_wide_mode { 6 } else { 5 } {
            bus.status.hblank = false;
            bus.horizontal_interrupt = false;
            bus.status.sprite_limit = false;
            bus.status.sprite_overlap = false;
            self.prev_line_dot_overflow = self.dot_overflow;
            self.dot_overflow = false;
        } else if self.h_counter == if bus.mode_4.h_40_wide_mode { 330 } else { 266 } {
            self.v_counter += 1;
            if self.v_counter == 224 {
                bus.status.vblank = true;
                if bus.mode_2.enable_vertical_interrupt {
                    bus.vertical_interrupt = true;
                }
                bus.z80_interrupt = true;
                for buf in &mut self.image_buffers {
                    buf.publish();
                }
                let bg = self.get_color(bus.bg_palette, bus.bg_color, false, false);
                self.renderer.set_background(bg.map(|c| (c as f32) / 255.0));
                for buf in &mut self.image_buffers {
                    buf.input_buffer().fill([0, 0, 0, 0]);
                }
                if self.dump_mode {
                    self.dump_sprite_table(bus.sprite_table_addr as usize);
                }
            } else if self.v_counter == 225 {
                bus.z80_interrupt = false;
            } else if self.v_counter == 261 {
                bus.status.vblank = false;
                bus.vertical_interrupt = false;
            } else if self.v_counter == 262 {
                self.v_counter = 0;
            }
        } else if self.h_counter == if bus.mode_4.h_40_wide_mode { 358 } else { 294 } {
            bus.status.hblank = true;
            if bus.mode_1.enable_horizontal_interrupt {
                if self.scanline == 0 || self.scanline > 224 {
                    self.hblank_counter = bus.horizontal_interrupt_counter;
                } else if self.hblank_counter == 0 {
                    bus.horizontal_interrupt = true;
                    self.hblank_counter = bus.horizontal_interrupt_counter;
                } else {
                    self.hblank_counter -= 1;
                }
            }
        } else if self.h_counter == if bus.mode_4.h_40_wide_mode { 420 } else { 324 } {
            self.h_counter = 0;
            self.dot = 0;
            self.scanline += 1;
            if self.v_counter == 0 {
                self.prev_line_dot_overflow = false;
                self.scanline = 0;
            }
            self.fill_sprite_buffer(
                self.scanline,
                bus.sprite_table_addr as usize,
                max_sprites_per_line as usize,
                max_sprites_per_frame,
                width,
                bus.mode_4.enable_shadow_highlight,
                &mut bus.status,
            );
        }

        bus.beam_vpos = self.v_counter;
        if bus.beam_vpos > 234 {
            bus.beam_vpos += 250;
        }

        bus.beam_hpos = self.h_counter;
        if bus.mode_4.h_40_wide_mode {
            if bus.beam_hpos > 364 {
                bus.beam_hpos += 92;
            }
        } else {
            if bus.beam_hpos > 295 {
                bus.beam_hpos += 170;
            }
        }
        if bus.beam_hpos > 512 {
            bus.beam_hpos -= 512;
        }
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
            HorizontalScrollingMode::Row1Pixel => (y * 2 * 2) as usize,
            HorizontalScrollingMode::Row8Pixel => (y / 8 * 8 * 2 * 2) as usize,
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

        let x = (x.wrapping_add_signed(-h_scroll)) % plane_width;
        let y = (y.wrapping_add_signed(v_scroll)) % plane_height;

        let tile_x = x / 8;
        let tile_y = y / 8;
        let tile_index = tile_y * (plane_width / 8) + tile_x;
        let tile_data_addr = (nametable_addr + tile_index * 2) as usize;
        let tile_data =
            (self.vram[tile_data_addr] as u16) << 8 | (self.vram[tile_data_addr + 1] as u16);

        (x, y, tile_data)
    }

    fn fill_sprite_buffer(
        &mut self,
        y: u16,
        sprite_table_addr: usize,
        max_sprites_per_line: usize,
        max_sprites_per_frame: usize,
        width: u16,
        enable_shadow_highlight: bool,
        status: &mut Status,
    ) {
        self.sprite_line_buffer = [(SpritePixel::Transparent, false); 320];
        let y = y + 128;
        let mut sprite_index = 0;
        let mut sprites_in_line = 0;
        let mut dots_in_line = 0;
        let mut unmasked_sprite_on_line = self.prev_line_dot_overflow;
        let mut masked = false;
        let mut total_sprites = 0;
        let mut line_sprites = [Sprite::default(); 20];
        while {
            let sprite_addr = sprite_table_addr + sprite_index * 8;
            let sprite = self.read_sprite(sprite_addr);

            if sprite.y <= y && sprite.y + 8 * sprite.height > y {
                if !masked {
                    if sprite.x == 0 {
                        if unmasked_sprite_on_line {
                            masked = true;
                        }
                    } else {
                        unmasked_sprite_on_line = true;
                        line_sprites[sprites_in_line] = sprite;
                        line_sprites[sprites_in_line].width = line_sprites[sprites_in_line]
                            .width
                            .min((width - dots_in_line) / 8);
                    }
                }
                sprites_in_line += 1;
                dots_in_line += sprite.width * 8;
            }

            total_sprites += 1;
            sprite_index = sprite.next;
            sprite.next != 0
                && sprite_index < max_sprites_per_frame
                && sprites_in_line < max_sprites_per_line
                && dots_in_line < width
                && total_sprites < max_sprites_per_frame
        } {}

        if sprites_in_line >= max_sprites_per_line {
            status.sprite_limit = true;
        }
        if dots_in_line >= width {
            self.dot_overflow = true;
        }
        for x in 0..width {
            let mut sprite_pixel = SpritePixel::Transparent;
            let mut high_priority = false;

            for sprite in line_sprites {
                let x = x + 128;
                if sprite.x <= x && sprite.x + 8 * sprite.width > x {
                    let x_in_sprite = x - sprite.x;
                    let y_in_sprite = y - sprite.y;
                    let (mut x_tile, mut x_offset) = x_in_sprite.div_rem(&8);
                    let (mut y_tile, mut y_offset) = y_in_sprite.div_rem(&8);
                    if sprite.flip_vertical {
                        y_tile = sprite.height - 1 - y_tile;
                        y_offset = 7 - y_offset;
                    }
                    if sprite.flip_horizontal {
                        x_tile = sprite.width - 1 - x_tile;
                        x_offset = 7 - x_offset;
                    }
                    let tile_index = sprite.height * x_tile + y_tile;
                    let tile_addr = (sprite.tile + tile_index) as usize * 0x20;
                    let pixel_addr =
                        tile_addr + (y_offset as usize * 8) / 2 + x_offset as usize / 2;
                    let pixel_data = self.vram[pixel_addr];
                    let palette_color = if x_offset % 2 == 1 {
                        pixel_data & 0xF
                    } else {
                        pixel_data >> 4
                    };

                    match (&sprite_pixel, palette_color) {
                        (_, 0) => {}
                        (SpritePixel::Transparent, _) => {
                            sprite_pixel = if enable_shadow_highlight
                                && sprite.palette_line == 3
                                && palette_color == 14
                            {
                                SpritePixel::Highlight
                            } else if enable_shadow_highlight
                                && sprite.palette_line == 3
                                && palette_color == 15
                            {
                                SpritePixel::Shadow
                            } else {
                                SpritePixel::Color {
                                    palette_line: sprite.palette_line,
                                    palette_color,
                                }
                            };
                            high_priority = sprite.high_priority;
                        }
                        (_, _) => status.sprite_overlap = true,
                    }
                }
            }

            self.sprite_line_buffer[x as usize] = (sprite_pixel, high_priority);
        }
    }

    fn read_sprite(&self, sprite_addr: usize) -> Sprite {
        Sprite {
            y: (self.vram[sprite_addr] as u16) << 8 | (self.vram[sprite_addr + 1] as u16),
            x: (self.vram[sprite_addr + 6] as u16) << 8 | (self.vram[sprite_addr + 7] as u16),
            height: (self.vram[sprite_addr + 2] & 0b11) as u16 + 1,
            width: ((self.vram[sprite_addr + 2] >> 2) & 0b11) as u16 + 1,
            tile: ((self.vram[sprite_addr + 4] & 0b111) as u16) << 8
                | (self.vram[sprite_addr + 5] as u16),
            high_priority: (self.vram[sprite_addr + 4] & 0b10000000) > 0,
            flip_vertical: (self.vram[sprite_addr + 4] & 0b00010000) > 0,
            flip_horizontal: (self.vram[sprite_addr + 4] & 0b00001000) > 0,
            palette_line: (self.vram[sprite_addr + 4] >> 5) & 0b11,
            next: self.vram[sprite_addr + 3] as usize,
        }
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
        let brightness_vals = if shadow {
            BRIGHTNESS_VALS_SHADOW
        } else if highlight {
            BRIGHTNESS_VALS_HIGHLIGHT
        } else {
            BRIGHTNESS_VALS
        };
        [
            brightness_vals[((palette_val_low & 0xF) / 2) as usize],
            brightness_vals[((palette_val_low >> 4) / 2) as usize],
            brightness_vals[(palette_val_high / 2) as usize],
            0xff,
        ]
    }

    fn draw_dump_pixel(&mut self, x: u16, y: u16, width: u16) {
        let pixel = if y < 8 {
            self.get_color((y / 2) as u8, (x / (width / 16)) as u8, false, false)
        } else {
            let tile_index = (y / 8 - 1) * width / 8 + x / 8;
            let tile_x = x % 8;
            let tile_y = y % 8;
            let tile_addr = tile_index as usize * 0x20;
            let pixel_addr = tile_addr + (tile_y as usize * 8) / 2 + tile_x as usize / 2;
            let pixel_data = self.vram[pixel_addr];
            let palette_color = if tile_x % 2 == 1 {
                pixel_data & 0xF
            } else {
                pixel_data >> 4
            };
            self.get_color(0, palette_color, false, false)
        };
        self.image_buffers[0].input_buffer()
            [y as usize * 320 as usize + ((320 - width) / 2) as usize + x as usize] = pixel;
    }

    fn dump_sprite_table(&self, sprite_table_addr: usize) {
        debug!(target: "vdp", "Sprite table:");
        for sprite_index in 0..128 {
            let sprite_addr = sprite_table_addr + sprite_index * 8;

            let sprite = self.read_sprite(sprite_addr);
            debug!(target: "vdp", "{:?}", sprite);
        }
    }

    pub fn render(
        &mut self,
        c: Context,
        texture_ctx: &mut G2dTextureContext,
        gl: &mut G2d,
        device: &mut Device,
        layers: usize,
    ) {
        self.renderer
            .render(c, texture_ctx, gl, device, 1.0, layers);
    }

    pub fn close(&mut self) {
        for buf in &mut self.image_buffers {
            buf.publish();
        }
        self.renderer.close();
    }
}
