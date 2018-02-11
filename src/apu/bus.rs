pub struct SweepCtrl {
    pub enabled: bool,
    pub period: u8,
    pub negate: bool,
    pub shift_count: u8,
    pub ones_complement_adj: bool,
    pub reload: bool,
}

impl SweepCtrl {
    fn write(&mut self, value: u8) {
        self.enabled = value & 0x80 > 0;
        self.period = (value & 0x70) >> 4;
        self.negate = value & 0x8 > 0;
        self.shift_count = value & 0x7;
        self.reload = true;
    }
}

pub struct SquareCtrl {
    pub enabled: bool,

    pub duty_cycle: usize,
    pub halt_flag_envelope_loop: bool,
    pub constant_volume: bool,
    pub envelope_param: u8,

    pub sweep: SweepCtrl,

    pub timer: u16,
    pub length_counter: u8,
    pub length_counter_load: Option<u8>,
}

impl SquareCtrl {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0 => {
                self.duty_cycle = (value >> 6) as usize;
                self.halt_flag_envelope_loop = (value >> 5) & 1 > 0;
                self.constant_volume = (value >> 4) & 1 > 0;
                self.envelope_param = value & 0xF;
            }
            1 => {
                self.sweep.write(value);
            }
            2 => {
                let timer = (self.timer & (!0xFF)) + u16::from(value);
                self.timer = timer;
            }
            3 => {
                let timer = (self.timer & 0xFF) + (u16::from(value & 0x7) << 8);
                self.timer = timer;
                if self.enabled {
                    self.length_counter_load = Some((value & (!0x7)) >> 3);
                }
            }
            _ => panic!("bad APU channel control write {:04X}", address),
        }
    }
}

pub struct TriangleCtrl {
    pub enabled: bool,

    pub control_flag: bool,
    pub reload_value: u8,

    pub timer: u16,
    pub length_counter: u8,
    pub length_counter_load: Option<u8>,
    pub linear_counter_reload: bool,
}

impl TriangleCtrl {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0 => {
                self.control_flag = value & 0x80 > 0;
                self.reload_value = value & (!0x80);
            },
            1 => (),
            2 => {
                let timer = (self.timer & (!0xFF)) + u16::from(value);
                self.timer = timer;
            }
            3 => {
                let timer = (self.timer & 0xFF) + (u16::from(value & 0x7) << 8);
                self.timer = timer;
                if self.enabled {
                    self.length_counter_load = Some((value & (!0x7)) >> 3);
                }
                self.linear_counter_reload = true;
            }
            _ => panic!("bad APU channel control write {:04X}", address),
        }
    }
}

pub struct NoiseCtrl {
    pub enabled: bool,

    pub halt_flag_envelope_loop: bool,
    pub constant_volume: bool,
    pub envelope_param: u8,

    pub loop_noise: bool,
    pub timer: u16,

    pub length_counter: u8,
    pub length_counter_load: Option<u8>,
}

impl NoiseCtrl {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0 => {
                self.halt_flag_envelope_loop = (value >> 5) & 1 > 0;
                self.constant_volume = (value >> 4) & 1 > 0;
                self.envelope_param = value & 0xF;
            }
            1 => (),
            2 => {
                self.loop_noise = (value >> 7) & 1 > 0;
                self.timer = super::noise::TIMER_VALUES[(value & 0xF) as usize];
            }
            3 => {
                if self.enabled {
                    self.length_counter_load = Some((value & (!0x7)) >> 3);
                }
            }
            _ => panic!("bad APU channel control write {:04X}", address),
        }
    }
}

pub struct DmcCtrl {
    pub enabled: bool,

    pub irq_enabled: bool,
    pub loop_sample: bool,
    pub rate: u16,

    pub direct_load: Option<u8>,

    pub sample_address: u16,
    pub sample_length: u16,

    pub bytes_remaining: u16,
}

impl DmcCtrl {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0 => {
                self.irq_enabled = (value >> 7) & 1 > 0;
                self.loop_sample = (value >> 6) & 1 > 0;
                self.rate = super::dmc::TIMER_VALUES[(value & 0xF) as usize];
            }
            1 => self.direct_load = Some(value & (!0x80)),
            2 => self.sample_address = 0xC000 + u16::from(value) * 64,
            3 => self.sample_length = u16::from(value) * 16 + 1,
            _ => panic!("bad APU channel control write {:04X}", address),
        }
    }
}

pub struct ApuBus {
    pub pulse_1: SquareCtrl,
    pub pulse_2: SquareCtrl,
    pub triangle: TriangleCtrl,
    pub noise: NoiseCtrl,
    pub dmc: DmcCtrl,

    pub frame_mode: bool,
    pub frame_irq_inhibit: bool,
    pub frame_mode_written: bool,
    pub frame_mode_age: u8,

    pub dmc_delay: bool,
    pub frame_interrupt: bool,
    pub dmc_interrupt: bool,
}

impl ApuBus {
    pub fn new() -> ApuBus {
        ApuBus {
            pulse_1: SquareCtrl {
                duty_cycle: 0,
                halt_flag_envelope_loop: false,
                constant_volume: false,
                envelope_param: 0,
                sweep: SweepCtrl {
                    enabled: false,
                    period: 0,
                    negate: false,
                    shift_count: 0,
                    ones_complement_adj: true,
                    reload: false,
                },
                timer: 0,
                length_counter: 0,
                length_counter_load: None,
                enabled: false,
            },
            pulse_2: SquareCtrl {
                duty_cycle: 0,
                halt_flag_envelope_loop: false,
                constant_volume: false,
                envelope_param: 0,
                sweep: SweepCtrl {
                    enabled: false,
                    period: 0,
                    negate: false,
                    shift_count: 0,
                    ones_complement_adj: false,
                    reload: false,
                },
                timer: 0,
                length_counter: 0,
                length_counter_load: None,
                enabled: false,
            },
            triangle: TriangleCtrl {
                enabled: false,
                control_flag: false,
                reload_value: 0,
                timer: 0,
                length_counter: 0,
                length_counter_load: None,
                linear_counter_reload: false,
            },
            noise: NoiseCtrl {
                enabled: false,
                halt_flag_envelope_loop: false,
                constant_volume: false,
                envelope_param: 0,
                loop_noise: false,
                timer: 0,
                length_counter: 0,
                length_counter_load: None,
            },
            dmc: DmcCtrl {
                enabled: false,
                irq_enabled: false,
                loop_sample: false,
                rate: 0,
                direct_load: None,
                sample_address: 0xC000,
                sample_length: 0,
                bytes_remaining: 0,
            },
            frame_mode: false,
            frame_irq_inhibit: false,
            frame_mode_written: false,
            frame_mode_age: 0,
            dmc_delay: false,
            frame_interrupt: false,
            dmc_interrupt: false,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x4000 ... 0x4003 => self.pulse_1.write(address - 0x4000, value),
            0x4004 ... 0x4007 => self.pulse_2.write(address - 0x4004, value),
            0x4008 ... 0x400B => self.triangle.write(address - 0x4008, value),
            0x400C ... 0x400F => self.noise.write(address - 0x400C, value),
            0x4010 ... 0x4013 => self.dmc.write(address - 0x4010, value),
            0x4015 => {
                self.pulse_1.enabled = value & 1 > 0;
                self.pulse_2.enabled = value & 2 > 0;
                self.triangle.enabled = value & 4 > 0;
                self.noise.enabled = value & 8 > 0;
                self.dmc.enabled = value & 0x10 > 0;
                self.dmc_interrupt = false;
            }
            0x4017 => {
                self.frame_mode = value & 0x80 > 0;
                self.frame_irq_inhibit = value & 0x40 > 0;
                if self.frame_irq_inhibit {
                    self.frame_interrupt = false;
                }
                self.frame_mode_written = true;
                self.frame_mode_age = 0;
            }
            _ => panic!("bad APU bus write {:04X}", address),
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let mut status = 0;
        if self.pulse_1.length_counter > 0 {
            status += 1 << 0;
        }
        if self.pulse_2.length_counter > 0 {
            status += 1 << 1;
        }
        if self.triangle.length_counter > 0 {
            status += 1 << 2;
        }
        if self.noise.length_counter > 0 {
            status += 1 << 3;
        }
        if self.dmc.bytes_remaining > 0 {
            status += 1 << 4;
        }
        if self.frame_interrupt {
            status += 1 << 6;
        }
        if self.dmc_interrupt {
            status += 1 << 7;
        }
        self.frame_interrupt = false;
        status
    }

    pub fn reset(&mut self, retain_mode: bool) {
        self.pulse_1.enabled = false;
        self.pulse_2.enabled = false;
        self.triangle.enabled = false;
        self.noise.enabled = false;
        self.dmc.enabled = false;
        self.dmc_interrupt = false;
        self.frame_interrupt = false;
        if !retain_mode {
            self.frame_mode = false;
        }
        self.frame_irq_inhibit = false;
        self.frame_mode_written = true;
        self.frame_mode_age = 0;
    }

    pub fn irq_interrupt(&self) -> bool {
        self.frame_interrupt || self.dmc_interrupt
    }
}