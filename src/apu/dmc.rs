use apu::bus::*;
use cartridge::CartridgeBus;

pub const TIMER_VALUES: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54];

pub struct Dmc {
    curr_timer: u16,
    address: u16,
    sample_buffer: Option<u8>,
    shift_register: u8,
    bits_remaining: u8,
    output_level: u8,
    silence: bool,
}

impl Dmc {
    pub fn new() -> Dmc {
        Dmc {
            curr_timer: 0,
            address: 0,
            sample_buffer: None,
            shift_register: 0,
            bits_remaining: 8,
            output_level: 0,
            silence: true,
        }
    }

    pub fn tick(&mut self, cpu_bus: &mut ApuBus, cartridge: &Box<CartridgeBus>) -> f32 {
        let ctrl_bus = &mut cpu_bus.dmc;
        if ctrl_bus.enabled_set {
            ctrl_bus.enabled_set = false;
            if ctrl_bus.enabled && ctrl_bus.bytes_remaining == 0 {
                self.address = ctrl_bus.sample_address;
                ctrl_bus.bytes_remaining = ctrl_bus.sample_length;
            }
            if !ctrl_bus.enabled {
                self.silence = true;
                self.sample_buffer = None;
                ctrl_bus.bytes_remaining = 0;
                self.output_level = 0
            }
        }
        if ctrl_bus.enabled {
            if self.sample_buffer.is_none() && ctrl_bus.bytes_remaining > 0 {
                cpu_bus.dmc_delay = true;
                self.sample_buffer = Some(cartridge.read_memory(self.address, 0));
                if self.address == 0xFFFF {
                    self.address = 0x8000;
                } else {
                    self.address += 1;
                }
                ctrl_bus.bytes_remaining -= 1;
                if ctrl_bus.bytes_remaining == 0 {
                    if ctrl_bus.loop_sample {
                        self.address = ctrl_bus.sample_address;
                        ctrl_bus.bytes_remaining = ctrl_bus.sample_length;
                    } else {
                        if ctrl_bus.irq_enabled {
                            cpu_bus.dmc_interrupt = true;
                        }
                        ctrl_bus.enabled = false;
                    }
                }
            }
        }

        if ctrl_bus.enabled || self.sample_buffer.is_some() {
            if self.curr_timer == 0 {
                if self.silence {
                    self.output_level = 0;
                } else {
                    if self.shift_register & 1 > 0 {
                        if self.output_level <= 125 {
                            self.output_level += 2;
                        }
                    } else {
                        if self.output_level >= 2 {
                            self.output_level -= 2;
                        }
                    }
                }
                self.shift_register >>= 1;
                self.bits_remaining -= 1;
                if self.bits_remaining == 0 {
                    self.bits_remaining = 8;
                    match self.sample_buffer.take() {
                        Some(value) => {
                            self.silence = false;
                            self.shift_register = value;
                        }
                        None => {
                            self.silence = true;
                        }
                    }
                }
                self.curr_timer = ctrl_bus.rate;
            }
            self.curr_timer -= 2;
        } else {
            self.silence = true;
            self.sample_buffer = None;
            ctrl_bus.bytes_remaining = 0;
            self.output_level = 0
        }

        if let Some(value) = ctrl_bus.direct_load.take() {
            self.output_level = value;
        }
        f32::from(self.output_level)
    }
}