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
                self.length_counter_load = Some((value & (!0x7)) >> 3);
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
                self.length_counter_load = Some((value & (!0x7)) >> 3);
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
                self.length_counter_load = Some((value & (!0x7)) >> 3);
            }
            _ => panic!("bad APU channel control write {:04X}", address),
        }
    }
}

pub struct ApuBus {
    pub pulse_1: SquareCtrl,
    pub pulse_2: SquareCtrl,
    pub triangle: TriangleCtrl,
    pub noise: NoiseCtrl,

    pub frame_mode: bool,
    pub irq_inhibit: bool,

    pub irq_interrupt: bool,
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
                length_counter_load: None,
                enabled: false,
            },
            triangle: TriangleCtrl {
                enabled: false,
                control_flag: false,
                reload_value: 0,
                timer: 0,
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
                length_counter_load: None,
            },
            frame_mode: false,
            irq_inhibit: false,
            irq_interrupt: false,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x4000 ... 0x4003 => self.pulse_1.write(address - 0x4000, value),
            0x4004 ... 0x4007 => self.pulse_2.write(address - 0x4004, value),
            0x4008 ... 0x400B => self.triangle.write(address - 0x4008, value),
            0x400C ... 0x400F => self.noise.write(address - 0x400C, value),
            0x4010 ... 0x4013 => (),  // TODO: DMC
            0x4015 => {
                // TODO: DMC control
                self.pulse_1.enabled = value & 1 > 0;
                self.pulse_2.enabled = value & 2 > 0;
                self.triangle.enabled = value & 4 > 0;
                self.noise.enabled = value & 8 > 0;
                // TODO: DMC
            }
            0x4017 => {
                self.frame_mode = value & 0x80 > 0;
                self.irq_inhibit = value & 0x40 > 0;
            }
            _ => panic!("bad APU bus write {:04X}", address),
        }
    }
}