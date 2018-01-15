pub mod bus;

use std::cell::RefCell;
use graphics::*;
use image::{GenericImage, DynamicImage, Rgba};
use piston::input::RenderArgs;
use opengl_graphics::*;

use self::bus::*;
use cartridge::CartridgeBus;

const NES_RGB: [[u8; 4]; 64] =
    [[0x7C, 0x7C, 0x7C, 0xFF], [0x00, 0x00, 0xFC, 0xFF], [0x00, 0x00, 0xBC, 0xFF], [0x44, 0x28, 0xBC, 0xFF], [0x94, 0x00, 0x84, 0xFF], [0xA8, 0x00, 0x20, 0xFF], [0xA8, 0x10, 0x00, 0xFF], [0x88, 0x14, 0x00, 0xFF],
        [0x50, 0x30, 0x00, 0xFF], [0x00, 0x78, 0x00, 0xFF], [0x00, 0x68, 0x00, 0xFF], [0x00, 0x58, 0x00, 0xFF], [0x00, 0x40, 0x58, 0xFF], [0x00, 0x00, 0x00, 0xFF], [0x00, 0x00, 0x00, 0xFF], [0x00, 0x00, 0x00, 0xFF],
        [0xBC, 0xBC, 0xBC, 0xFF], [0x00, 0x78, 0xF8, 0xFF], [0x00, 0x58, 0xF8, 0xFF], [0x68, 0x44, 0xFC, 0xFF], [0xD8, 0x00, 0xCC, 0xFF], [0xE4, 0x00, 0x58, 0xFF], [0xF8, 0x38, 0x00, 0xFF], [0xE4, 0x5C, 0x10, 0xFF],
        [0xAC, 0x7C, 0x00, 0xFF], [0x00, 0xB8, 0x00, 0xFF], [0x00, 0xA8, 0x00, 0xFF], [0x00, 0xA8, 0x44, 0xFF], [0x00, 0x88, 0x88, 0xFF], [0x00, 0x00, 0x00, 0xFF], [0x00, 0x00, 0x00, 0xFF], [0x00, 0x00, 0x00, 0xFF],
        [0xF8, 0xF8, 0xF8, 0xFF], [0x3C, 0xBC, 0xFC, 0xFF], [0x68, 0x88, 0xFC, 0xFF], [0x98, 0x78, 0xF8, 0xFF], [0xF8, 0x78, 0xF8, 0xFF], [0xF8, 0x58, 0x98, 0xFF], [0xF8, 0x78, 0x58, 0xFF], [0xFC, 0xA0, 0x44, 0xFF],
        [0xF8, 0xB8, 0x00, 0xFF], [0xB8, 0xF8, 0x18, 0xFF], [0x58, 0xD8, 0x54, 0xFF], [0x58, 0xF8, 0x98, 0xFF], [0x00, 0xE8, 0xD8, 0xFF], [0x78, 0x78, 0x78, 0xFF], [0x00, 0x00, 0x00, 0xFF], [0x00, 0x00, 0x00, 0xFF],
        [0xFC, 0xFC, 0xFC, 0xFF], [0xA4, 0xE4, 0xFC, 0xFF], [0xB8, 0xB8, 0xF8, 0xFF], [0xD8, 0xB8, 0xF8, 0xFF], [0xF8, 0xB8, 0xF8, 0xFF], [0xF8, 0xA4, 0xC0, 0xFF], [0xF0, 0xD0, 0xB0, 0xFF], [0xFC, 0xE0, 0xA8, 0xFF],
        [0xF8, 0xD8, 0x78, 0xFF], [0xD8, 0xF8, 0x78, 0xFF], [0xB8, 0xF8, 0xB8, 0xFF], [0xB8, 0xF8, 0xD8, 0xFF], [0x00, 0xFC, 0xFC, 0xFF], [0xF8, 0xD8, 0xF8, 0xFF], [0x00, 0x00, 0x00, 0xFF], [0x00, 0x00, 0x00, 0xFF]];

pub struct Ppu<'a> {
    image: DynamicImage,
    texture: Texture,
    scanline: u16,
    dot: u16,

    vram_addr: u16,
    tmp_vram_addr: u16,
    fine_x_scroll: u8,

    odd_frame: bool,

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

    internal_ram: Box<[u8]>,
    palette_ram: Box<[u8]>,
    oam_ram: Box<[u8]>,
    cartridge: &'a mut Box<CartridgeBus>,

    bus: &'a RefCell<PpuBus>,
}

impl<'a> Ppu<'a> {
    pub fn new<'b>(cartridge: &'b mut Box<CartridgeBus>, bus: &'b RefCell<PpuBus>) -> Ppu<'b> {
        let image = DynamicImage::new_rgba8(256, 240);
        let texture = Texture::from_image(image.as_rgba8().unwrap(), &TextureSettings::new());
        Ppu {
            image,
            texture,
            scanline: 0,
            dot: 0,
            vram_addr: 0,
            tmp_vram_addr: 0,
            fine_x_scroll: 0,
            odd_frame: false,
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
            internal_ram: vec![0; 0x800].into_boxed_slice(),
            palette_ram: vec![0; 0x20].into_boxed_slice(),
            oam_ram: vec![0; 0x100].into_boxed_slice(),
            cartridge,
            bus,
        }
    }

    fn rendering(&self) -> bool {
        let mask = &self.bus.borrow().mask;
        mask.show_bgd || mask.show_sprite
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
        if let Some(addr) = bus.addr.take() {
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
        if let Some(data) = bus.data.take() {
            let addr = self.vram_addr;
            self.write_memory(addr, data);
            self.vram_addr += if bus.ctrl.address_increment_vertical { 32 } else { 1 };
        }
    }

    fn read_memory(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x1FFF => self.cartridge.read_memory(address),
            0x2000 ... 0x2FFF => self.internal_ram[self.cartridge.mirror_nametable(address) as usize],
            0x3000 ... 0x3EFF => self.internal_ram[(self.cartridge.mirror_nametable(address - 0x1000)) as usize],
            0x3F00 ... 0x3FFF => {
                let mut palette_address = address;
                if palette_address & 0x13 == 0x10 {
                    palette_address &= !0x10;
                }
                self.palette_ram[(palette_address % 0x20) as usize] & (if self.bus.borrow().mask.grayscale { 0x30 } else { 0xFF })
            }
            _ => panic!("Bad PPU memory read {:04X}", address),
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ... 0x1FFF => self.cartridge.write_memory(address, value),
            0x2000 ... 0x2FFF => self.internal_ram[self.cartridge.mirror_nametable(address) as usize] = value,
            0x3000 ... 0x3EFF => self.internal_ram[(self.cartridge.mirror_nametable(address - 0x1000)) as usize] = value,
            0x3F00 ... 0x3FFF => self.palette_ram[(address % 0x20) as usize] = value,
            _ => panic!("Bad PPU memory write {:04X}", address),
        }
    }

    pub fn tick(&mut self, instrument: bool) {
        if instrument {
            debug!("{}x{} V:{:04X} T:{:04X} fX:{} nt:{:04X} at:{:02X}{:02X} bg:{:02X}{:02X}",
                   self.scanline, self.dot, self.vram_addr, self.tmp_vram_addr, self.fine_x_scroll,
                   self.nametable, self.shift_attrtable_high, self.shift_attrtable_low,
                   self.shift_bgd_high, self.shift_bgd_low);
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
        if self.dot == 341 || (self.odd_frame && self.rendering() && self.dot == 340 && self.scanline == 261) {
            self.dot = 0;
            self.scanline += 1;
            self.scanline %= 262;
            self.odd_frame = !self.odd_frame;
        }
    }

    pub fn dots_per_frame(&self) -> u32 {
        341 * 262 - if self.odd_frame { 1 } else { 0 }
    }

    fn reload_shift(&mut self) {
        self.shift_bgd_low = (self.shift_bgd_low & 0xFF00) | u16::from(self.latch_bgd_low);
        self.shift_bgd_high = (self.shift_bgd_high & 0xFF00) | u16::from(self.latch_bgd_high);

        self.attrtable_latch_low = self.latch_attrtable & 1 > 0;
        self.attrtable_latch_high = self.latch_attrtable & 2 > 0;
    }

    fn draw_pixel(&mut self) {
        let mut palette: u16 = 0;
        if self.scanline < 240 && self.dot >= 2 && self.dot < 258 {
            let mask = &self.bus.borrow().mask;
            if mask.show_bgd && (mask.show_bgd_left8 || self.dot >= 10) {
                palette = (((self.shift_bgd_high >> (15 - self.fine_x_scroll)) & 1) << 1) | ((self.shift_bgd_low >> (15 - self.fine_x_scroll)) & 1);
                if palette > 0 {
                    palette |= u16::from((((self.shift_attrtable_high >> (7 - self.fine_x_scroll)) & 1) << 3) | (((self.shift_attrtable_low >> (7 - self.fine_x_scroll)) & 1) << 2));
                }
            }
            // TODO sprite
            let color = self.read_memory(0x3F00 + if self.rendering() { palette } else { 0 });
            self.image.put_pixel(u32::from(self.dot) - 2, u32::from(self.scanline), Rgba(NES_RGB[color as usize]))
        }
        self.shift_bgd_low <<= 1;
        self.shift_bgd_high <<= 1;
        self.shift_attrtable_low = (self.shift_attrtable_low << 1) + if self.attrtable_latch_low { 1 } else { 0 };
        self.shift_attrtable_high = (self.shift_attrtable_high << 1) + if self.attrtable_latch_high { 1 } else { 0 };
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
            let fine_y = self.vram_addr & 0x7000 >> 12;
            if fine_y < 7 {
                self.vram_addr = (self.vram_addr & (!0x7000)) | ((fine_y + 1) << 12);
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
        match self.dot % 8 {
            1 => {
                self.addr = 0x2000 | (self.vram_addr & 0xFFF);
                if self.dot != 321 {
                    self.reload_shift();
                }
            }
            2 => {
                self.nametable = self.read_memory(self.addr);
            }
            3 => {
                self.addr = 0x23C0 | (self.vram_addr & 0x0C00) | ((self.vram_addr >> 4) & 0x38) | ((self.vram_addr >> 2) & 0x07);
            }
            4 => {
                self.latch_attrtable = self.read_memory(self.addr);
                // TODO needs to be adjusted?
            }
            5 => {
                self.addr = if self.bus.borrow().ctrl.bgd_pattern_table_high { 0x1000 } else { 0 } +
                    u16::from(self.nametable) * 16 + (self.vram_addr & 0x7000 >> 12);
            }
            6 => {
                self.latch_bgd_low = self.read_memory(self.addr);
            }
            7 => {
                self.addr += 8;
            }
            0 => {
                self.latch_bgd_high = self.read_memory(self.addr);
                self.scroll_horizontal();
            }
            _ => panic!("bad dot"),
        }
        if self.dot == 256 {
            self.scroll_vertical();
        }
    }

    fn tick_render(&mut self) {
        // TODO sprites
        match self.dot {
            2 ... 256 => {
                self.draw_pixel();
                self.read_into_latches();
            }
            257 => {
                self.draw_pixel();
                self.reload_shift();
                self.update_horizontal();
            }
            321 ... 337 => {
                self.read_into_latches();
            }
            338 | 340 => {
                self.nametable = self.read_memory(self.addr);
            }
            1 | 339 => {
                self.addr = 0x2000 | (self.vram_addr & 0xFFF);
            }
            _ => (),
        }
        // TODO signal scanline to mapper
    }

    fn tick_post_render(&mut self) {
        if self.dot == 0 {
            self.texture.update(self.image.as_rgba8().unwrap());
        }
    }

    fn tick_vblank(&mut self) {
        if self.scanline == 241 && self.dot == 1 {
            let mut bus = self.bus.borrow_mut();
            bus.status.vertical_blank = true;
            if bus.ctrl.gen_nmi {
                bus.nmi_interrupt = true;
            }
        }
    }

    fn tick_prerender(&mut self) {
        if self.dot == 1 {
            self.bus.borrow_mut().status.vertical_blank = false;
        }
        self.tick_render();
        if self.dot >= 280 && self.dot <= 304 {
            self.update_vertical();
        }
    }

    pub fn render(&self, gl: &mut GlGraphics, r: RenderArgs) {
        gl.draw(r.viewport(), |c, gl| {
            image(&self.texture, c.transform, gl);
        })
    }
}