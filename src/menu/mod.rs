extern crate bincode;

use input::{ControllerState, Input};
use input::Input::*;
use piston_window::*;
use piston_window::Button::*;
use std::fs::File;
use std::path::Path;

const CONTROLS: [(&str, usize); 8] = [
    ("Up", 4),
    ("Down", 5),
    ("Left", 6),
    ("Right", 7),
    ("B", 1),
    ("A", 0),
    ("Select", 2),
    ("Start", 3),
];

pub struct Menu {
    showing: bool,
    buttons: [[Input; 8]; 2],
    current_index: usize,
    awaiting_input: bool,
}

impl Menu {
    pub fn new(default_controls: &[ControllerState; 2]) -> Menu {
        let settings_file = File::open(Path::new("settings.dat"));
        let buttons = match settings_file {
            Ok(file) => match bincode::deserialize_from(file) {
                Ok(buttons) => buttons,
                Err(_) => [default_controls[0].buttons(), default_controls[1].buttons()],
            },
            Err(_) => [default_controls[0].buttons(), default_controls[1].buttons()],
        };
        Menu {
            showing: false,
            buttons,
            current_index: 0,
            awaiting_input: false,
        }
    }

    pub fn update_controls(&self, controls: &mut [ControllerState; 2]) {
        controls[0].set_buttons(&self.buttons[0]);
        controls[1].set_buttons(&self.buttons[1]);
    }

    pub fn event(&mut self, event: &Event) -> bool {
        if self.awaiting_input {
            if let Some(button) = event.release_args() {
                self.buttons[self.current_index / 8][CONTROLS[self.current_index % 8].1] = Button(button);
                self.awaiting_input = false;
            }
        } else {
            match event.release_args() {
                Some(Keyboard(Key::Escape)) => self.showing = !self.showing,
                Some(Keyboard(Key::Up)) => if self.current_index > 0 { self.current_index -= 1 },
                Some(Keyboard(Key::Down)) => if self.current_index < 15 { self.current_index += 1 },
                Some(Keyboard(Key::Return)) => self.awaiting_input = true,
                _ => (),
            }
        }
        self.showing
    }

    pub fn render(&self, c: Context, gl: &mut G2d, glyphs: &mut Glyphs) {
        if self.showing {
            rectangle([0.0, 0.0, 0.0, 0.7], [0.0, 0.0, 293.0, 240.0], c.transform, gl);
            self.render_controls_menu("Player 1", 0, self.buttons[0], c.trans(10.0, 20.0), gl, glyphs);
            self.render_controls_menu("Player 2", 8, self.buttons[1], c.trans(10.0, 135.0), gl, glyphs);
        }
    }

    fn render_controls_menu(&self, header_text: &str, start_index: usize,
                            buttons: [Input; 8],
                            c: Context, gl: &mut G2d, glyphs: &mut Glyphs) {
        self.render_header(header_text, c, gl, glyphs);
        for (menu_index, &(name, array_index)) in CONTROLS.iter().enumerate() {
            self.render_item(name, &input_to_string(buttons[array_index]),
                             self.current_index == start_index + menu_index,
                             c.trans(0.0, 12.0 * (1.0 + menu_index as f64)),
                             gl, glyphs,
            );
        }
    }

    fn render_header(&self, value: &str, c: Context, gl: &mut G2d, glyphs: &mut Glyphs) {
        text([1.0, 1.0, 1.0, 1.0], 10, value, glyphs, c.transform, gl).unwrap();
    }

    fn render_item(&self, name: &str, value: &str, highlight: bool, c: Context, gl: &mut G2d, glyphs: &mut Glyphs) {
        if highlight {
            rectangle(if self.awaiting_input { [0.7, 0.5, 0.3, 1.0] } else { [0.5, 0.7, 0.3, 1.0] }, [0.0, -8.0, 150.0, 12.0], c.transform, gl);
        }
        text([1.0, 1.0, 1.0, 1.0], 8, name, glyphs, c.transform, gl).unwrap();
        text([1.0, 1.0, 1.0, 1.0], 8, value, glyphs, c.trans(50.0, 0.0).transform, gl).unwrap();
    }

    pub fn save_settings(&self) {
        let settings_file = File::create(Path::new("settings.dat")).unwrap();
        bincode::serialize_into(settings_file, &self.buttons).unwrap();
    }
}

fn input_to_string(input: Input) -> String {
    match input {
        Button(Keyboard(key)) => format!("{:?}", key),
        Button(Mouse(button)) => format!("Mouse {:?}", button),
        Button(Controller(button)) => format!("Joy {} button {}", button.id, button.button),
        Axis(axis_args) => format!("Joy {} axis {} {}", axis_args.id, axis_args.axis, if axis_args.position < 0.0 { "-" } else { "+" }),
    }
}
