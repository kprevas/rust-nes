use gen::z80::opcodes::*;

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
    _cartridge: &'a Box<[u8]>,
    test_ram: Option<Box<[u8]>>,

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
            interrupt_mode: 0,
            stopped: false,
            ram: [0; 0x2000],
            _cartridge: cartridge,
            test_ram: None,
            cycles_to_next: 0,
            ticks_to_next: 0,
            cycle_count: 0,
            instrumented,
        }
    }

    fn read_addr(&mut self, addr: u16) -> u8 {
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
                0x8000..=0xFFFF => 0, // TODO: M68k
            },
        }
    }

    fn read_word_addr(&mut self, addr: u16) -> u16 {
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

    fn write_addr(&mut self, addr: u16, val: u8) {
        match &mut self.test_ram {
            Some(ram) => ram[addr as usize] = val,
            None => match addr {
                0x0000..=0x1FFF => self.ram[addr as usize] = val,
                0x2000..=0x3FFF => self.ram[(addr - 0x2000) as usize] = val,
                0x4000..=0x5FFF => {} // TODO: YM2612
                0x6000..=0x60FF => {} // TODO: bank addr register
                0x6100..=0x7EFF => {}
                0x7F00..=0x7F1F => {} // TODO: VDP
                0x7F20..=0x7FFF => panic!(),
                0x8000..=0xFFFF => {}
            },
        }
    }

    fn write_word(&mut self, addr: u16, val: u16) {
        self.write_addr(addr, (val & 0xFF) as u8);
        self.write_addr(addr + 1, (val >> 8) as u8);
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

    pub fn reset(&mut self) {
        self.pc = 0;
        self.i = 0;
        self.r = 0;
        self.interrupt_mode = 0;
    }

    pub fn tick(&mut self) {
        if self.ticks_to_next == 0 {
            if self.cycles_to_next == 0 {
                self.next_operation();
            }
            assert_ne!(self.cycles_to_next, 0);
            self.cycles_to_next = self.cycles_to_next.saturating_sub(1);
            self.ticks_to_next = 15;
            self.cycle_count = self.cycle_count.wrapping_add(1);
        }
        self.ticks_to_next = self.ticks_to_next.saturating_sub(1);
    }

    fn next_operation(&mut self) {
        if self.stopped {
            self.cycles_to_next = 0;
        } else {
            self.execute_opcode();
            self.r += 1;
            self.r &= 0b1111111;
        }
    }

    fn execute_opcode(&mut self) {
        let opcode_pc = self.pc;
        let opcode_hex = self.read_addr(self.pc) as usize;
        let mut opcode = OPCODES[opcode_hex];
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

        match opcode {
            Opcode::ADD(dest, src) => {
                match (dest, src) {
                    (AddrMode::RegisterPair(_), AddrMode::RegisterPair(_)) => {
                        let val = self.read_word(dest).unwrap();
                        let operand = self.read_word(src).unwrap();
                        let result = val.wrapping_add(operand);
                        self.set_flag(CARRY, result < val);
                        self.set_flag(SUBTRACT, false);
                        self.write_byte_or_word(dest, None, Some(result));
                        self.cycles_to_next += 11;
                    }
                    _ => {
                        let val = self.read_byte(dest).unwrap();
                        let operand = self.read_byte(src).unwrap();
                        let result = val.wrapping_add(operand);
                        self.set_flag(CARRY, result < val);
                        self.set_flag(ZERO, result == 0);
                        self.set_flag(PARITY_OVERFLOW, (result as i8) < (val as i8));
                        self.set_flag(SIGN, result & 0x80 > 1);
                        self.set_flag(SUBTRACT, false);
                        self.set_flag(HALF_CARRY, (result & 0xF) < (val & 0xF));
                        self.write_byte_or_word(dest, Some(result), None);
                        self.cycles_to_next += Self::arithmetic_cycles(src);
                    }
                };
            }
            Opcode::AND(mode) => {
                let result = self.a[self.af_bank] & self.read_byte(mode).unwrap();
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, false);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, result & 0b1 == 0);
                self.set_flag(SIGN, result & 0x80 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += Self::arithmetic_cycles(mode);
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
            Opcode::CP(mode) => {
                let val = self.a[self.af_bank] as i8;
                let operand = self.read_byte(mode).unwrap() as i8;
                let (carry, result) = match val.checked_sub(operand) {
                    Some(result) => (false, result),
                    None => (true, val.wrapping_sub(operand)),
                };
                let overflow = (operand < 0) == (result < 0) && (operand < 0) != (val < 0);
                self.set_flag(CARRY, carry);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, overflow);
                self.set_flag(SIGN, result < 0);
                self.set_flag(SUBTRACT, true);
                self.set_flag(HALF_CARRY, operand & 0xF > val & 0xF);
                self.cycles_to_next += Self::arithmetic_cycles(mode);
            }
            Opcode::DEC(mode) => {
                match mode {
                    AddrMode::Indexed(_)
                    | AddrMode::Register(_)
                    | AddrMode::RegisterIndirect(_) => {
                        let operand = self.read_byte(mode).unwrap();
                        let val = operand.wrapping_sub(1);
                        self.set_flag(ZERO, val == 0);
                        self.set_flag(PARITY_OVERFLOW, val == 0x0F);
                        self.set_flag(SIGN, val & 0x80 > 0);
                        self.set_flag(SUBTRACT, true);
                        self.set_flag(HALF_CARRY, operand & 0xF < val & 0xF);
                        self.write_byte_or_word(mode, Some(val), None);
                    }
                    AddrMode::RegisterPair(_) => {
                        let val = self.read_word(mode).unwrap().wrapping_sub(1);
                        self.write_byte_or_word(mode, None, Some(val));
                    }
                    _ => panic!(),
                }
                self.cycles_to_next += match mode {
                    AddrMode::Indexed(_) => 23,
                    AddrMode::Register(_) => 4,
                    AddrMode::RegisterPair(RegisterPair::IXP)
                    | AddrMode::RegisterPair(RegisterPair::IYP) => 10,
                    AddrMode::RegisterPair(_) => 6,
                    AddrMode::RegisterIndirect(_) => 11,
                    _ => panic!(),
                };
            }
            Opcode::DI => {
                self.interrupt_enabled = false;
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
            }
            Opcode::INC(mode) => {
                match mode {
                    AddrMode::Indexed(_)
                    | AddrMode::Register(_)
                    | AddrMode::RegisterIndirect(_) => {
                        let operand = self.read_byte(mode).unwrap();
                        let val = operand.wrapping_add(1);
                        self.set_flag(ZERO, val == 0);
                        self.set_flag(PARITY_OVERFLOW, val == 0xF0);
                        self.set_flag(SIGN, val & 0x80 > 0);
                        self.set_flag(SUBTRACT, false);
                        self.set_flag(HALF_CARRY, operand & 0xF > val & 0xF);
                        self.write_byte_or_word(mode, Some(val), None);
                    }
                    AddrMode::RegisterPair(_) => {
                        let val = self.read_word(mode).unwrap().wrapping_add(1);
                        self.write_byte_or_word(mode, None, Some(val));
                    }
                    _ => panic!(),
                }
                self.cycles_to_next += match mode {
                    AddrMode::Indexed(_) => 23,
                    AddrMode::Register(_) => 4,
                    AddrMode::RegisterPair(RegisterPair::IXP)
                    | AddrMode::RegisterPair(RegisterPair::IYP) => 10,
                    AddrMode::RegisterPair(_) => 6,
                    AddrMode::RegisterIndirect(_) => 11,
                    _ => panic!(),
                };
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
                self.cycles_to_next += 4;
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
                let (val_8, val_16) = match (dest, src) {
                    (AddrMode::RegisterPair(_), _) | (_, AddrMode::RegisterPair(_)) => {
                        (None, self.read_word(src))
                    }
                    _ => (self.read_byte(src), None),
                };
                self.write_byte_or_word(dest, val_8, val_16);
                match (dest, src) {
                    (AddrMode::Register(Register::A), AddrMode::Register(Register::I))
                    | (AddrMode::Register(Register::A), AddrMode::Register(Register::R)) => {
                        let val = val_8.unwrap();
                        self.set_flag(ZERO, val == 0);
                        self.set_flag(PARITY_OVERFLOW, self.interrupt_enabled);
                        self.set_flag(SIGN, val & 0x80 > 0);
                    }
                    _ => {}
                }
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
                };
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
                self.cycles_to_next += 16;
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
            }
            Opcode::NOP => {
                self.cycles_to_next += 4;
            }
            Opcode::OR(mode) => {
                let result = self.a[self.af_bank] | self.read_byte(mode).unwrap();
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, false);
                self.set_flag(ZERO, result == 0);
                self.set_flag(PARITY_OVERFLOW, result & 0b1 == 0);
                self.set_flag(SIGN, result & 0x80 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += Self::arithmetic_cycles(mode);
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
            Opcode::RLCA => {
                let result = self.a[self.af_bank].rotate_left(1);
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, result & 0x1 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 4;
            }
            Opcode::RRCA => {
                let result = self.a[self.af_bank].rotate_right(1);
                self.a[self.af_bank] = result;
                self.set_flag(CARRY, result & 0x80 > 0);
                self.set_flag(SUBTRACT, false);
                self.set_flag(HALF_CARRY, false);
                self.cycles_to_next += 4;
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

    fn af(&mut self) -> u16 {
        ((self.a[self.af_bank] as u16) << 8) | (self.f[self.af_bank] as u16)
    }

    fn arithmetic_cycles(mode: AddrMode) -> u16 {
        match mode {
            AddrMode::Register(_) => 4,
            AddrMode::Immediate | AddrMode::RegisterIndirect(_) => 7,
            AddrMode::Indexed(_) => 19,
            AddrMode::Extended | AddrMode::RegisterPair(_) => panic!(),
        }
    }
}

#[cfg(feature = "test")]
#[allow(dead_code)]
pub mod testing {
    use std::borrow::Borrow;

    use gen::z80::Cpu;

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

        pub fn load_ram(&mut self, start: usize, src: &[u8]) {
            let mut ram = vec![0; start + src.len() + 0x100];
            ram[start..start + src.len()].copy_from_slice(src);
            self.sp = (ram.len() - 1) as u16;
            self.test_ram = Some(ram.into_boxed_slice());
            self.write_addr(0x5, 0xC9);
            self.write_word(0x6, self.sp);
        }

        pub fn output_test_string(&self) {
            let mut n = self.de[self.register_bank] as usize;
            let test_ram: &[u8] = self.test_ram.as_ref().unwrap().borrow();
            while test_ram[n] != '$' as u8 {
                print!("{}", test_ram[n] as char);
                n += 1;
            }
            println!()
        }
    }
}
