use gen::z80::opcodes::*;

pub mod opcodes;

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
    interrupt_enabled_tmp: bool,
    interrupt_mode: u8,
    pub reset: bool,
    pub stopped: bool,
    pub bus_req: bool,
    pub has_bus: bool,

    ram: [u8; 0x2000],
    _cartridge: &'a Box<[u8]>,
    test_ram: Option<Box<[u8]>>,

    bank_register: u32,
    bank_register_bit: usize,

    cycles_to_next: u16,
    ticks_to_next: u16,
    cycle_count: u64,
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
            interrupt_enabled: false,
            interrupt_enabled_tmp: false,
            interrupt_mode: 0,
            stopped: false,
            reset: true,
            bus_req: false,
            has_bus: true,
            ram: [0; 0x2000],
            _cartridge: cartridge,
            test_ram: None,
            bank_register: 0,
            bank_register_bit: 15,
            cycles_to_next: 0,
            ticks_to_next: 0,
            cycle_count: 0,
            instrumented,
        }
    }

    pub fn read_addr(&mut self, addr: u16) -> u8 {
        match &self.test_ram {
            Some(ram) => ram[addr as usize],
            None => match addr {
                0x0000..=0x1FFF => self.ram[addr as usize],
                0x2000..=0x3FFF => self.ram[(addr - 0x2000) as usize],
                0x4000..=0x5FFF => 0, // TODO: YM2612
                0x6000..=0x60FF => 0xFF,
                0x6100..=0x7EFF => 0xFF,
                0x7F00..=0x7F1F => 0, // TODO: VDP
                0x7F20..=0x7FFF => 0xFF,
                0x8000..=0xFFFF => self._cartridge[self.bank_register as usize + addr as usize],
            },
        }
    }

    pub fn read_word_addr(&mut self, addr: u16) -> u16 {
        (self.read_addr(addr) as u16) | ((self.read_addr(addr + 1) as u16) << 8)
    }

    fn read_byte(&mut self, mode: AddrMode) -> Option<u8> {
        match mode {
            AddrMode::Immediate => {
                let val = self.read_addr(self.pc);
                self.pc += 1;
                Some(val)
            }
            AddrMode::Extended => {
                let addr = self.read_word_addr(self.pc);
                self.pc += 2;
                Some(self.read_addr(addr))
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
            AddrMode::IndexedDisplacement(register, displacement) => {
                let addr = match register {
                    IndexRegister::IX => self.ix,
                    IndexRegister::IY => self.iy,
                }
                    .wrapping_add_signed(displacement as i16);
                Some(self.read_addr(addr))
            }
            AddrMode::Register(register) => Some(match register {
                Register::A => self.a[self.af_bank],
                Register::B => (self.bc[self.register_bank] >> 8) as u8,
                Register::C => (self.bc[self.register_bank] & 0xFF) as u8,
                Register::D => (self.de[self.register_bank] >> 8) as u8,
                Register::E => (self.de[self.register_bank] & 0xFF) as u8,
                Register::H => (self.hl[self.register_bank] >> 8) as u8,
                Register::L => (self.hl[self.register_bank] & 0xFF) as u8,
                Register::I => self.i,
                Register::R => self.r,
                Register::IXH => (self.ix >> 8) as u8,
                Register::IXL => (self.ix & 0xFF) as u8,
                Register::IYH => (self.iy >> 8) as u8,
                Register::IYL => (self.iy & 0xFF) as u8,
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
        }
    }

    fn read_word(&mut self, mode: AddrMode) -> Option<u16> {
        match mode {
            AddrMode::Immediate => {
                let val = self.read_word_addr(self.pc);
                self.pc += 2;
                Some(val)
            }
            AddrMode::Extended => {
                let addr = self.read_word_addr(self.pc);
                self.pc += 2;
                Some(self.read_word_addr(addr))
            }
            AddrMode::Indexed(register) => {
                let addr = match register {
                    IndexRegister::IX => self.ix,
                    IndexRegister::IY => self.iy,
                }
                    .wrapping_add_signed((self.read_addr(self.pc) as i8) as i16);
                self.pc += 1;
                Some(self.read_word_addr(addr))
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
                Some(self.read_word_addr(addr))
            }
            _ => None,
        }
    }

    pub fn write_addr(&mut self, addr: u16, val: u8) {
        match &mut self.test_ram {
            Some(ram) => ram[addr as usize] = val,
            None => match addr {
                0x0000..=0x1FFF => self.ram[addr as usize] = val,
                0x2000..=0x3FFF => self.ram[(addr - 0x2000) as usize] = val,
                0x4000..=0x5FFF => {} // TODO: YM2612
                0x6000 => {
                    self.bank_register = (self.bank_register & !(1 << self.bank_register_bit))
                        | ((val & 0b1) << self.bank_register_bit) as u32;
                    self.bank_register_bit += 1;
                    if self.bank_register_bit > 23 {
                        self.bank_register_bit = 15;
                    }
                }
                0x6001..=0x60FF => {}
                0x6100..=0x7EFF => {}
                0x7F00..=0x7F1F => {} // TODO: VDP
                0x7F20..=0x7FFF => panic!(),
                0x8000..=0xFFFF => {}
            },
        }
    }

    pub fn write_word(&mut self, addr: u16, val: u16) {
        self.write_addr(addr, (val & 0xFF) as u8);
        self.write_addr(addr + 1, (val >> 8) as u8);
    }

    fn write(
        &mut self,
        mode: AddrMode,
        op: &mut dyn FnMut(&mut Self) -> (Option<u8>, Option<u16>),
    ) -> (Option<u8>, Option<u16>) {
        self.write_with_addr(mode, &mut |cpu, _| op(cpu))
    }

    fn read_write_byte(
        &mut self,
        mode: AddrMode,
        op: &mut dyn FnMut(&mut Self, u8) -> (Option<u8>, Option<u16>),
    ) -> u8 {
        match mode {
            AddrMode::Register(_) => {
                let val = self.read_byte(mode).unwrap();
                self.write_with_addr(mode, &mut |cpu, _| op(cpu, val))
            }
            _ => self.write_with_addr(mode, &mut |cpu, addr| {
                let val = cpu.read_addr(addr);
                op(cpu, val)
            }),
        }
            .0
            .unwrap()
    }

    fn write_with_addr(
        &mut self,
        mode: AddrMode,
        op: &mut dyn FnMut(&mut Self, u16) -> (Option<u8>, Option<u16>),
    ) -> (Option<u8>, Option<u16>) {
        match mode {
            AddrMode::Extended => {
                let addr = self.read_word_addr(self.pc);
                self.pc += 2;
                let (byte, word) = op(self, addr);
                self.write_byte_or_word_addr(byte, word, addr);
                (byte, word)
            }
            AddrMode::Indexed(register) => {
                let addr = match register {
                    IndexRegister::IX => self.ix,
                    IndexRegister::IY => self.iy,
                }
                    .wrapping_add_signed((self.read_addr(self.pc) as i8) as i16);
                self.pc += 1;
                let (byte, word) = op(self, addr);
                self.write_byte_or_word_addr(byte, word, addr);
                (byte, word)
            }
            AddrMode::IndexedDisplacement(register, displacement) => {
                let addr = match register {
                    IndexRegister::IX => self.ix,
                    IndexRegister::IY => self.iy,
                }
                    .wrapping_add_signed(displacement as i16);
                let (byte, word) = op(self, addr);
                self.write_byte_or_word_addr(byte, word, addr);
                (byte, word)
            }
            AddrMode::Register(register) => {
                let (byte, word) = op(self, 0);
                match register {
                    Register::A => {
                        let val = byte.unwrap();
                        self.a[self.af_bank] = val;
                    }
                    Register::B => {
                        self.bc[self.register_bank] =
                            (self.bc[self.register_bank] & 0xFF) | ((byte.unwrap() as u16) << 8)
                    }
                    Register::C => {
                        self.bc[self.register_bank] =
                            (self.bc[self.register_bank] & 0xFF00) | (byte.unwrap() as u16)
                    }
                    Register::D => {
                        self.de[self.register_bank] =
                            (self.de[self.register_bank] & 0xFF) | ((byte.unwrap() as u16) << 8)
                    }
                    Register::E => {
                        self.de[self.register_bank] =
                            (self.de[self.register_bank] & 0xFF00) | (byte.unwrap() as u16)
                    }
                    Register::H => {
                        self.hl[self.register_bank] =
                            (self.hl[self.register_bank] & 0xFF) | ((byte.unwrap() as u16) << 8)
                    }
                    Register::L => {
                        self.hl[self.register_bank] =
                            (self.hl[self.register_bank] & 0xFF00) | (byte.unwrap() as u16)
                    }
                    Register::I => self.i = byte.unwrap(),
                    Register::R => self.r = byte.unwrap(),
                    Register::IXH => self.ix = (self.ix & 0xFF) | ((byte.unwrap() as u16) << 8),
                    Register::IXL => self.ix = (self.ix & 0xFF00) | (byte.unwrap() as u16),
                    Register::IYH => self.iy = (self.iy & 0xFF) | ((byte.unwrap() as u16) << 8),
                    Register::IYL => self.iy = (self.iy & 0xFF00) | (byte.unwrap() as u16),
                }
                (byte, word)
            }
            AddrMode::RegisterPair(register) => {
                let (byte, word) = op(self, 0);
                match register {
                    RegisterPair::AF => {
                        self.a[self.af_bank] = (word.unwrap() >> 8) as u8;
                        self.f[self.af_bank] = (word.unwrap() & 0xFF) as u8;
                    }
                    RegisterPair::BC => self.bc[self.register_bank] = word.unwrap(),
                    RegisterPair::DE => self.de[self.register_bank] = word.unwrap(),
                    RegisterPair::HL => self.hl[self.register_bank] = word.unwrap(),
                    RegisterPair::SP => self.sp = word.unwrap(),
                    RegisterPair::IXP => self.ix = word.unwrap(),
                    RegisterPair::IYP => self.iy = word.unwrap(),
                }
                (byte, word)
            }
            AddrMode::RegisterIndirect(register) => {
                let addr = self.register_addr(register);
                let (byte, word) = op(self, addr);
                self.write_byte_or_word_addr(byte, word, addr);
                (byte, word)
            }
            _ => panic!(),
        }
    }

    fn write_byte_or_word(&mut self, mode: AddrMode, byte: Option<u8>, word: Option<u16>) {
        match mode {
            AddrMode::Extended => {
                let addr = self.read_word_addr(self.pc);
                self.pc += 2;
                self.write_byte_or_word_addr(byte, word, addr);
            }
            AddrMode::Indexed(register) => {
                let addr = match register {
                    IndexRegister::IX => self.ix,
                    IndexRegister::IY => self.iy,
                }
                    .wrapping_add_signed((self.read_addr(self.pc) as i8) as i16);
                self.pc += 1;
                self.write_byte_or_word_addr(byte, word, addr);
            }
            AddrMode::Register(register) => match register {
                Register::A => {
                    let val = byte.unwrap();
                    self.a[self.af_bank] = val;
                }
                Register::B => {
                    self.bc[self.register_bank] =
                        (self.bc[self.register_bank] & 0xFF) | ((byte.unwrap() as u16) << 8)
                }
                Register::C => {
                    self.bc[self.register_bank] =
                        (self.bc[self.register_bank] & 0xFF00) | (byte.unwrap() as u16)
                }
                Register::D => {
                    self.de[self.register_bank] =
                        (self.de[self.register_bank] & 0xFF) | ((byte.unwrap() as u16) << 8)
                }
                Register::E => {
                    self.de[self.register_bank] =
                        (self.de[self.register_bank] & 0xFF00) | (byte.unwrap() as u16)
                }
                Register::H => {
                    self.hl[self.register_bank] =
                        (self.hl[self.register_bank] & 0xFF) | ((byte.unwrap() as u16) << 8)
                }
                Register::L => {
                    self.hl[self.register_bank] =
                        (self.hl[self.register_bank] & 0xFF00) | (byte.unwrap() as u16)
                }
                Register::I => self.i = byte.unwrap(),
                Register::R => self.r = byte.unwrap(),
                Register::IXH => self.ix = (self.ix & 0xFF) | ((byte.unwrap() as u16) << 8),
                Register::IXL => self.ix = (self.ix & 0xFF00) | (byte.unwrap() as u16),
                Register::IYH => self.iy = (self.iy & 0xFF) | ((byte.unwrap() as u16) << 8),
                Register::IYL => self.iy = (self.iy & 0xFF00) | (byte.unwrap() as u16),
            },
            AddrMode::RegisterPair(register) => match register {
                RegisterPair::AF => {
                    self.a[self.af_bank] = (word.unwrap() >> 8) as u8;
                    self.f[self.af_bank] = (word.unwrap() & 0xFF) as u8;
                }
                RegisterPair::BC => self.bc[self.register_bank] = word.unwrap(),
                RegisterPair::DE => self.de[self.register_bank] = word.unwrap(),
                RegisterPair::HL => self.hl[self.register_bank] = word.unwrap(),
                RegisterPair::SP => self.sp = word.unwrap(),
                RegisterPair::IXP => self.ix = word.unwrap(),
                RegisterPair::IYP => self.iy = word.unwrap(),
            },
            AddrMode::RegisterIndirect(register) => {
                let addr = self.register_addr(register);
                self.write_byte_or_word_addr(byte, word, addr);
            }
            _ => panic!(),
        }
    }

    fn write_byte_or_word_addr(&mut self, byte: Option<u8>, word: Option<u16>, addr: u16) {
        if let Some(val) = byte {
            self.write_addr(addr, val);
        } else if let Some(val) = word {
            self.write_word(addr, val);
        } else {
            panic!();
        }
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

    fn condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::True => true,
            Condition::Carry => self.flag(CARRY),
            Condition::ParityOverflow => self.flag(PARITY_OVERFLOW),
            Condition::Sign => self.flag(SIGN),
            Condition::Zero => self.flag(ZERO),
            Condition::NoCarry => !self.flag(CARRY),
            Condition::NoSign => !self.flag(SIGN),
            Condition::NoParityOverflow => !self.flag(PARITY_OVERFLOW),
            Condition::NoZero => !self.flag(ZERO),
        }
    }

    fn push(&mut self, val: u16) {
        self.sp -= 2;
        self.write_word(self.sp, val);
    }

    fn pop(&mut self) -> u16 {
        let val = self.read_word_addr(self.sp);
        self.sp += 2;
        val
    }

    fn reset(&mut self) {
        self.pc = 0;
        self.i = 0;
        self.r = 0;
        self.interrupt_mode = 0;
    }

    pub fn tick(&mut self, interrupt: bool) {
        if self.stopped {
            self.cycles_to_next = 0;
            self.ticks_to_next = 0;
        } else if self.reset {
            self.reset();
        } else if self.has_bus {
            if self.ticks_to_next == 0 {
                if self.cycles_to_next == 0 {
                    if self.bus_req {
                        self.has_bus = false;
                        self.bus_req = false;
                    } else {
                        if interrupt && self.interrupt_enabled {
                            match self.interrupt_mode {
                                0 => {}
                                1 => {
                                    self.push(self.pc);
                                    self.pc = 0x38;
                                    self.cycles_to_next += 11;
                                }
                                2 => {}
                                _ => panic!()
                            }
                        } else {
                            self.execute_opcode();
                            assert_ne!(self.cycles_to_next, 0);
                        }
                    }
                }
                self.cycles_to_next = self.cycles_to_next.saturating_sub(1);
                self.ticks_to_next = 15;
                self.cycle_count = self.cycle_count.wrapping_add(1);
            }
            self.ticks_to_next = self.ticks_to_next.saturating_sub(1);
        }
    }

    fn execute_opcode(&mut self) {
        let opcode_pc = self.pc;
        let (opcode, opcode_reads) = self.get_opcode();
        self.pc += opcode_reads;

        if self.instrumented {
            debug!(target: "z80", "{:04X} {:?} A:{:02X} F:{:08b} BC:{:04X} DE:{:04X} HL:{:04X} IX:{:04X} IY:{:04X} I:{:02X} R:{:02X} SP:{:04X} {}",
                opcode_pc,
                opcode,
                self.a[self.af_bank],
                self.f[self.af_bank],
                self.bc[self.register_bank],
                self.de[self.register_bank],
                self.hl[self.register_bank],
                self.ix,
                self.iy,
                self.i,
                self.r,
                self.sp,
                self.cycle_count,
            );
        }

        let r_high_bit = self.r >> 7;
        self.r += if opcode_reads == 4 { 2 } else { opcode_reads } as u8;
        self.r &= 0b1111111;
        self.r |= r_high_bit << 7;

        match opcode {
            Opcode::ADC(dest, src) => {
                match (dest, src) {
                    (AddrMode::RegisterPair(_), AddrMode::RegisterPair(_)) => {
                        self.write(dest, &mut |cpu| {
                            let val = cpu.read_word(dest).unwrap();
                            let operand = cpu.read_word(src).unwrap();
                            let result = val
                                .wrapping_add(operand)
                                .wrapping_add(if cpu.flag(CARRY) { 1 } else { 0 });
                            cpu.set_flag(CARRY, result < val);
                            cpu.set_flag(ZERO, result == 0);
                            cpu.set_flag(
                                PARITY_OVERFLOW,
                                Self::overflow_16(val, operand, result, false),
                            );
                            cpu.set_flag(SIGN, result & 0x8000 > 1);
                            cpu.set_flag(SUBTRACT, false);
                            cpu.set_flag(HALF_CARRY, (result & 0xF00) < (val & 0xF00));
                            (None, Some(result))
                        });
                        self.cycles_to_next += 15 + 4 * (opcode_reads - 1);
                    }
                    _ => {
                        self.write(dest, &mut |cpu| {
                            let val = cpu.read_byte(dest).unwrap();
                            let operand = cpu.read_byte(src).unwrap();
                            let result = val
                                .wrapping_add(operand)
                                .wrapping_add(if cpu.flag(CARRY) { 1 } else { 0 });
                            cpu.set_flag(CARRY, result < val);
                            cpu.set_flag(ZERO, result == 0);
                            cpu.set_flag(
                                PARITY_OVERFLOW,
                                Self::overflow_8(val, operand, result, false),
                            );
                            cpu.set_flag(SIGN, result & 0x80 > 1);
                            cpu.set_flag(SUBTRACT, false);
                            cpu.set_flag(HALF_CARRY, (result & 0xF) < (val & 0xF));
                            (Some(result), None)
                        });
                        self.cycles_to_next +=
                            Self::arithmetic_cycles(src) + 4 * (opcode_reads - 1);
                    }
                }
            }
            Opcode::ADD(dest, src) => {
                match (dest, src) {
                    (AddrMode::RegisterPair(_), AddrMode::RegisterPair(_)) => {
                        self.write(dest, &mut |cpu| {
                            let val = cpu.read_word(dest).unwrap();
                            let operand = cpu.read_word(src).unwrap();
                            let result = val.wrapping_add(operand);
                            cpu.set_flag(CARRY, result < val);
                            cpu.set_flag(SUBTRACT, false);
                            cpu.set_flag(HALF_CARRY, (result & 0xF00) < (val & 0xF00));
                            (None, Some(result))
                        });
                        self.cycles_to_next += match dest {
                            AddrMode::RegisterPair(RegisterPair::IXP)
                            | AddrMode::RegisterPair(RegisterPair::IYP) => 15,
                            _ => 11,
                        } + 4 * (opcode_reads - 1);
                    }
                    _ => {
                        self.write(dest, &mut |cpu| {
                            let val = cpu.read_byte(dest).unwrap();
                            let operand = cpu.read_byte(src).unwrap();
                            let result = val.wrapping_add(operand);
                            cpu.set_flag(CARRY, result < val);
                            cpu.set_flag(ZERO, result == 0);
                            cpu.set_flag(
                                PARITY_OVERFLOW,
                                Self::overflow_8(val, operand, result, false),
                            );
                            cpu.set_flag(SIGN, result & 0x80 > 1);
                            cpu.set_flag(SUBTRACT, false);
                            cpu.set_flag(HALF_CARRY, (result & 0xF) < (val & 0xF));
                            (Some(result), None)
                        });
                        self.cycles_to_next +=
                            Self::arithmetic_cycles(src) + 4 * (opcode_reads - 1);
                    }
                };
            }
            Opcode::AND(mode) => {
                let result = self.a[self.af_bank] & self.read_byte(mode).unwrap();
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, false);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, Self::parity(result));
                self.set_flag(SIGN, result & 0x80 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, true);
                self.cycles_to_next += Self::arithmetic_cycles(mode) + 4 * (opcode_reads - 1);
            }
            Opcode::CALL(condition) => {
                let addr = self.read_word_addr(self.pc);
                self.pc += 2;
                if self.condition(condition) {
                    self.push(self.pc);
                    self.pc = addr;
                    self.cycles_to_next += 7;
                }
                self.cycles_to_next += 10;
            }
            Opcode::BIT(bit, mode) => {
                let resolved_mode = self.resolve_index_for_bit_op(mode);
                let val = (self.read_byte(resolved_mode).unwrap() >> bit) & 0b1;
                self.set_flag(ZERO, val == 0);
                self.set_flag(PARITY_OVERFLOW, val == 0);
                self.set_flag(SIGN, bit == 7 && val == 1);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, true);
                self.cycles_to_next += Self::bit_test_cycles(mode);
            }
            Opcode::CCF => {
                self.set_flag(CARRY, !self.flag(CARRY));
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, true);
                self.cycles_to_next += 4;
            }
            Opcode::CP(mode) => {
                let val = self.a[self.af_bank];
                let operand = self.read_byte(mode).unwrap();
                let result = val.wrapping_sub(operand);
                let overflow = Self::overflow_8(val, operand, result, true);
                self.set_flag(CARRY, operand > val);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, overflow);
                self.set_flag(SIGN, result & 0x80 > 0);
                self.set_flag(SUBTRACT, true);
                self.set_flag(HALF_CARRY, operand & 0xF > val & 0xF);
                self.cycles_to_next += Self::arithmetic_cycles(mode) + 4 * (opcode_reads - 1);
            }
            Opcode::CPI | Opcode::CPIR | Opcode::CPD | Opcode::CPDR => {
                let val = self.a[self.af_bank];
                let operand = self.read_addr(self.hl[self.register_bank]);
                let result = val.wrapping_sub(operand);
                if let Opcode::CPI | Opcode::CPIR = opcode {
                    self.hl[self.register_bank] = self.hl[self.register_bank].wrapping_add(1);
                } else {
                    self.hl[self.register_bank] = self.hl[self.register_bank].wrapping_sub(1);
                }
                self.bc[self.register_bank] = self.bc[self.register_bank].wrapping_sub(1);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, self.bc[self.register_bank] != 0);
                self.set_flag(SIGN, result & 0x80 > 0);
                self.set_flag(SUBTRACT, true);
                self.set_flag(HALF_CARRY, operand & 0xF > val & 0xF);
                if let Opcode::CPIR | Opcode::CPDR = opcode {
                    if self.bc[self.register_bank] != 0 && result != 0 {
                        self.pc = opcode_pc;
                        self.cycles_to_next += 5;
                    }
                }
                self.cycles_to_next += 16;
            }
            Opcode::CPL => {
                self.a[self.af_bank] = self.a[self.af_bank] ^ 0xFF;
                self.set_flag(SUBTRACT, true);
                self.set_flag(HALF_CARRY, true);
                self.cycles_to_next += 4;
            }
            Opcode::DAA => {
                let mut a = self.a[self.af_bank];
                let adj_lo = self.flag(HALF_CARRY) || a & 0xF > 0x9;
                let adj_hi = self.flag(CARRY) || a > 0x99;
                let half_carry = if self.flag(SUBTRACT) && !self.flag(HALF_CARRY) {
                    false
                } else if self.flag(SUBTRACT) && self.flag(HALF_CARRY) {
                    a & 0xF < 0x6
                } else {
                    a & 0xF > 0xA
                };
                if adj_hi && adj_lo {
                    if self.flag(SUBTRACT) {
                        a = a.wrapping_sub(0x66);
                    } else {
                        a = a.wrapping_add(0x66);
                    }
                } else if adj_hi {
                    if self.flag(SUBTRACT) {
                        a = a.wrapping_sub(0x60);
                    } else {
                        a = a.wrapping_add(0x60);
                    }
                } else if adj_lo {
                    if self.flag(SUBTRACT) {
                        a = a.wrapping_sub(0x6);
                    } else {
                        a = a.wrapping_add(0x6);
                    }
                }
                self.a[self.af_bank] = a;
                self.set_flag(CARRY, adj_hi);
                self.set_flag(ZERO, a == 0);
                self.set_flag(PARITY_OVERFLOW, Self::parity(a));
                self.set_flag(SIGN, a & 0x80 > 0);
                self.set_flag(HALF_CARRY, half_carry);
                self.cycles_to_next += 4;
            }
            Opcode::DEC(mode) => {
                match mode {
                    AddrMode::Indexed(_)
                    | AddrMode::Register(_)
                    | AddrMode::RegisterIndirect(_) => {
                        self.read_write_byte(mode, &mut |cpu, operand| {
                            let val = operand.wrapping_sub(1);
                            cpu.set_flag(ZERO, val == 0);
                            cpu.set_flag(PARITY_OVERFLOW, val == 0x7F);
                            cpu.set_flag(SIGN, val & 0x80 > 0);
                            cpu.set_flag(SUBTRACT, true);
                            cpu.set_flag(HALF_CARRY, operand & 0xF < val & 0xF);
                            (Some(val), None)
                        });
                    }
                    AddrMode::RegisterPair(_) => {
                        self.write(mode, &mut |cpu| {
                            let val = cpu.read_word(mode).unwrap().wrapping_sub(1);
                            (None, Some(val))
                        });
                    }
                    _ => panic!(),
                }
                self.cycles_to_next += Self::inc_dec_cycles(mode) + 4 * (opcode_reads - 1);
            }
            Opcode::DI => {
                self.interrupt_enabled = false;
                self.interrupt_enabled_tmp = false;
                self.cycles_to_next += 4;
            }
            Opcode::DJNZ => {
                let displacement = self.read_addr(self.pc) as i8;
                self.pc += 1;
                self.bc[self.register_bank] =
                    self.bc[self.register_bank].wrapping_add_signed(-0x100);
                let zero = self.bc[self.register_bank] & 0xFF00 == 0;
                if !zero {
                    self.pc = self.pc.wrapping_add_signed(displacement as i16);
                }
                self.cycles_to_next += if zero { 8 } else { 13 };
            }
            Opcode::EX(dest, src) => {
                let dest_val = self.read_word(dest).unwrap();
                let src_val = self.read_word(src).unwrap();
                self.write_byte_or_word(dest, None, Some(src_val));
                self.write_byte_or_word(src, None, Some(dest_val));
                self.cycles_to_next += match (dest, src) {
                    (AddrMode::RegisterPair(_), AddrMode::RegisterPair(_)) => 4,
                    (AddrMode::RegisterIndirect(_), AddrMode::RegisterPair(RegisterPair::IXP))
                    | (AddrMode::RegisterIndirect(_), AddrMode::RegisterPair(RegisterPair::IYP)) => {
                        23
                    }
                    (AddrMode::RegisterIndirect(_), AddrMode::RegisterPair(_)) => 19,
                    _ => panic!(),
                }
            }
            Opcode::EI => {
                self.interrupt_enabled = true;
                self.interrupt_enabled_tmp = true;
                self.cycles_to_next += 4;
            }
            Opcode::EX_AF => {
                self.af_bank = 1 - self.af_bank;
                self.cycles_to_next += 4;
            }
            Opcode::EXX => {
                self.register_bank = 1 - self.register_bank;
                self.cycles_to_next += 4;
            }
            Opcode::HALT => {
                self.stopped = true;
                self.pc = opcode_pc;
            }
            Opcode::IM(mode) => {
                self.interrupt_mode = mode;
                self.cycles_to_next += 8;
            }
            Opcode::IN(dest, src) => {
                self.read_byte(src);
                self.read_byte(dest);
                self.cycles_to_next += match dest {
                    AddrMode::Immediate => 10,
                    AddrMode::Register(_) => 11,
                    _ => panic!(),
                }
            }
            Opcode::INC(mode) => {
                match mode {
                    AddrMode::Indexed(_)
                    | AddrMode::Register(_)
                    | AddrMode::RegisterIndirect(_) => {
                        self.read_write_byte(mode, &mut |cpu, operand| {
                            let val = operand.wrapping_add(1);
                            cpu.set_flag(ZERO, val == 0);
                            cpu.set_flag(PARITY_OVERFLOW, val == 0x80);
                            cpu.set_flag(SIGN, val & 0x80 > 0);
                            cpu.set_flag(SUBTRACT, false);
                            cpu.set_flag(HALF_CARRY, operand & 0xF > val & 0xF);
                            (Some(val), None)
                        });
                    }
                    AddrMode::RegisterPair(_) => {
                        self.write(mode, &mut |cpu| {
                            let val = cpu.read_word(mode).unwrap().wrapping_add(1);
                            (None, Some(val))
                        });
                    }
                    _ => panic!(),
                }
                self.cycles_to_next += Self::inc_dec_cycles(mode) + 4 * (opcode_reads - 1);
            }
            Opcode::JP(condition) => {
                let addr = self.read_word_addr(self.pc);
                self.pc += 2;
                if self.condition(condition) {
                    self.pc = addr;
                }
                self.cycles_to_next += 10;
            }
            Opcode::JP_Register(register) => {
                self.pc = self.read_word(AddrMode::RegisterPair(register)).unwrap();
                self.cycles_to_next += 4 + 4 * (opcode_reads - 1);
            }
            Opcode::JR(condition) => {
                let displacement = self.read_addr(self.pc) as i8;
                self.pc += 1;
                let condition_val = self.condition(condition);
                if condition_val {
                    self.pc = self.pc.wrapping_add_signed(displacement as i16);
                }
                self.cycles_to_next += if condition_val { 12 } else { 7 };
            }
            Opcode::LD(dest, src) => {
                self.write(dest, &mut |cpu| {
                    let (val_8, val_16) = match (dest, src) {
                        (AddrMode::RegisterPair(_), _) | (_, AddrMode::RegisterPair(_)) => {
                            (None, cpu.read_word(src))
                        }
                        _ => (cpu.read_byte(src), None),
                    };
                    match (dest, src) {
                        (AddrMode::Register(Register::A), AddrMode::Register(Register::I))
                        | (AddrMode::Register(Register::A), AddrMode::Register(Register::R)) => {
                            let val = val_8.unwrap();
                            cpu.set_flag(ZERO, val == 0);
                            cpu.set_flag(PARITY_OVERFLOW, cpu.interrupt_enabled_tmp);
                            cpu.set_flag(SIGN, val & 0x80 > 0);
                            cpu.set_flag(SUBTRACT, false);
                            cpu.set_flag(HALF_CARRY, false);
                        }
                        _ => {}
                    }
                    (val_8, val_16)
                });
                self.cycles_to_next += match (dest, src) {
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
                } + 4 * (opcode_reads - 1);
            }
            Opcode::LDD | Opcode::LDDR | Opcode::LDI | Opcode::LDIR => {
                let val = self.read_byte(AddrMode::RegisterIndirect(RegisterPair::HL));
                self.write_byte_or_word(AddrMode::RegisterIndirect(RegisterPair::DE), val, None);
                if let Opcode::LDI | Opcode::LDIR = opcode {
                    self.de[self.register_bank] = self.de[self.register_bank].wrapping_add(1);
                    self.hl[self.register_bank] = self.hl[self.register_bank].wrapping_add(1);
                } else {
                    self.de[self.register_bank] = self.de[self.register_bank].wrapping_sub(1);
                    self.hl[self.register_bank] = self.hl[self.register_bank].wrapping_sub(1);
                }
                self.bc[self.register_bank] = self.bc[self.register_bank].wrapping_sub(1);
                if let Opcode::LDIR | Opcode::LDDR = opcode {
                    if self.bc[self.register_bank] != 0 {
                        self.pc = opcode_pc;
                        self.cycles_to_next += 5;
                    }
                    self.set_flag(PARITY_OVERFLOW, false);
                } else {
                    self.set_flag(PARITY_OVERFLOW, self.bc[self.register_bank] != 0);
                }
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 16;
            }
            Opcode::NEG => {
                let val = 0u8;
                let operand = self.read_byte(AddrMode::Register(Register::A)).unwrap();
                let result = val.wrapping_sub(operand);
                self.set_flag(CARRY, result > val);
                self.set_flag(ZERO, result == 0);
                self.set_flag(
                    PARITY_OVERFLOW,
                    Self::overflow_8(val, operand, result, true),
                );
                self.set_flag(SIGN, result & 0x80 > 1);
                self.set_flag(SUBTRACT, true);
                self.set_flag(HALF_CARRY, (result & 0xF) > (val & 0xF));
                self.write_byte_or_word(AddrMode::Register(Register::A), Some(result), None);
                self.cycles_to_next += 8;
            }
            Opcode::NOP => {
                self.cycles_to_next += 4 * opcode_reads;
            }
            Opcode::OR(mode) => {
                let result = self.a[self.af_bank] | self.read_byte(mode).unwrap();
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, false);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, Self::parity(result));
                self.set_flag(SIGN, result & 0x80 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += Self::arithmetic_cycles(mode) + 4 * (opcode_reads - 1);
            }
            Opcode::OUT(dest, src) => {
                self.read_byte(src);
                self.read_byte(dest);
                self.cycles_to_next += match dest {
                    AddrMode::Immediate => 11,
                    AddrMode::Register(_) => 12,
                    _ => panic!(),
                }
            }
            Opcode::POP(mode) => {
                let val = self.pop();
                self.write_byte_or_word(mode, None, Some(val));
                self.cycles_to_next += match mode {
                    AddrMode::RegisterPair(RegisterPair::IXP)
                    | AddrMode::RegisterPair(RegisterPair::IYP) => 14,
                    _ => 10,
                };
            }
            Opcode::PUSH(mode) => {
                let val = self.read_word(mode).unwrap();
                self.push(val);
                self.cycles_to_next += match mode {
                    AddrMode::RegisterPair(RegisterPair::IXP)
                    | AddrMode::RegisterPair(RegisterPair::IYP) => 15,
                    _ => 11,
                };
            }
            Opcode::RES(bit, src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |_: &mut Cpu, val: u8| {
                    let mask = 1 << bit;
                    let result = val & !mask;
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::RET(condition) => {
                let condition_val = self.condition(condition);
                if condition_val {
                    self.pc = self.pop();
                }
                self.cycles_to_next += if let Condition::True = condition {
                    10
                } else if condition_val {
                    11
                } else {
                    5
                };
            }
            Opcode::RETI => {
                self.pc = self.pop();
                self.cycles_to_next += 14;
            }
            Opcode::RETN => {
                self.pc = self.pop();
                self.interrupt_enabled = self.interrupt_enabled_tmp;
                self.cycles_to_next += 14;
            }
            Opcode::RL(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let carry_bit = val >> 7;
                    let result = val << 1 | if cpu.flag(CARRY) { 1 } else { 0 };
                    cpu.set_flag(CARRY, carry_bit > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::RLA => {
                let carry_bit = self.a[self.af_bank] >> 7;
                let result = self.a[self.af_bank] << 1 | if self.flag(CARRY) { 1 } else { 0 };
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, carry_bit > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 4;
            }
            Opcode::RLC(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let result = val.rotate_left(1);
                    cpu.set_flag(CARRY, result & 0x1 > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::RLCA => {
                let result = self.a[self.af_bank].rotate_left(1);
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, result & 0x1 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 4;
            }
            Opcode::RLD => {
                let left = self.a[self.af_bank];
                let left_lo = left & 0xF;
                let left_hi = left >> 4;
                let right = self.read_addr(self.hl[self.register_bank]);
                let right_lo = right & 0xF;
                let right_hi = right >> 4;
                self.a[self.af_bank] = (left_hi << 4) | right_hi;
                self.write_addr(self.hl[self.register_bank], (right_lo << 4) | left_lo);
                self.set_flag(ZERO, self.a[self.af_bank] == 0);
                self.set_flag(SIGN, self.a[self.af_bank] & 0x80 > 0);
                self.set_flag(PARITY_OVERFLOW, Self::parity(self.a[self.af_bank]));
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 18;
            }
            Opcode::RR(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let carry_bit = val & 0x1;
                    let result = val >> 1 | if cpu.flag(CARRY) { 0x80 } else { 0 };
                    cpu.set_flag(CARRY, carry_bit > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::RRA => {
                let carry_bit = self.a[self.af_bank] & 0x1;
                let result = self.a[self.af_bank] >> 1 | if self.flag(CARRY) { 0x80 } else { 0 };
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, carry_bit > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 4;
            }
            Opcode::RRC(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let result = val.rotate_right(1);
                    cpu.set_flag(CARRY, result & 0x80 > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::RRCA => {
                let result = self.a[self.af_bank].rotate_right(1);
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, result & 0x80 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 4;
            }
            Opcode::RRD => {
                let left = self.a[self.af_bank];
                let left_lo = left & 0xF;
                let left_hi = left >> 4;
                let right = self.read_addr(self.hl[self.register_bank]);
                let right_lo = right & 0xF;
                let right_hi = right >> 4;
                self.a[self.af_bank] = (left_hi << 4) | right_lo;
                self.write_addr(self.hl[self.register_bank], (left_lo << 4) | right_hi);
                self.set_flag(ZERO, self.a[self.af_bank] == 0);
                self.set_flag(SIGN, self.a[self.af_bank] & 0x80 > 0);
                self.set_flag(PARITY_OVERFLOW, Self::parity(self.a[self.af_bank]));
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 18;
            }
            Opcode::RST(pc) => {
                self.push(self.pc);
                self.pc = pc as u16;
                self.cycles_to_next += 11;
            }
            Opcode::SBC(dest, src) => match (dest, src) {
                (AddrMode::RegisterPair(_), AddrMode::RegisterPair(_)) => {
                    let val = self.read_word(dest).unwrap();
                    let operand = self.read_word(src).unwrap();
                    let result = val.wrapping_sub(operand).wrapping_sub(if self.flag(CARRY) {
                        1
                    } else {
                        0
                    });
                    self.set_flag(CARRY, result > val);
                    self.set_flag(ZERO, result == 0);
                    self.set_flag(
                        PARITY_OVERFLOW,
                        Self::overflow_16(val, operand, result, true),
                    );
                    self.set_flag(SIGN, result & 0x8000 > 1);
                    self.set_flag(SUBTRACT, true);
                    self.set_flag(HALF_CARRY, (result & 0xF00) > (val & 0xF00));
                    self.write_byte_or_word(dest, None, Some(result));
                    self.cycles_to_next += 15;
                }
                _ => {
                    self.write(dest, &mut |cpu| {
                        let val = cpu.read_byte(dest).unwrap();
                        let operand = cpu.read_byte(src).unwrap();
                        let result = val.wrapping_sub(operand).wrapping_sub(if cpu.flag(CARRY) {
                            1
                        } else {
                            0
                        });
                        cpu.set_flag(CARRY, result > val);
                        cpu.set_flag(ZERO, result == 0);
                        cpu.set_flag(
                            PARITY_OVERFLOW,
                            Self::overflow_8(val, operand, result, true),
                        );
                        cpu.set_flag(SIGN, result & 0x80 > 1);
                        cpu.set_flag(SUBTRACT, true);
                        cpu.set_flag(HALF_CARRY, (result & 0xF) > (val & 0xF));
                        (Some(result), None)
                    });
                    self.cycles_to_next += Self::arithmetic_cycles(src) + 4 * (opcode_reads - 1);
                }
            },
            Opcode::SCF => {
                self.set_flag(CARRY, true);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 4;
            }
            Opcode::SET(bit, src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |_: &mut Cpu, val: u8| {
                    let mask = 1 << bit;
                    let result = (val & !mask) | mask;
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::SLA(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let carry_bit = val >> 7;
                    let result = val << 1;
                    cpu.set_flag(CARRY, carry_bit > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::SLL(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let carry_bit = val >> 7;
                    let result = val << 1 | 0x1;
                    cpu.set_flag(CARRY, carry_bit > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::SRA(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let carry_bit = val & 0x1;
                    let result = ((val as i8) >> 1) as u8;
                    cpu.set_flag(CARRY, carry_bit > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::SRL(src, dest) => {
                let resolved_src = self.resolve_index_for_bit_op(src);
                let result = self.read_write_byte(resolved_src, &mut |cpu: &mut Cpu, val: u8| {
                    let carry_bit = val & 0x1;
                    let result = val >> 1;
                    cpu.set_flag(CARRY, carry_bit > 0);
                    cpu.set_flag(ZERO, result == 0);
                    cpu.set_flag(PARITY_OVERFLOW, Self::parity(result));
                    cpu.set_flag(SIGN, result & 0x80 > 0);
                    cpu.set_flag(SUBTRACT, false);
                    cpu.set_flag(HALF_CARRY, false);
                    (Some(result), None)
                });
                if src != dest {
                    let resolved_dest = self.resolve_index_for_bit_op(dest);
                    self.write_byte_or_word(resolved_dest, Some(result), None);
                }
                self.cycles_to_next += Self::bit_op_cycles(src);
            }
            Opcode::SUB(src) => {
                let val = self.read_byte(AddrMode::Register(Register::A)).unwrap();
                let operand = self.read_byte(src).unwrap();
                let result = val.wrapping_sub(operand);
                self.set_flag(CARRY, result > val);
                self.set_flag(ZERO, result == 0);
                self.set_flag(
                    PARITY_OVERFLOW,
                    Self::overflow_8(val, operand, result, true),
                );
                self.set_flag(SIGN, result & 0x80 > 1);
                self.set_flag(SUBTRACT, true);
                self.set_flag(HALF_CARRY, (result & 0xF) > (val & 0xF));
                self.write_byte_or_word(AddrMode::Register(Register::A), Some(result), None);
                self.cycles_to_next += Self::arithmetic_cycles(src) + 4 * (opcode_reads - 1);
            }
            Opcode::XOR(mode) => {
                let result = self.a[self.af_bank] ^ self.read_byte(mode).unwrap();
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, false);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, Self::parity(result));
                self.set_flag(SIGN, result & 0x80 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += Self::arithmetic_cycles(mode) + 4 * (opcode_reads - 1);
            }
            _ => panic!("{:?}", opcode),
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

    fn get_opcode(&mut self) -> (Opcode, u16) {
        let mut pc = self.pc;
        let opcode_hex = self.read_addr(pc) as usize;
        let mut opcode = OPCODES[opcode_hex];
        pc += 1;
        match opcode {
            Opcode::Bit => {
                opcode = BIT_INSTRUCTIONS[self.read_addr(pc) as usize];
                pc += 1;
            }
            Opcode::Ix => {
                opcode = IX_INSTRUCTIONS[self.read_addr(pc) as usize];
                pc += 1;
            }
            Opcode::Iy => {
                opcode = IY_INSTRUCTIONS[self.read_addr(pc) as usize];
                pc += 1;
            }
            Opcode::Misc => {
                opcode = MISC_INSTRUCTIONS[self.read_addr(pc) as usize];
                pc += 1;
            }
            _ => {}
        }
        match opcode {
            Opcode::IxBit => {
                opcode = IX_BIT_INSTRUCTIONS[self.read_addr(pc + 1) as usize];
                pc += 2;
            }
            Opcode::IyBit => {
                opcode = IY_BIT_INSTRUCTIONS[self.read_addr(pc + 1) as usize];
                pc += 2;
            }
            _ => {}
        }
        (opcode, pc - self.pc)
    }

    fn af(&mut self) -> u16 {
        ((self.a[self.af_bank] as u16) << 8) | (self.f[self.af_bank] as u16)
    }

    fn resolve_index_for_bit_op(&mut self, mode: AddrMode) -> AddrMode {
        match mode {
            AddrMode::Indexed(register) => {
                AddrMode::IndexedDisplacement(register, self.read_addr(self.pc - 2) as i8)
            }
            _ => mode,
        }
    }

    pub fn next_op(&mut self) -> u8 {
        self.read_addr(self.pc)
    }

    fn arithmetic_cycles(mode: AddrMode) -> u16 {
        match mode {
            AddrMode::Register(_) => 4,
            AddrMode::Immediate | AddrMode::RegisterIndirect(_) => 7,
            AddrMode::Indexed(_) => 19,
            _ => panic!(),
        }
    }

    fn bit_test_cycles(mode: AddrMode) -> u16 {
        match mode {
            AddrMode::Register(_) => 8,
            AddrMode::RegisterIndirect(_) => 12,
            AddrMode::Indexed(_) => 20,
            _ => panic!(),
        }
    }

    fn bit_op_cycles(mode: AddrMode) -> u16 {
        match mode {
            AddrMode::Register(_) => 8,
            AddrMode::RegisterIndirect(_) => 15,
            AddrMode::Indexed(_) => 23,
            _ => panic!(),
        }
    }

    fn inc_dec_cycles(mode: AddrMode) -> u16 {
        match mode {
            AddrMode::Indexed(_) => 19,
            AddrMode::Register(_) => 4,
            AddrMode::RegisterPair(_) => 6,
            AddrMode::RegisterIndirect(_) => 11,
            _ => panic!(),
        }
    }

    fn overflow_8(op1: u8, op2: u8, result: u8, subtract: bool) -> bool {
        let result_sign = result & 0x80 > 0;
        let op1_sign = op1 & 0x80 > 0;
        let op2_sign = op2 & 0x80 > 0;
        ((op1_sign ^ op2_sign) ^ (!subtract)) & (op1_sign ^ result_sign)
    }

    fn overflow_16(op1: u16, op2: u16, result: u16, subtract: bool) -> bool {
        let result_sign = result & 0x8000 > 0;
        let op1_sign = op1 & 0x8000 > 0;
        let op2_sign = op2 & 0x8000 > 0;
        ((op1_sign ^ op2_sign) ^ (!subtract)) & (op1_sign ^ result_sign)
    }

    fn parity(val: u8) -> bool {
        val.count_ones() % 2 == 0
    }
}

#[cfg(feature = "test")]
#[allow(dead_code)]
pub mod testing {
    use std::borrow::Borrow;

    use gen::z80::Cpu;
    use gen::z80::opcodes::Opcode;

    impl Cpu<'_> {
        pub fn get_de(&self) -> u16 {
            self.de[0]
        }

        pub fn get_pc(&self) -> u16 {
            self.pc
        }

        pub fn get_cycle_count(&self) -> u64 {
            self.cycle_count
        }

        pub fn set_pc(&mut self, pc: u16) {
            self.pc = pc;
        }

        pub fn peek_opcode(&mut self) -> Opcode {
            self.get_opcode().0
        }

        pub fn load_ram(&mut self, start: usize, src: &[u8]) {
            let mut ram = vec![0; start + src.len() + 0x100];
            ram[start..start + src.len()].copy_from_slice(src);
            self.sp = (ram.len() - 1) as u16;
            self.test_ram = Some(ram.into_boxed_slice());
        }

        pub fn init_zex_test_vectors(&mut self) {
            self.write_addr(0x5, 0xC9);
            self.write_word(0x6, self.sp);
        }

        pub fn init_state(
            &mut self,
            af: [u16; 2],
            bc: [u16; 2],
            de: [u16; 2],
            hl: [u16; 2],
            ix: u16,
            iy: u16,
            sp: u16,
            pc: u16,
            i: u8,
            r: u8,
            interupt_enabled: bool,
            interupt_enabled_tmp: bool,
        ) {
            self.a[0] = (af[0] >> 8) as u8;
            self.a[1] = (af[1] >> 8) as u8;
            self.f[0] = (af[0] & 0xFF) as u8;
            self.f[1] = (af[1] & 0xFF) as u8;
            self.bc = bc;
            self.de = de;
            self.hl = hl;
            self.ix = ix;
            self.iy = iy;
            self.sp = sp;
            self.pc = pc;
            self.i = i;
            self.r = r;
            self.interrupt_enabled = interupt_enabled;
            self.interrupt_enabled_tmp = interupt_enabled_tmp;
        }

        pub fn verify_state(
            &mut self,
            af: [u16; 2],
            bc: [u16; 2],
            de: [u16; 2],
            hl: [u16; 2],
            ix: u16,
            iy: u16,
            sp: u16,
            pc: u16,
            i: u8,
            r: u8,
            interupt_enabled: bool,
            interupt_enabled_tmp: bool,
            halted: bool,
            test_id: &str,
        ) {
            let flags_mask = 0b11010111;
            assert_eq!(self.a[self.af_bank], (af[0] >> 8) as u8, "{}   A", test_id);
            assert_eq!(
                self.a[1 - self.af_bank],
                (af[1] >> 8) as u8,
                "{}   A'",
                test_id
            );
            assert_eq!(
                self.f[self.af_bank] & flags_mask,
                (af[0] & 0xFF) as u8 & flags_mask,
                "{}   F exp {:08b} act {:08b}  S V X H X P/V N C",
                test_id,
                (af[0] & 0xFF) as u8 & flags_mask,
                self.f[self.af_bank] & flags_mask
            );
            assert_eq!(
                self.f[1 - self.af_bank] & flags_mask,
                (af[1] & 0xFF) as u8 & flags_mask,
                "{}   F' exp {:08b} act {:08b}  S V X H X P/V N C",
                test_id,
                (af[1] & 0xFF) as u8 & flags_mask,
                self.f[1 - self.af_bank] & flags_mask
            );
            assert_eq!(self.bc[self.register_bank], bc[0], "{}   BC", test_id);
            assert_eq!(self.bc[1 - self.register_bank], bc[1], "{}   BC'", test_id);
            assert_eq!(self.de[self.register_bank], de[0], "{}   DE", test_id);
            assert_eq!(self.de[1 - self.register_bank], de[1], "{}   DE'", test_id);
            assert_eq!(self.hl[self.register_bank], hl[0], "{}   HL", test_id);
            assert_eq!(self.hl[1 - self.register_bank], hl[1], "{}   HL'", test_id);
            assert_eq!(self.ix, ix, "{}   IX", test_id);
            assert_eq!(self.iy, iy, "{}   IY", test_id);
            assert_eq!(self.sp, sp, "{}   SP", test_id);
            assert_eq!(self.pc, pc, "{}   PC", test_id);
            assert_eq!(self.i, i, "{}   I", test_id);
            assert_eq!(self.r, r, "{}   R", test_id);
            assert_eq!(
                self.interrupt_enabled, interupt_enabled,
                "{}   IFF1",
                test_id
            );
            assert_eq!(
                self.interrupt_enabled_tmp, interupt_enabled_tmp,
                "{}   IFF2",
                test_id
            );
            assert_eq!(self.stopped, halted, "{}   HALT", test_id);
        }

        pub fn poke_ram(&mut self, addr: usize, data: &[u8]) {
            self.test_ram.as_mut().unwrap()[addr..addr + data.len()].copy_from_slice(data);
        }

        pub fn verify_ram(&mut self, addr: usize, data: &[u8], test_id: &str) {
            for (i, &d) in data.iter().enumerate() {
                assert_eq!(
                    self.test_ram.as_ref().unwrap()[addr + i],
                    d,
                    "{}   {:04X}",
                    test_id,
                    addr + i
                );
            }
        }

        pub fn step(&mut self) {
            if self.stopped {
                self.cycles_to_next = 0;
            } else {
                self.execute_opcode();
            }
            self.cycle_count += self.cycles_to_next as u64;
            self.cycles_to_next = 0;
        }

        pub fn output_test_string(&self) {
            if self.bc[self.register_bank] & 0xFF == 2 {
                print!("{}", (self.de[self.register_bank] & 0xFF) as u8 as char)
            } else {
                let mut n = self.de[self.register_bank] as usize;
                let test_ram: &[u8] = self.test_ram.as_ref().unwrap().borrow();
                while test_ram[n] != '$' as u8 {
                    print!("{}", test_ram[n] as char);
                    n += 1;
                }
            }
        }
    }
}
