use bincode::{deserialize_from, serialize};
use bytes::*;
use cartridge::CartridgeBus;
use image::{DynamicImage, GenericImage, Rgba};
use piston_window::*;
use self::bus::*;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::io::Cursor;

pub mod bus;

const NES_RGB: [u8; 0x600] = *include_bytes!("ntscpalette.pal");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sprite {
    id: u8,
    x: u8,
    y: u8,
    tile: u8,
    attr: u8,
    data_low: u8,
    data_high: u8,
}

impl Default for Sprite {
    fn default() -> Sprite {
        Sprite {
            id: 64,
            x: 0xFF,
            y: 0xFF,
            tile: 0xFF,
            attr: 0xFF,
            data_low: 0,
            data_high: 0,
        }
    }
}

pub struct Ppu<'a> {
    image: DynamicImage,
    image_buffer: DynamicImage,
    texture: Option<G2dTexture>,

    scanline: u16,
    dot: u16,

    vram_addr: u16,
    tmp_vram_addr: u16,
    fine_x_scroll: u8,

    odd_frame: bool,
    skip_tick: bool,

    nametable: u8,
    latch_attrtable: u8,
    latch_bgd_low: u8,
    latch_bgd_high: u8,

    shift_attrtable_low: u8,
    shift_attrtable_high: u8,
    shift_bgd_low: u16,
    shift_bgd_high: u16,

    attrtable_latch_low: bool,
    attrtable_latch_high: bool,

    addr: u16,

    oam: [Sprite; 8],
    sec_oam: [Sprite; 8],
    sprite_overflow_tick_delay: Option<u8>,

    internal_ram: Box<[u8]>,
    palette_ram: Box<[u8]>,
    oam_ram: Box<[u8]>,
    cartridge: &'a mut Box<CartridgeBus>,

    bus: &'a RefCell<PpuBus>,

    instrumented: bool,
}

impl<'a> Ppu<'a> {
    pub fn new<'b, W: Window>(cartridge: &'b mut Box<CartridgeBus>, bus: &'b RefCell<PpuBus>, window: Option<&mut PistonWindow<W>>, instrumented: bool) -> Ppu<'b> {
        let image = DynamicImage::new_rgba8(256, 240);
        let texture = window.map(|window| {
            G2dTexture::from_image(window.factory.borrow_mut(), image.as_rgba8().unwrap(), &TextureSettings::new()).unwrap()
        });
        Ppu {
            image,
            image_buffer: DynamicImage::new_rgba8(256, 240),
            texture,
            scanline: 0,
            dot: 0,
            vram_addr: 0,
            tmp_vram_addr: 0,
            fine_x_scroll: 0,
            odd_frame: false,
            skip_tick: false,
            nametable: 0,
            latch_attrtable: 0,
            latch_bgd_low: 0,
            latch_bgd_high: 0,
            shift_attrtable_low: 0,
            shift_attrtable_high: 0,
            shift_bgd_low: 0,
            shift_bgd_high: 0,
            attrtable_latch_low: false,
            attrtable_latch_high: false,
            addr: 0,
            oam: Default::default(),
            sec_oam: Default::default(),
            sprite_overflow_tick_delay: None,
            internal_ram: vec![0; 0x800].into_boxed_slice(),
            palette_ram: vec![0; 0x20].into_boxed_slice(),
            oam_ram: vec![0; 0x100].into_boxed_slice(),
            cartridge,
            bus,
            instrumented,
        }
    }

    fn rendering(&self) -> bool {
        let mask = &self.bus.borrow().mask;
        mask.show_bgd || mask.show_sprite
    }

    fn spr_height(&self) -> u8 {
        if self.bus.borrow().ctrl.sprite_size_large { 16 } else { 8 }
    }

    fn update_tmp_addr(&mut self) {
        let mut bus = self.bus.borrow_mut();
        if let Some(nametable_address) = bus.ctrl.nametable_select.take() {
            self.tmp_vram_addr = (self.tmp_vram_addr & (!0xC00)) | (u16::from(nametable_address) << 10);
        }
        if let Some(scroll) = bus.scroll.take() {
            if bus.first_write {
                self.fine_x_scroll = scroll & 0x7;
                self.tmp_vram_addr = (self.tmp_vram_addr & (!0x1F)) | (u16::from(scroll) >> 3);
            } else {
                self.tmp_vram_addr = (self.tmp_vram_addr & 0xC1F) | ((u16::from(scroll) & 0x7) << 12)
                    | ((u16::from(scroll) >> 3) << 5);
            }
        }
        if let Some(addr) = bus.addr_write.take() {
            if bus.first_write {
                self.tmp_vram_addr = (self.tmp_vram_addr & 0xFF) | ((u16::from(addr) & 0x3F) << 8);
            } else {
                self.tmp_vram_addr = (self.tmp_vram_addr & (!0xFF)) | u16::from(addr);
                self.vram_addr = self.tmp_vram_addr;
            }
        }
    }

    fn process_data_write(&mut self) {
        let mut bus = self.bus.borrow_mut();
        if let Some(data) = bus.data_write.take() {
            let addr = self.vram_addr;
            self.write_memory(addr, data);
            self.vram_addr += if bus.ctrl.address_increment_vertical { 32 } else { 1 };
        }
        if let Some(mut data) = bus.oam_data_write.take() {
            let addr = bus.oam_addr;
            if addr & 0x3 == 0x2 {
                data &= 0xE3;
            }
            self.oam_ram[addr as usize] = data;
            bus.oam_addr = bus.oam_addr.wrapping_add(1);
        }
    }

    fn read_memory(&self, address: u16, grayscale: bool) -> u8 {
        match address {
            0x0000 ... 0x1FFF => self.cartridge.read_memory(address, 0),
            0x2000 ... 0x2FFF => self.internal_ram[self.cartridge.mirror_nametable(address) as usize],
            0x3000 ... 0x3EFF => self.internal_ram[(self.cartridge.mirror_nametable(address - 0x1000)) as usize],
            0x3F00 ... 0x3FFF => {
                let mut palette_address = address;
                if palette_address & 0x13 == 0x10 {
                    palette_address &= !0x10;
                }
                self.palette_ram[(palette_address % 0x20) as usize] & (if grayscale { 0x30 } else { 0xFF })
            }
            _ => panic!("Bad PPU memory read {:04X}", address),
        }
    }

    fn read_memory_under_palette(&self, address: u16) -> u8 {
        self.internal_ram[(self.cartridge.mirror_nametable(address - 0x1000)) as usize]
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ... 0x1FFF => self.cartridge.write_memory(address, value, 0),
            0x2000 ... 0x2FFF => self.internal_ram[self.cartridge.mirror_nametable(address) as usize] = value,
            0x3000 ... 0x3EFF => self.internal_ram[(self.cartridge.mirror_nametable(address - 0x1000)) as usize] = value,
            0x3F00 ... 0x3FFF => {
                let mut palette_address = address;
                if palette_address & 0x13 == 0x10 {
                    palette_address &= !0x10;
                }
                self.palette_ram[(palette_address % 0x20) as usize] = value
            }
            _ => panic!("Bad PPU memory write {:04X}", address),
        }
    }

    pub fn tick(&mut self) {
        if self.instrumented {
            debug!(target: "ppu", "{}x{} V:{:04X} T:{:04X} fX:{} nt:{:04X} at:{:02X}{:02X} bg:{:04X} {:04X}",
                   self.scanline, self.dot, self.vram_addr, self.tmp_vram_addr, self.fine_x_scroll,
                   self.nametable, self.shift_attrtable_high, self.shift_attrtable_low,
                   self.shift_bgd_high, self.shift_bgd_low);
        }
        {
            let mut bus = self.bus.borrow_mut();
            if bus.nmi_interrupt && bus.nmi_interrupt_age < 255 {
                bus.nmi_interrupt_age += 1;
            }
            if self.vram_addr >= 0x3F00 && self.vram_addr < 0x4000 {
                bus.palette_data = self.read_memory(self.vram_addr, bus.mask.grayscale);
                if bus.read_buffer.is_none() {
                    bus.read_buffer = Some(self.read_memory_under_palette(self.vram_addr));
                    self.vram_addr += if bus.ctrl.address_increment_vertical { 32 } else { 1 };
                }
            } else {
                if bus.read_buffer.is_none() {
                    bus.read_buffer = Some(self.read_memory(self.vram_addr, bus.mask.grayscale));
                    self.vram_addr += if bus.ctrl.address_increment_vertical { 32 } else { 1 };
                }
            }
            bus.oam_data = self.oam_ram[bus.oam_addr as usize];
        }
        self.update_tmp_addr();
        self.process_data_write();
        match self.scanline {
            0 ... 239 => self.tick_render(),
            240 => self.tick_post_render(),
            241 ... 260 => self.tick_vblank(),
            261 => self.tick_prerender(),
            _ => panic!("Bad scanline {}", self.scanline)
        }
        self.dot += 1;
        if self.dot == 341 || (self.skip_tick && self.dot == 340) {
            self.dot = 0;
            self.scanline += 1;
            self.scanline %= 262;
            if self.scanline == 0 {
                self.odd_frame = !self.odd_frame;
            }
        }
        self.skip_tick = self.scanline == 261 && self.dot == 339 && self.odd_frame && self.rendering();
        let mut bus = self.bus.borrow_mut();
        bus.status.just_read = false;
        bus.addr = self.vram_addr;
    }

    fn reload_shift(&mut self) {
        self.shift_bgd_low = (self.shift_bgd_low & 0xFF00) | u16::from(self.latch_bgd_low);
        self.shift_bgd_high = (self.shift_bgd_high & 0xFF00) | u16::from(self.latch_bgd_high);

        self.attrtable_latch_low = self.latch_attrtable & 1 > 0;
        self.attrtable_latch_high = self.latch_attrtable & 2 > 0;
    }

    fn draw_pixel(&mut self) {
        // TODO: color emphasis
        let mut palette: u16 = 0;
        let mut obj_palette: u16 = 0;
        let mut obj_priority = false;
        if self.scanline < 240 && self.dot >= 2 && self.dot < 258 {
            let show_bgd;
            let show_bgd_left8;
            let show_sprite;
            let show_sprite_left8;
            {
                let mask = &self.bus.borrow().mask;
                show_bgd = mask.show_bgd;
                show_bgd_left8 = mask.show_bgd_left8;
                show_sprite = mask.show_sprite;
                show_sprite_left8 = mask.show_sprite_left8;
            }
            if show_bgd && (show_bgd_left8 || self.dot >= 10) {
                palette = (((self.shift_bgd_high >> (15 - self.fine_x_scroll)) & 1) << 1)
                    | ((self.shift_bgd_low >> (15 - self.fine_x_scroll)) & 1);
                if palette > 0 {
                    palette |= u16::from((((self.shift_attrtable_high >> (7 - self.fine_x_scroll)) & 1) << 3)
                        | (((self.shift_attrtable_low >> (7 - self.fine_x_scroll)) & 1) << 2));
                }
            }
            if show_sprite && (show_sprite_left8 || self.dot >= 10) {
                for sprite in self.oam.iter() {
                    if sprite.id != 64 && u16::from(sprite.x) <= self.dot - 2 {
                        let mut sprite_x = self.dot - 2 - u16::from(sprite.x);
                        if sprite_x < 8 {
                            if sprite.attr & 0x40 > 0 {
                                sprite_x ^= 7;
                            }

                            let mut sprite_palette = (((sprite.data_high >> (7 - sprite_x)) & 1) << 1)
                                | ((sprite.data_low >> (7 - sprite_x)) & 1);
                            if sprite_palette > 0 {
                                if sprite.id == 0 && palette > 0 && self.dot != 257 {
                                    self.bus.borrow_mut().status.sprite_0_hit = true;
                                }
                                sprite_palette |= (sprite.attr & 3) << 2;
                                obj_palette = u16::from(sprite_palette) + 16;
                                obj_priority = sprite.attr & 0x20 > 0;
                            }
                        }
                    }
                }
                if obj_palette > 0 && (palette == 0 || !obj_priority) {
                    palette = obj_palette;
                }
            }
            let bus = self.bus.borrow();
            let color = self.read_memory(0x3F00 + if self.rendering() { palette } else { 0 }, bus.mask.grayscale);
            let color_index = (0xc0 * bus.mask.color_emphasis + color * 3) as usize;
            self.image.put_pixel(u32::from(self.dot) - 2, u32::from(self.scanline),
                                 Rgba([NES_RGB[color_index], NES_RGB[color_index + 1], NES_RGB[color_index + 2], 0xff]));
        }
        self.adjust_shifts();
    }

    fn adjust_shifts(&mut self) {
        self.shift_bgd_low <<= 1;
        self.shift_bgd_high <<= 1;
        self.shift_attrtable_low = (self.shift_attrtable_low << 1) | if self.attrtable_latch_low { 1 } else { 0 };
        self.shift_attrtable_high = (self.shift_attrtable_high << 1) | if self.attrtable_latch_high { 1 } else { 0 };
    }

    fn scroll_horizontal(&mut self) {
        if self.rendering() {
            if (self.vram_addr & 0x001F) == 31 {
                self.vram_addr ^= 0x041F;
            } else {
                self.vram_addr += 1;
            }
        }
    }

    fn scroll_vertical(&mut self) {
        if self.rendering() {
            let fine_y = (self.vram_addr & 0x7000) >> 12;
            if fine_y < 7 {
                self.vram_addr += 0x1000;
            } else {
                self.vram_addr &= !0x7000;
                let mut coarse_y = (self.vram_addr & 0x3E0) >> 5;
                if coarse_y == 29 {
                    coarse_y = 0;
                    self.vram_addr ^= 0x800;
                } else if coarse_y == 31 {
                    coarse_y = 0;
                } else {
                    coarse_y += 1;
                }
                self.vram_addr = (self.vram_addr & (!0x3E0)) | (coarse_y << 5);
            }
        }
    }

    fn update_horizontal(&mut self) {
        if self.rendering() {
            self.vram_addr = (self.vram_addr & (!0x41F)) | (self.tmp_vram_addr & 0x41F);
        }
    }

    fn update_vertical(&mut self) {
        if self.rendering() {
            self.vram_addr = (self.vram_addr & (!0x7BE0)) | (self.tmp_vram_addr & 0x7BE0);
        }
    }

    fn read_into_latches(&mut self) {
        let bus = self.bus.borrow();
        match self.dot % 8 {
            1 => {
                self.addr = 0x2000 | (self.vram_addr & 0xFFF);
                self.reload_shift();
            }
            2 => {
                self.nametable = self.read_memory(self.addr, bus.mask.grayscale);
            }
            3 => {
                self.addr = 0x23C0 | (self.vram_addr & 0x0C00) | ((self.vram_addr >> 4) & 0x38) | ((self.vram_addr >> 2) & 0x07);
            }
            4 => {
                self.latch_attrtable = self.read_memory(self.addr, bus.mask.grayscale);
                if ((self.vram_addr >> 5) & 2) > 0 {
                    self.latch_attrtable >>= 4;
                }
                if (self.vram_addr & 2) > 0 {
                    self.latch_attrtable >>= 2;
                }
            }
            5 => {
                self.addr = if self.bus.borrow().ctrl.bgd_pattern_table_high { 0x1000 } else { 0 } +
                    u16::from(self.nametable) * 16 + ((self.vram_addr & 0x7000) >> 12);
            }
            6 => {
                self.latch_bgd_low = self.read_memory(self.addr, bus.mask.grayscale);
            }
            7 => {
                self.addr += 8;
            }
            0 => {
                self.latch_bgd_high = self.read_memory(self.addr, bus.mask.grayscale);
                self.scroll_horizontal();
            }
            _ => panic!("bad dot"),
        }
        if self.dot == 256 {
            self.scroll_vertical();
        }
    }

    fn clear_oam(&mut self) {
        for sprite in self.sec_oam.iter_mut() {
            sprite.id = 64;
            sprite.x = 0xFF;
            sprite.y = 0xFF;
            sprite.tile = 0xFF;
            sprite.attr = 0xFF;
            sprite.data_low = 0;
            sprite.data_high = 0;
        }
    }

    fn eval_sprites(&mut self) {
        let mut sprite_index = 0;
        let mut overflow_bug_offset = 0;
        let mut overflow_tick = 0;
        for i in 0..64 {
            let sprite_start = (i * 4) as usize + overflow_bug_offset;
            let sprite_y = u16::from(self.oam_ram[sprite_start]);
            overflow_tick += 2;
            let mut in_range = false;
            if sprite_y <= self.scanline {
                let line = self.scanline - sprite_y;
                if line < u16::from(self.spr_height()) {
                    in_range = true;
                }
            }
            if in_range {
                if sprite_index == 8 {
                    self.sprite_overflow_tick_delay = Some(overflow_tick);
                    break;
                } else {
                    let sprite = &mut self.sec_oam[sprite_index];
                    sprite.id = i;
                    sprite.y = self.oam_ram[sprite_start];
                    sprite.tile = self.oam_ram[sprite_start + 1];
                    sprite.attr = self.oam_ram[sprite_start + 2];
                    sprite.x = self.oam_ram[sprite_start + 3];

                    overflow_tick += 6;
                    sprite_index += 1;
                }
            } else if sprite_index == 8 {
                overflow_bug_offset += 1;
                if overflow_bug_offset == 4 {
                    overflow_bug_offset = 0;
                }
            }
        }
    }

    fn load_sprites(&mut self) {
        for (i, sprite) in self.sec_oam.iter().enumerate() {
            let mut sprite = sprite.clone();
            let mut addr: u16;
            let bus = self.bus.borrow();
            if bus.ctrl.sprite_size_large {
                addr = (u16::from(sprite.tile & 1) * 0x1000) + (u16::from(sprite.tile & (!1)) * 16);
            } else {
                addr = if bus.ctrl.sprite_pattern_table_high { 0x1000 } else { 0 } + u16::from(sprite.tile) * 16;
            }
            if self.scanline >= u16::from(sprite.y) {
                let mut sprite_y = (self.scanline - u16::from(sprite.y)) % u16::from(self.spr_height());
                if sprite.attr & 0x80 > 0 {
                    sprite_y ^= u16::from(self.spr_height()) - 1;
                }
                addr += sprite_y + (sprite_y & 8);

                sprite.data_low = self.read_memory(addr, bus.mask.grayscale);
                sprite.data_high = self.read_memory(addr + 8, bus.mask.grayscale);
            }
            self.oam[i] = sprite;
        }
    }

    fn tick_render(&mut self) {
        match self.dot {
            1 => {
                self.clear_oam();
                if self.scanline == 261 {
                    let mut bus = self.bus.borrow_mut();
                    bus.status.sprite_0_hit = false;
                    bus.status.sprite_overflow = false;
                }
            }
            65 => {
                if self.scanline != 261 {
                    self.eval_sprites();
                }
            }
            321 => {
                self.load_sprites();
            }
            _ => (),
        }
        if let Some(val) = self.sprite_overflow_tick_delay {
            if val == 0 {
                if self.rendering() {
                    self.bus.borrow_mut().status.sprite_overflow = true;
                }
                self.sprite_overflow_tick_delay = None;
            } else {
                self.sprite_overflow_tick_delay = Some(val - 1);
            }
        }
        match self.dot {
            1 ... 256 => {
                if self.dot >= 2 {
                    self.draw_pixel();
                }
                self.read_into_latches();
            }
            257 => {
                self.draw_pixel();
                self.update_horizontal();
            }
            321 ... 336 => {
                self.read_into_latches();
                self.adjust_shifts();
            }
            337 | 339 => {
                self.nametable = self.read_memory(self.addr, self.bus.borrow().mask.grayscale);
            }
            _ => (),
        }
        // TODO signal scanline to mapper
    }

    fn tick_post_render(&mut self) {
        if self.dot == 0 {
            self.image_buffer.copy_from(&self.image, 0, 0);
        }
    }

    fn tick_vblank(&mut self) {
        if self.scanline == 241 && self.dot == 1 {
            let mut bus = self.bus.borrow_mut();
            if !bus.status.just_read {
                bus.status.vertical_blank = true;
                if bus.ctrl.gen_nmi {
                    bus.nmi_interrupt = true;
                    bus.nmi_interrupt_age = 0;
                }
            }
        }
    }

    fn tick_prerender(&mut self) {
        self.bus.borrow_mut().status.vertical_blank_about_to_clear = self.dot == 0;
        if self.dot == 1 {
            self.bus.borrow_mut().status.vertical_blank = false;
        }
        self.tick_render();
        if self.dot >= 280 && self.dot <= 304 {
            self.update_vertical();
        }
    }

    pub fn render(&mut self, c: Context, gl: &mut G2d, _glyphs: &mut Glyphs) {
        if let Some(ref mut texture) = self.texture {
            texture.update(gl.encoder, self.image_buffer.as_rgba8().unwrap()).unwrap();
            image(texture, c.transform.scale(8.0 / 7.0, 1.0), gl);
        }
    }

    pub fn save_state(&self, out: &mut Vec<u8>) {
        out.put_u16::<BigEndian>(self.scanline);
        out.put_u16::<BigEndian>(self.dot);
        out.put_u16::<BigEndian>(self.vram_addr);
        out.put_u16::<BigEndian>(self.tmp_vram_addr);
        out.put_u8(self.fine_x_scroll);
        out.put_u8(if self.odd_frame { 1 } else { 0 });
        out.put_u8(if self.skip_tick { 1 } else { 0 });
        out.put_u8(self.nametable);
        out.put_u8(self.latch_attrtable);
        out.put_u8(self.latch_bgd_low);
        out.put_u8(self.latch_bgd_high);
        out.put_u8(self.shift_attrtable_low);
        out.put_u8(self.shift_attrtable_high);
        out.put_u16::<BigEndian>(self.shift_bgd_low);
        out.put_u16::<BigEndian>(self.shift_bgd_high);
        out.put_u8(if self.attrtable_latch_low { 1 } else { 0 });
        out.put_u8(if self.attrtable_latch_high { 1 } else { 0 });
        out.put_u16::<BigEndian>(self.addr);
        out.put_slice(&serialize(&self.oam).unwrap());
        out.put_slice(&serialize(&self.sec_oam).unwrap());
        out.put_slice(&serialize(&self.sprite_overflow_tick_delay).unwrap());
        out.put_slice(&self.internal_ram);
        out.put_slice(&self.palette_ram);
        out.put_slice(&self.oam_ram);
        self.cartridge.save_state(out);
    }

    pub fn load_state(&mut self, state: &mut Cursor<Vec<u8>>) {
        self.scanline = state.get_u16::<BigEndian>();
        self.dot = state.get_u16::<BigEndian>();
        self.vram_addr = state.get_u16::<BigEndian>();
        self.tmp_vram_addr = state.get_u16::<BigEndian>();
        self.fine_x_scroll = state.get_u8();
        self.odd_frame = state.get_u8() == 1;
        self.skip_tick = state.get_u8() == 1;
        self.nametable = state.get_u8();
        self.latch_attrtable = state.get_u8();
        self.latch_bgd_low = state.get_u8();
        self.latch_bgd_high = state.get_u8();
        self.shift_attrtable_low = state.get_u8();
        self.shift_attrtable_high = state.get_u8();
        self.shift_bgd_low = state.get_u16::<BigEndian>();
        self.shift_bgd_high = state.get_u16::<BigEndian>();
        self.attrtable_latch_low = state.get_u8() == 1;
        self.attrtable_latch_high = state.get_u8() == 1;
        self.addr = state.get_u16::<BigEndian>();
        self.oam = deserialize_from(state.reader()).unwrap();
        self.sec_oam = deserialize_from(state.reader()).unwrap();
        self.sprite_overflow_tick_delay = deserialize_from(state.reader()).unwrap();
        state.copy_to_slice(&mut self.internal_ram);
        state.copy_to_slice(&mut self.palette_ram);
        state.copy_to_slice(&mut self.oam_ram);
        self.cartridge.load_state(state);
    }

    pub fn instrumentation_short(&self) -> String {
        format!("{}x{}", self.scanline, self.dot)
    }
}