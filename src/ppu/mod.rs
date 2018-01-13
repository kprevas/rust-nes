use graphics::*;
use image::{GenericImage, DynamicImage, Rgba};
use piston::input::RenderArgs;
use opengl_graphics::*;

use cartridge::CartridgeBus;

pub struct Ppu<'a> {
    image: DynamicImage,
    texture: Texture,
    scanline: u16,
    dot: u16,
    rendering: bool,
    odd_frame: bool,
    internal_ram: Box<[u8]>,
    palette_ram: Box<[u8]>,
    oam_ram: Box<[u8]>,
    cartridge: &'a mut Box<CartridgeBus>,
}

impl<'a> Ppu<'a> {
    pub fn new(cartridge: &mut Box<CartridgeBus>) -> Ppu {
        let mut image = DynamicImage::new_rgba8(256, 240);
        for y in 0..240 {
            for x in 0..256 {
                image.put_pixel(x, y, Rgba([x as u8, y as u8, 0, 255]));
            }
        }
        let texture = Texture::from_image(image.as_rgba8().unwrap(), &TextureSettings::new());
        Ppu {
            image,
            texture,
            scanline: 0,
            dot: 0,
            rendering: false,
            odd_frame: false,
            internal_ram: vec![0; 0x800].into_boxed_slice(),
            palette_ram: vec![0; 0x20].into_boxed_slice(),
            oam_ram: vec![0; 0x100].into_boxed_slice(),
            cartridge,
        }
    }

    fn background(&self) -> types::Color {
        [1.0, 0.0, 0.0, 1.0]
    }

    fn read_memory(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x1FFF => self.cartridge.read_memory(address),
            0x2000 ... 0x2FFF => self.internal_ram[self.cartridge.mirror_nametable(address) as usize],
            0x3000 ... 0x3EFF => self.internal_ram[(self.cartridge.mirror_nametable(address - 0x1000)) as usize],
            0x3F00 ... 0x3FFF => self.palette_ram[(address % 0x20) as usize],
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

    fn tick(&mut self) {
        match self.scanline {
            0 ... 239 => self.tick_visible(),
            240 => self.tick_post_render(),
            241 ... 260 => self.tick_vblank(),
            261 => self.tick_prerender(),
            _ => panic!("Bad scanline {}", self.scanline)
        }
        self.dot += 1;
        if self.dot == 341 || (self.odd_frame && self.rendering && self.dot == 340 && self.scanline == 261) {
            self.dot = 0;
            self.scanline += 1;
            self.scanline %= 262;
        }
    }

    fn tick_visible(&mut self) {}

    fn tick_post_render(&mut self) {}

    fn tick_vblank(&mut self) {}

    fn tick_prerender(&mut self) {}

    pub fn render(&mut self, gl: &mut GlGraphics, r: RenderArgs) {
        self.texture.update(self.image.as_rgba8().unwrap());
        gl.draw(r.viewport(), |c, gl| {
            clear(self.background(), gl);
            image(&self.texture, c.transform, gl);
        })
    }
}