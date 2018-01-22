pub struct SweepCtrl {
    pub enabled: bool,
    pub period: u8,
    pub negate: bool,
    pub shift_count: u8,
}

impl SweepCtrl {
    fn write(&mut self, value: u8) {
        self.enabled = value & 0x80 > 0;
        self.period = (value & 0x70) >> 4;
        self.negate = value & 0x8 > 0;
        self.shift_count = value & 0x7;
    }
}

pub struct ChannelCtrl {
    pub enabled: bool,

    pub duty_cycle: usize,
    pub halt_flag_envelope_loop: bool,
    pub constant_volume: bool,
    pub envelope_param: u8,

    pub sweep: SweepCtrl,

    pub timer: u16,
    pub length_counter_load: Option<u8>,
}

impl ChannelCtrl {
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

pub struct ApuBus {
    pub pulse_1: ChannelCtrl,
    pub pulse_2: ChannelCtrl,

    pub frame_mode: bool,
    pub irq_inhibit: bool,

    pub irq_interrupt: bool,
}

impl ApuBus {
    pub fn new() -> ApuBus {
        ApuBus {
            pulse_1: ChannelCtrl {
                duty_cycle: 0,
                halt_flag_envelope_loop: false,
                constant_volume: false,
                envelope_param: 0,
                sweep: SweepCtrl {
                    enabled: false,
                    period: 0,
                    negate: false,
                    shift_count: 0,
                },
                timer: 0,
                length_counter_load: None,
                enabled: false,
            },
            pulse_2: ChannelCtrl {
                duty_cycle: 0,
                halt_flag_envelope_loop: false,
                constant_volume: false,
                envelope_param: 0,
                sweep: SweepCtrl {
                    enabled: false,
                    period: 0,
                    negate: false,
                    shift_count: 0,
                },
                timer: 0,
                length_counter_load: None,
                enabled: false,
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
            0x4008 ... 0x400B => (),  // TODO: triangle
            0x400C ... 0x400F => (),  // TODO: noise
            0x4010 ... 0x4013 => (),  // TODO: DMC
            0x4015 => {
                // TODO: DMC control
                self.pulse_1.enabled = value & 1 > 0;
                self.pulse_2.enabled = value & 2 > 0;
                // TODO: triangle
                // TODO: noise
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