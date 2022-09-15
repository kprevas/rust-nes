use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

use gfx_device_gl::Device;
use image::{DynamicImage, GenericImage};
use piston_window::*;
use triple_buffer::Output;

pub struct Renderer<const L: usize> {
    background: [f32; 4],
    images: [Arc<Mutex<DynamicImage>>; L],
    textures: Option<[G2dTexture; L]>,
    join_handle: Option<JoinHandle<()>>,
    closed: Arc<AtomicBool>,
}

impl<const L: usize> Renderer<L> {
    pub fn new<P: Send + 'static, const N: usize, W: Window>(
        window: Option<&mut PistonWindow<W>>,
        mut image_buffer_outs: [Output<Box<[P; N]>>; L],
        width: u32,
        fill: fn(&mut Output<Box<[P; N]>>, &mut DynamicImage),
    ) -> Renderer<L> {
        let height = (N as u32) / width;
        let images = [0; L].map(|_| Arc::new(Mutex::new(DynamicImage::new_rgba8(width, height))));
        let image_clones = images.each_ref().map(|image| image.clone());
        let textures = window.map(|window| {
            [0; L].map(|i| G2dTexture::from_image(
                &mut window.create_texture_context(),
                images[i].lock().unwrap().as_rgba8().unwrap(),
                &TextureSettings::new(),
            )
                .unwrap())
        });
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = closed.clone();

        let join_handle = thread::spawn(move || {
            let mut images = [0; L].map(|_| DynamicImage::new_rgba8(width, height));
            loop {
                for buf in &mut image_buffer_outs {
                    buf.update();
                }
                if closed_clone.load(Ordering::Relaxed) {
                    break;
                }
                for i in 0..L {
                    fill(&mut image_buffer_outs[i], &mut images[i]);
                    image_clones[i].lock().unwrap().copy_from(&images[i], 0, 0).unwrap();
                }
            }
        });

        Renderer {
            background: [0.0, 0.0, 0.0, 1.0],
            images,
            textures,
            join_handle: Some(join_handle),
            closed,
        }
    }

    pub fn set_background(&mut self, background: [f32; 4]) {
        self.background = background;
    }

    pub fn render(
        &mut self,
        c: Context,
        mut texture_ctx: &mut G2dTextureContext,
        gl: &mut G2d,
        device: &mut Device,
        x_scale: f64,
        layers: usize,
    ) {
        let layers = layers % (L + 1);
        clear(if layers == 0 { self.background } else { [1.0, 0.0, 1.0, 1.0] }, gl);
        if let Some(ref mut textures) = self.textures {
            for (i, texture) in textures.iter_mut().enumerate() {
                if layers == 0 || layers - 1 == i {
                    texture
                        .update(
                            &mut texture_ctx,
                            self.images[i].lock().unwrap().as_rgba8().unwrap(),
                        )
                        .unwrap();
                    image(texture, c.transform.scale(x_scale, 1.0), gl);
                }
            }
            texture_ctx.encoder.flush(device);
        }
    }

    pub fn close(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
        self.join_handle.take().unwrap().join().unwrap();
    }
}
