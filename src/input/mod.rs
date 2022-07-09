use piston_window::*;
use piston_window::Button::*;

use self::Input::*;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Input {
    Button(::piston_window::Button),
    Axis(ControllerAxisArgs),
}

pub struct ControllerState<const N: usize> {
    inputs: [Input; N],
    state: u8,
}

impl<const N: usize> ControllerState<N> {
    fn new(inputs: [Input; N]) -> ControllerState<N> {
        ControllerState { inputs, state: 0 }
    }

    pub fn event(&mut self, event: &Event) -> bool {
        let prev_state = self.state;
        if let Some(button_pressed) = event.press_args() {
            for (i, input) in self.inputs.iter().enumerate() {
                if *input == Button(button_pressed) {
                    self.state |= 1 << i;
                }
            }
        }

        if let Some(button_released) = event.release_args() {
            for (i, input) in self.inputs.iter().enumerate() {
                if *input == Button(button_released) {
                    self.state &= !(1 << i);
                }
            }
        }

        if let Some(axis_args) = event.controller_axis_args() {
            for (i, input) in self.inputs.iter().enumerate() {
                if let Axis(input_axis_args) = *input {
                    if axis_args.id == input_axis_args.id && axis_args.axis == input_axis_args.axis
                    {
                        if axis_args.position == 0.0
                            || axis_args.position.signum() != input_axis_args.position.signum()
                        {
                            self.state &= !(1 << i);
                        } else {
                            self.state |= 1 << i;
                        }
                    }
                }
            }
        }

        self.state != prev_state
    }

    pub fn to_u8(&self) -> u8 {
        self.state
    }

    pub fn set_from_u8(&mut self, value: u8) {
        self.state = value;
    }

    pub fn buttons(&self) -> [Input; N] {
        self.inputs.clone()
    }

    pub fn set_buttons(&mut self, buttons: &[Input]) {
        for i in 0..8 {
            self.inputs[i] = buttons[i];
        }
    }

    pub fn render_overlay(&self, c: Context, gl: &mut G2d, glyphs: &mut Glyphs) {
        const INPUT_STRS: [&str; 8] = ["A ", "B ", "s ", "S ", "^ ", "v ", "< ", "> "];
        let value = (0..8)
            .map(|i| {
                if self.state & (1 << i) > 0 {
                    INPUT_STRS[i]
                } else {
                    "  "
                }
            })
            .collect::<String>();
        text(
            [0.0, 0.0, 0.0, 1.0],
            8,
            &value,
            glyphs,
            c.trans(1.0, 1.0).transform,
            gl,
        )
            .unwrap();
        text([1.0, 1.0, 0.5, 1.0], 8, &value, glyphs, c.transform, gl).unwrap();
    }
}

pub fn player_1_nes() -> ControllerState<8> {
    ControllerState::new([
        Button(Keyboard(Key::J)),
        Button(Keyboard(Key::H)),
        Button(Keyboard(Key::Backslash)),
        Button(Keyboard(Key::Return)),
        Button(Keyboard(Key::Up)),
        Button(Keyboard(Key::Down)),
        Button(Keyboard(Key::Left)),
        Button(Keyboard(Key::Right)),
    ])
}

pub fn player_2_nes() -> ControllerState<8> {
    ControllerState::new([
        Button(Keyboard(Key::G)),
        Button(Keyboard(Key::F)),
        Button(Keyboard(Key::CapsLock)),
        Button(Keyboard(Key::Tab)),
        Button(Keyboard(Key::W)),
        Button(Keyboard(Key::S)),
        Button(Keyboard(Key::A)),
        Button(Keyboard(Key::D)),
    ])
}
