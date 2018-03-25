extern crate byteorder;

use std::sync::mpsc::Sender;
use std::sync::mpsc;
use std::thread;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::VecDeque;
use piston_window::*;
use self::byteorder::{ByteOrder, BigEndian};
use super::input::ControllerState;

pub struct Recorder {
    start_frame: u32,
    sender: Option<Sender<u64>>,
    join_handle: Option<thread::JoinHandle<()>>,
    record_path: PathBuf,
    recording: bool,
    playback: Option<Playback>,
}

impl Recorder {
    pub fn new(path: &Path) -> Recorder {
        let (sender, receiver) = mpsc::channel();
        let path = PathBuf::from(path);
        let record_path = path.clone();
        let join_handle = thread::spawn(move || {
            let mut file: Option<File> = None;
            let mut buf = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
            loop {
                match receiver.recv() {
                    Ok(input) => {
                        BigEndian::write_u64(&mut buf, input);
                        file.get_or_insert_with(|| { File::create(&path).unwrap() }).write_all(&buf).unwrap();
                    }
                    Err(_) => break,
                };
            }
        });
        Recorder {
            start_frame: 0,
            sender: Some(sender),
            join_handle: Some(join_handle),
            record_path,
            recording: false,
            playback: None,
        }
    }

    pub fn toggle(&mut self, frame: u32) {
        if self.recording {
            self.recording = false;
        } else {
            self.recording = true;
            self.start_frame = frame;
        }
    }

    pub fn input_changed(&mut self, inputs: &[ControllerState; 2], frame_count: u32) {
        if self.recording {
            if let Some(ref sender) = self.sender {
                sender.send((((frame_count - self.start_frame) as u64) << 32)
                    | ((inputs[0].to_u8() as u64) << 8)
                    | (inputs[1].to_u8() as u64)).unwrap();
            }
        }
    }

    pub fn stop(&mut self) {
        drop(self.sender.take().unwrap());
        self.join_handle.take().unwrap().join().unwrap();
    }

    pub fn toggle_playback(&mut self, frame: u32) {
        if self.playback.is_none() {
            let mut src = File::open(&self.record_path).unwrap();
            self.playback = Some(Playback::new(&mut src, frame));
        } else {
            self.playback = None;
        }
    }

    pub fn set_frame_inputs(&mut self, inputs: &mut [ControllerState; 2], frame: u32) {
        let mut done = false;
        if let Some(ref mut playback) = self.playback {
            done = playback.set_frame_inputs(inputs, frame);
        }
        if done {
            self.playback = None;
        }
    }

    pub fn render_overlay(&self, c: Context, gl: &mut G2d) {
        if self.recording {
            ellipse([1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 10.0, 10.0], c.transform, gl);
        }
        if self.playback.is_some() {
            polygon([0.0, 1.0, 0.0, 1.0], &[[0.0, 0.0], [10.0, 5.0], [0.0, 10.0]], c.transform, gl);
        }
    }
}

struct Playback {
    start_frame: u32,
    next_frame: u32,
    input_data: VecDeque<u8>,
}

impl Playback {
    fn new(src: &mut Read, start_frame: u32) -> Playback {
        let mut input_vec = Vec::new();
        src.read_to_end(&mut input_vec).unwrap();
        let mut input_data = VecDeque::from(input_vec);
        let mut next_frame = 0u32;
        for _ in 0..4 {
            next_frame <<= 8;
            next_frame += input_data.pop_front().unwrap() as u32;
        }
        next_frame += start_frame;
        input_data.pop_front();
        input_data.pop_front();
        Playback {
            start_frame,
            next_frame,
            input_data,
        }
    }

    pub fn set_frame_inputs(&mut self, inputs: &mut [ControllerState; 2], frame: u32) -> bool {
        if frame == self.next_frame {
            inputs[0].set_from_u8(self.input_data.pop_front().unwrap());
            inputs[1].set_from_u8(self.input_data.pop_front().unwrap());
            if !self.input_data.is_empty() {
                let mut next_frame = 0u32;
                for _ in 0..4 {
                    next_frame <<= 8;
                    next_frame += self.input_data.pop_front().unwrap() as u32;
                }
                self.input_data.pop_front();
                self.input_data.pop_front();
                self.next_frame = self.start_frame + next_frame;
                false
            } else {
                self.next_frame = 0;
                true
            }
        } else {
            false
        }
    }
}