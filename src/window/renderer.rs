use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

use image::{DynamicImage, GenericImage};
use piston_window::*;
use triple_buffer::Output;

use gfx_device_gl::Device;

pub struct Renderer {
    image: Arc<Mutex<DynamicImage>>,
    texture: Option<G2dTexture>,
    join_handle: Option<JoinHandle<()>>,
    closed: Arc<AtomicBool>,
}

impl Renderer {
    pub fn new<P: Send + 'static, const N: usize, W: Window>(
        window: Option<&mut PistonWindow<W>>,
        mut image_buffer_out: Output<Box<[P; N]>>,
        width: u32,
        fill: fn(&mut Output<Box<[P; N]>>, &mut DynamicImage),
    ) -> Renderer {
        let height = (N as u32) / width;
        let image = Arc::new(Mutex::new(DynamicImage::new_rgba8(width, height)));
        let image_clone = image.clone();
        let texture = window.map(|window| {
            G2dTexture::from_image(
                &mut window.create_texture_context(),
                image.lock().unwrap().as_rgba8().unwrap(),
                &TextureSettings::new(),
            )
                .unwrap()
        });
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = closed.clone();

        let join_handle = thread::spawn(move || {
            let mut image = DynamicImage::new_rgba8(width, height);
            loop {
                image_buffer_out.update();
                if closed_clone.load(Ordering::Relaxed) {
                    break;
                }
                fill(&mut image_buffer_out, &mut image);
                image_clone.lock().unwrap().copy_from(&image, 0, 0).unwrap();
            }
        });

        Renderer {
            image,
            texture,
            join_handle: Some(join_handle),
            closed,
        }
    }

    pub fn render(
        &mut self,
        c: Context,
        mut texture_ctx: &mut G2dTextureContext,
        gl: &mut G2d,
        device: &mut Device,
        x_scale: f64,
    ) {
        if let Some(ref mut texture) = self.texture {
            texture
                .update(
                    &mut texture_ctx,
                    self.image.lock().unwrap().as_rgba8().unwrap(),
                )
                .unwrap();
            image(texture, c.transform.scale(x_scale, 1.0), gl);
            texture_ctx.encoder.flush(device);
        }
    }

    pub fn close(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
        self.join_handle.take().unwrap().join().unwrap();
    }
}
