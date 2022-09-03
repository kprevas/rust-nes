use gen::z80::opcodes::{
    AddrMode, BIT_INSTRUCTIONS, IndexRegister, IX_BIT_INSTRUCTIONS, IX_INSTRUCTIONS, IY_BIT_INSTRUCTIONS, IY_INSTRUCTIONS,
    MISC_INSTRUCTIONS, Opcode, OPCODES, Register, RegisterPair,
};

mod opcodes;

const CARRY: u8 = 0b1;
const SUBTRACT: u8 = 0b10;
const PARITY_OVERFLOW: u8 = 0b100;
const HALF_CARRY: u8 = 0b10000;
const ZERO: u8 = 0b1000000;
const SIGN: u8 = 0b10000000;

pub struct Cpu<'a> {
    pc: u16,
    sp: u16,
    ix: u16,
    iy: u16,
    i: u8,
    r: u8,
    a: [u8; 2],
    f: [u8; 2],
    bc: [u16; 2],
    de: [u16; 2],
    hl: [u16; 2],

    register_bank: usize,
    af_bank: usize,

    interrupt_enabled: bool,
    interrupt_mode: u8,
    pub stopped: bool,

    ram: [u8; 0x2000],
    cartridge: &'a Box<[u8]>,

    ticks: u16,
    pub instrumented: bool,
}

impl Cpu<'_> {
    pub fn new(cartridge: &Box<[u8]>, instrumented: bool) -> Cpu {
        Cpu {
            pc: 0,
            sp: 0,
            ix: 0,
            iy: 0,
            i: 0,
            r: 0,
            a: [0, 0],
            f: [0, 0],
            bc: [0, 0],
            de: [0, 0],
            hl: [0, 0],
            register_bank: 0,
            af_bank: 0,
            interrupt_mode: 0,
            stopped: false,
            ram: [0; 0x2000],
            cartridge,
            ticks: 0,
            instrumented,
        }
    }

    fn read_addr(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize],
            0x2000..=0x3FFF => self.ram[(addr - 0x2000) as usize],
            0x4000..=0x5FFF => 0, // TODO: YM2612
            0x6000..=0x60FF => 0xFF,
            0x6100..=0x7EFF => 0xFF,
            0x7F00..=0x7F1F => 0, // TODO: VDP
            0x7F20..=0x7FFF => 0xFF,
            0x8000..=0xFFFF => 0, // TODO: M68k
            _ => panic!(),
        }
    }

    fn read_word(&mut self, addr: u16) -> u16 {
        (self.read_addr(addr) as u16) | ((self.read_addr(addr + 1) as u16) << 8)
    }

    fn write_addr(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize] = val,
            0x2000..=0x3FFF => self.ram[(addr - 0x2000) as usize] = val,
            0x4000..=0x5FFF => {} // TODO: YM2612
            0x6000..=0x60FF => {} // TODO: bank addr register
            0x6100..=0x7EFF => {}
            0x7F00..=0x7F1F => {} // TODO: VDP
            0x7F20..=0x7FFF => panic!(),
            0x8000..=0xFFFF => {}
            _ => panic!(),
        }
    }

    fn write_word(&mut self, addr: u16, val: u16) {
        self.write_addr(addr, (val & 0xFF) as u8);
        self.write_addr(addr + 1, (val >> 8) as u8);
    }

    fn set_flag(&mut self, flag: u8, set: bool) {
        if set {
            self.f[self.af_bank] = self.f[self.af_bank] | flag;
        } else {
            self.f[self.af_bank] = self.f[self.af_bank] & !flag;
        }
    }

    fn flag(&self, flag: u8) -> bool {
        self.f[self.af_bank] & flag > 0
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.i = 0;
        self.r = 0;
        self.interrupt_mode = 0;
    }

    pub fn tick(&mut self) {
        if self.ticks == 0 {
            self.next_operation();
        }
        self.ticks = self.ticks.saturating_sub(1);
    }

    fn next_operation(&mut self) {
        if self.stopped {
            self.ticks = 0;
        } else {
            self.execute_opcode();
            self.r += 1;
            self.r &= 0b1111111;
        }
    }

    fn execute_opcode(&mut self) {
        let opcode_pc = self.read_addr(self.pc) as usize;
        let mut opcode = OPCODES[opcode_pc];
        self.pc += 1;
        match opcode {
            Opcode::Bit => {
                opcode = BIT_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::Ix => {
                opcode = IX_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::Iy => {
                opcode = IY_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::Misc => {
                opcode = MISC_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            _ => {}
        }
        match opcode {
            Opcode::IxBit => {
                opcode = IX_BIT_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::IyBit => {
                opcode = IY_BIT_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            _ => {}
        }

        if self.instrumented {
            debug!(target: "z80", "{:04X} {:02X} {:02X} {:04X} {:04X} {:04X} {:02X} {:02X}",
            self.pc,
            self.a[self.af_bank],
            self.f[self.af_bank],
            self.bc[self.register_bank],
            self.de[self.register_bank],
            self.hl[self.register_bank],
            self.i,
            self.r);
        }

        match opcode {
            Opcode::HALT => {
                self.stopped = true;
            }
            Opcode::LD(dest, src) => {
                let val_8 = match src {
                    AddrMode::Immediate => {
                        if let AddrMode::RegisterPair(_) = dest {
                            None
                        } else {
                            let val = self.read_addr(self.pc);
                            self.pc += 1;
                            Some(val)
                        }
                    }
                    AddrMode::Extended => {
                        if let AddrMode::RegisterPair(_) = dest {
                            None
                        } else {
                            let addr = self.read_word(self.pc);
                            self.pc += 2;
                            Some(self.read_addr(addr))
                        }
                    }
                    AddrMode::Indexed(register) => {
                        let addr = match register {
                            IndexRegister::IX => self.ix,
                            IndexRegister::IY => self.iy,
                        }
                            .wrapping_add_signed((self.read_addr(self.pc) as i8) as i16);
                        self.pc += 1;
                        Some(self.read_addr(addr))
                    }
                    AddrMode::Register(register) => Some(match register {
                        Register::A => self.a[self.af_bank],
                        Register::B => (self.bc[self.register_bank] >> 8) as u8,
                        Register::C => (self.bc[self.register_bank] | 0xFF) as u8,
                        Register::D => (self.de[self.register_bank] >> 8) as u8,
                        Register::E => (self.de[self.register_bank] | 0xFF) as u8,
                        Register::H => (self.hl[self.register_bank] >> 8) as u8,
                        Register::L => (self.hl[self.register_bank] | 0xFF) as u8,
                        Register::I => self.i,
                        Register::R => self.r,
                        Register::IXH => (self.ix >> 8) as u8,
                        Register::IXL => (self.ix | 0xFF) as u8,
                        Register::IYH => (self.iy >> 8) as u8,
                        Register::IYL => (self.iy | 0xFF) as u8,
                    }),
                    AddrMode::RegisterIndirect(register) => Some(self.read_addr(match register {
                        RegisterPair::AF => {
                            ((self.a[self.af_bank] as u16) << 8) | (self.f[self.af_bank] as u16)
                        }
                        RegisterPair::BC => self.bc[self.register_bank],
                        RegisterPair::DE => self.de[self.register_bank],
                        RegisterPair::HL => self.hl[self.register_bank],
                        RegisterPair::SP => self.sp,
                        RegisterPair::IXP => self.ix,
                        RegisterPair::IYP => self.iy,
                    })),
                    _ => None,
                };
                let val_16 = match src {
                    AddrMode::Immediate => {
                        if let AddrMode::RegisterPair(_) = dest {
                            let val = self.read_word(self.pc);
                            self.pc += 2;
                            Some(val)
                        } else {
                            None
                        }
                    }
                    AddrMode::Extended => {
                        if let AddrMode::RegisterPair(_) = dest {
                            let addr = self.read_word(self.pc);
                            self.pc += 2;
                            Some(self.read_word(addr))
                        } else {
                            None
                        }
                    }
                    AddrMode::Indexed(register) => {
                        let addr = match register {
                            IndexRegister::IX => self.ix,
                            IndexRegister::IY => self.iy,
                        }
                            .wrapping_add_signed((self.read_addr(self.pc) as i8) as i16);
                        self.pc += 1;
                        Some(self.read_word(addr))
                    }
                    AddrMode::RegisterPair(register) => Some(match register {
                        RegisterPair::AF => self.af(),
                        RegisterPair::BC => self.bc[self.register_bank],
                        RegisterPair::DE => self.de[self.register_bank],
                        RegisterPair::HL => self.hl[self.register_bank],
                        RegisterPair::IXP => self.ix,
                        RegisterPair::IYP => self.iy,
                        RegisterPair::SP => self.sp,
                    }),
                    AddrMode::RegisterIndirect(register) => {
                        let addr = self.register_addr(register);
                        Some(self.read_word(addr))
                    }
                    _ => None,
                };
                match dest {
                    AddrMode::Extended => {
                        let addr = self.read_word(self.pc);
                        self.pc += 2;
                        self.write_byte_or_word(val_8, val_16, addr);
                    }
                    AddrMode::Indexed(register) => {
                        let addr = match register {
                            IndexRegister::IX => self.ix,
                            IndexRegister::IY => self.iy,
                        }
                            .wrapping_add_signed((self.read_addr(self.pc) as i8) as i16);
                        self.pc += 1;
                        self.write_byte_or_word(val_8, val_16, addr);
                    }
                    AddrMode::Register(register) => match register {
                        Register::A => {
                            let val = val_8.unwrap();
                            self.a[self.af_bank] = val;
                            match src {
                                AddrMode::Register(Register::I) | AddrMode::Register(Register::R) => {
                                    self.set_flag(ZERO, val == 0);
                                    self.set_flag(PARITY_OVERFLOW, self.interrupt_enabled);
                                    self.set_flag(SIGN, val & 0x80 > 0);
                                }
                                _ => {}
                            }
                        },
                        Register::B => {
                            self.bc[self.register_bank] = (self.bc[self.register_bank] & 0xFF)
                                | ((val_8.unwrap() as u16) << 8)
                        }
                        Register::C => {
                            self.bc[self.register_bank] =
                                (self.bc[self.register_bank] & 0xFF00) | (val_8.unwrap() as u16)
                        }
                        Register::D => {
                            self.de[self.register_bank] = (self.de[self.register_bank] & 0xFF)
                                | ((val_8.unwrap() as u16) << 8)
                        }
                        Register::E => {
                            self.de[self.register_bank] =
                                (self.de[self.register_bank] & 0xFF00) | (val_8.unwrap() as u16)
                        }
                        Register::H => {
                            self.hl[self.register_bank] = (self.hl[self.register_bank] & 0xFF)
                                | ((val_8.unwrap() as u16) << 8)
                        }
                        Register::L => {
                            self.hl[self.register_bank] =
                                (self.hl[self.register_bank] & 0xFF00) | (val_8.unwrap() as u16)
                        }
                        Register::I => self.i = val_8.unwrap(),
                        Register::R => self.r = val_8.unwrap(),
                        Register::IXH => {
                            self.ix = (self.ix & 0xFF) | ((val_8.unwrap() as u16) << 8)
                        }
                        Register::IXL => self.ix = (self.ix & 0xFF00) | (val_8.unwrap() as u16),
                        Register::IYH => {
                            self.iy = (self.iy & 0xFF) | ((val_8.unwrap() as u16) << 8)
                        }
                        Register::IYL => self.iy = (self.iy & 0xFF00) | (val_8.unwrap() as u16),
                    },
                    AddrMode::RegisterPair(register) => match register {
                        RegisterPair::AF => {
                            self.a[self.af_bank] = (val_16.unwrap() >> 8) as u8;
                            self.f[self.af_bank] = (val_16.unwrap() & 0xFF) as u8;
                        }
                        RegisterPair::BC => self.bc[self.register_bank] = val_16.unwrap(),
                        RegisterPair::DE => self.de[self.register_bank] = val_16.unwrap(),
                        RegisterPair::HL => self.hl[self.register_bank] = val_16.unwrap(),
                        RegisterPair::SP => self.sp = val_16.unwrap(),
                        RegisterPair::IXP => self.ix = val_16.unwrap(),
                        RegisterPair::IYP => self.iy = val_16.unwrap(),
                    },
                    AddrMode::RegisterIndirect(register) => {
                        let addr = self.register_addr(register);
                        self.write_byte_or_word(val_8, val_16, addr);
                    }
                    _ => panic!(),
                }
                self.ticks += match (dest, src) {
                    (AddrMode::Register(_), AddrMode::Register(Register::I))
                    | (AddrMode::Register(_), AddrMode::Register(Register::R))
                    | (AddrMode::Register(Register::I), AddrMode::Register(_))
                    | (AddrMode::Register(Register::R), AddrMode::Register(_)) => 9,
                    (
                        AddrMode::RegisterPair(RegisterPair::SP),
                        AddrMode::RegisterPair(RegisterPair::HL),
                    ) => 6,
                    (AddrMode::Register(_), AddrMode::Register(_)) => 4,
                    (AddrMode::Register(_), AddrMode::Immediate)
                    | (AddrMode::Register(_), AddrMode::RegisterIndirect(_))
                    | (AddrMode::RegisterIndirect(_), AddrMode::Register(_)) => 7,
                    (AddrMode::Indexed(_), _) | (_, AddrMode::Indexed(_)) => 19,
                    (AddrMode::RegisterPair(RegisterPair::IXP), AddrMode::Immediate)
                    | (AddrMode::RegisterPair(RegisterPair::IYP), AddrMode::Immediate) => 14,
                    (AddrMode::RegisterIndirect(_), AddrMode::Immediate)
                    | (AddrMode::RegisterPair(_), AddrMode::Immediate)
                    | (AddrMode::RegisterPair(_), AddrMode::RegisterPair(_)) => 10,
                    (AddrMode::Register(_), AddrMode::Extended)
                    | (AddrMode::Extended, AddrMode::Register(_)) => 13,
                    (AddrMode::RegisterPair(RegisterPair::HL), AddrMode::Extended)
                    | (AddrMode::Extended, AddrMode::RegisterPair(RegisterPair::HL)) => 16,
                    (AddrMode::RegisterPair(_), AddrMode::Extended)
                    | (AddrMode::Extended, AddrMode::RegisterPair(_)) => 20,
                    _ => panic!("{:?}", opcode),
                } * 15;
            }
            Opcode::NOP => {
                self.ticks += 4 * 15;
            }
            _ => {}
        }
    }

    fn write_byte_or_word(&mut self, byte: Option<u8>, word: Option<u16>, addr: u16) {
        if let Some(val) = byte {
            self.write_addr(addr, val);
        } else if let Some(val) = word {
            self.write_word(addr, val);
        } else {
            panic!();
        }
    }

    fn register_addr(&mut self, register: RegisterPair) -> u16 {
        match register {
            RegisterPair::AF => self.af(),
            RegisterPair::BC => self.bc[self.register_bank],
            RegisterPair::DE => self.de[self.register_bank],
            RegisterPair::HL => self.hl[self.register_bank],
            RegisterPair::SP => self.sp,
            RegisterPair::IXP => self.ix,
            RegisterPair::IYP => self.iy,
        }
    }

    fn af(&mut self) -> u16 {
        ((self.a[self.af_bank] as u16) << 8) | (self.f[self.af_bank] as u16)
    }
}

#[cfg(feature = "test")]
#[allow(dead_code)]
pub mod testing {
    use gen::z80::Cpu;

    impl Cpu<'_> {
        pub fn get_de(&mut self) -> u16 {
            self.de[0]
        }

        pub fn get_pc(&mut self) -> u16 {
            self.pc
        }

        pub fn set_pc(&mut self, pc: u16) {
            self.pc = pc;
        }

        pub fn load_ram(&mut self, start: usize, ram: &[u8]) {
            self.ram[start..start + ram.len()].copy_from_slice(ram);
        }
    }
}
