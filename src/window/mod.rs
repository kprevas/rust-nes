use std::path::{Path, PathBuf};

use bytes::Buf;
use gfx_device_gl::Device;
use piston_window::*;
use sdl2_window::Sdl2Window;
use time::Instant;

use input::ControllerState;

pub mod renderer;

pub trait Cpu {
    fn reset(&mut self, soft: bool);
    fn do_frame(&mut self, time_secs: f64, inputs: &[ControllerState<8>; 2], debug: bool) -> bool;
    fn render(
        &mut self,
        c: Context,
        texture_ctx: &mut G2dTextureContext,
        gl: &mut G2d,
        device: &mut Device,
        layers: usize,
        debug: bool,
    );
    fn save_state(&self, out: &mut Vec<u8>);
    fn load_state(&mut self, state: &mut dyn Buf);
    fn increase_speed(&mut self);
    fn decrease_speed(&mut self);
}

pub fn window_loop(
    mut window: PistonWindow<Sdl2Window>,
    mut inputs: &mut [ControllerState<8>; 2],
    record_path: &PathBuf,
    cpu: &mut dyn Cpu,
    width: f64,
    height: f64,
    settings_path: &Path,
    pause: bool,
    debug: bool,
) {

    let mut frame_count = 0u32;
    let mut last_frame = Instant::now();

    let mut glyphs = Glyphs::from_bytes(
        include_bytes!("../VeraMono.ttf"),
        window.create_texture_context(),
        TextureSettings::new(),
    )
        .unwrap();
    let mut texture_ctx = window.create_texture_context();

    let mut scale = 1.0;
    let mut x_trans = 0.0;
    let mut y_trans = 0.0;

    let mut control = ::control::Control::new();
    control.pause = pause;
    control.debug_cpu = debug;

    let mut input_changed = false;

    let mut menu = ::menu::Menu::new(::menu::NES_CONTROLS, &inputs, settings_path);
    menu.update_controls(&mut inputs);

    let mut recorder = ::record::Recorder::new(&record_path);

    while let Some(e) = window.next() {
        let menu_handled = menu.event(&e);
        if !menu_handled {
            input_changed |= inputs[0].event(&e);
            input_changed |= inputs[1].event(&e);
            control.event(
                &e,
                cpu,
                &mut recorder,
                frame_count,
            );
        } else {
            menu.update_controls(&mut inputs);
        }

        if let Some(u) = e.update_args() {
            if control.reset {
                control.reset = false;
                cpu.reset(true);
            }
            if !control.pause || control.step {
                control.step = false;
                if input_changed {
                    recorder.input_changed(&inputs, frame_count);
                    input_changed = false;
                }
                recorder.set_frame_inputs(&mut inputs, frame_count);
                let brk = cpu.do_frame(if control.step { 1.0 / 60.0 } else { u.dt }, &inputs, control.debug_cpu);
                if brk {
                    control.pause = true;
                    control.debug_cpu = true;
                }
                frame_count += 1;
            }
        }

        if let Some(_r) = e.render_args() {
            window.draw_2d(&e, |c, gl, device| {
                let trans = c.trans(x_trans, y_trans).scale(scale, scale);
                cpu.render(trans, &mut texture_ctx, gl, device, control.render_layers, control.debug_video);
                recorder.render_overlay(c, gl);
                if control.input_overlay {
                    inputs[0].render_overlay(trans.trans(10.0, height - 10.0), gl, &mut glyphs);
                    inputs[1].render_overlay(trans.trans(170.0, height - 10.0), gl, &mut glyphs);
                    text(
                        [1.0, 1.0, 1.0, 1.0],
                        8,
                        &format!(
                            "{}",
                            (Instant::now() - last_frame).as_seconds_f32() * 1000.0
                        ),
                        &mut glyphs,
                        c.trans(width - 40.0, 10.0).transform,
                        gl,
                    ).unwrap();
                    text(
                        [1.0, 1.0, 1.0, 1.0],
                        8,
                        &format!("{}", frame_count),
                        &mut glyphs,
                        c.trans(width - 40.0, 30.0).transform,
                        gl,
                    ).unwrap();
                }
                menu.render(trans, gl, &mut glyphs);
                glyphs.factory.encoder.flush(device);
                last_frame = Instant::now();
            });
        }

        if let Some(r) = e.resize_args() {
            let window_width = r.draw_size[0] as f64;
            let window_height = r.draw_size[1] as f64;
            let x_scale = window_width / width;
            let y_scale = window_height / height;
            scale = x_scale.min(y_scale);
            x_trans = (window_width - width * scale) / 2.0;
            y_trans = (window_height - height * scale) / 2.0;
        }
    }
    recorder.stop();
    menu.save_settings();
}
