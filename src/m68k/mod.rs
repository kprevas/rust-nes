use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::Sub;

use num_traits::{PrimInt, Signed};

use input::ControllerState;
use m68k::opcodes::{AddressingMode, BitNum, brief_extension_word, Condition, Direction, ExchangeMode, opcode, Opcode, OperandDirection, Size};

pub mod opcodes;

const CPU_TICKS_PER_SECOND: f64 = 7_670_454.0;

trait DataSize: TryFrom<u32> + PrimInt {
    fn address_size() -> u32;
    fn word_aligned_address_size() -> u32;
    fn from_register_value(value: u32) -> Self;
    fn to_register_value(self) -> u32;
    fn from_memory_bytes(bytes: &[u8]) -> Self;
    fn set_memory_bytes(self, bytes: &mut [u8]);
    fn apply_to_register(self, register_val: u32) -> u32;
    fn is_negative(self) -> bool;
    fn is_zero(self) -> bool;
}

impl DataSize for u8 {
    fn address_size() -> u32 {
        1
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFF) as Self
    }

    fn to_register_value(self) -> u32 {
        self as u32
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }

    fn set_memory_bytes(self, bytes: &mut [u8]) {
        bytes[0] = self;
    }

    fn apply_to_register(self, register_val: u32) -> u32 {
        (register_val & !0xFF) + (self as u32)
    }

    fn is_negative(self) -> bool { self >> 7 == 1 }

    fn is_zero(self) -> bool { self == 0 }
}

impl DataSize for i8 {
    fn address_size() -> u32 {
        1
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFF) as Self
    }

    fn to_register_value(self) -> u32 {
        self as u32
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        bytes[0] as Self
    }

    fn set_memory_bytes(self, bytes: &mut [u8]) {
        bytes[0] = self as u8;
    }

    fn apply_to_register(self, register_val: u32) -> u32 {
        (register_val & !0xFF) + ((self as u8) as u32)
    }

    fn is_negative(self) -> bool { self < 0 }

    fn is_zero(self) -> bool { self == 0 }
}

impl DataSize for u16 {
    fn address_size() -> u32 {
        2
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFFFF) as Self
    }

    fn to_register_value(self) -> u32 {
        self as u32
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        ((bytes[0] as u16) << 8) | (bytes[1] as u16)
    }

    fn set_memory_bytes(self, bytes: &mut [u8]) {
        bytes[0] = ((self & 0xFF00) >> 8) as u8;
        bytes[1] = (self & 0xFF) as u8;
    }

    fn apply_to_register(self, register_val: u32) -> u32 {
        (register_val & !0xFFFF) + (self as u32)
    }

    fn is_negative(self) -> bool { self >> 15 == 1 }

    fn is_zero(self) -> bool { self == 0 }
}

impl DataSize for i16 {
    fn address_size() -> u32 { 2 }
    fn word_aligned_address_size() -> u32 { 2 }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFFFF) as Self
    }

    fn to_register_value(self) -> u32 {
        self as u32
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        (((bytes[0] as u16) << 8) | (bytes[1] as u16)) as i16
    }

    fn set_memory_bytes(self, bytes: &mut [u8]) {
        bytes[0] = (((self as u16) & 0xFF00) >> 8) as u8;
        bytes[1] = ((self as u16) & 0xFF) as u8;
    }

    fn apply_to_register(self, register_val: u32) -> u32 {
        (register_val & !0xFFFF) + ((self as u16) as u32)
    }

    fn is_negative(self) -> bool { self < 0 }

    fn is_zero(self) -> bool { self == 0 }
}

impl DataSize for u32 {
    fn address_size() -> u32 {
        4
    }
    fn word_aligned_address_size() -> u32 {
        4
    }

    fn from_register_value(value: u32) -> Self {
        value as Self
    }

    fn to_register_value(self) -> u32 {
        self
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8) | (bytes[3] as u32)
    }

    fn set_memory_bytes(self, bytes: &mut [u8]) {
        bytes[0] = ((self & 0xFF000000) >> 24) as u8;
        bytes[1] = ((self & 0xFF0000) >> 16) as u8;
        bytes[2] = ((self & 0xFF00) >> 8) as u8;
        bytes[3] = (self & 0xFF) as u8;
    }

    fn apply_to_register(self, _register_val: u32) -> u32 {
        self
    }

    fn is_negative(self) -> bool { self >> 31 == 1 }

    fn is_zero(self) -> bool { self == 0 }
}

impl DataSize for i32 {
    fn address_size() -> u32 { 4 }
    fn word_aligned_address_size() -> u32 { 4 }

    fn from_register_value(value: u32) -> Self {
        value as Self
    }

    fn to_register_value(self) -> u32 {
        self as u32
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        (((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8) | (bytes[3] as u32)) as i32
    }

    fn set_memory_bytes(self, bytes: &mut [u8]) {
        bytes[0] = (((self as u32) & 0xFF000000) >> 24) as u8;
        bytes[1] = (((self as u32) & 0xFF0000) >> 16) as u8;
        bytes[2] = (((self as u32) & 0xFF00) >> 8) as u8;
        bytes[3] = ((self as u32) & 0xFF) as u8;
    }

    fn apply_to_register(self, _register_val: u32) -> u32 {
        self as u32
    }

    fn is_negative(self) -> bool { self < 0 }

    fn is_zero(self) -> bool { self == 0 }
}

pub struct Cpu<'a> {
    a: [u32; 8],
    ssp: u32,
    d: [u32; 8],
    status: u16,
    pc: u32,
    internal_ram: Box<[u8]>,
    ticks: f64,
    instrumented: bool,
    cycle_count: u64,

    pub speed_adj: f64,

    phantom: PhantomData<&'a u8>,
}

const CARRY: u16 = 0b1;
const OVERFLOW: u16 = 0b10;
const ZERO: u16 = 0b100;
const NEGATIVE: u16 = 0b1000;
const EXTEND: u16 = 0b10000;

const SUPERVISOR_MODE: u16 = 0b0010000000000000;

const INTERRUPT: u16 = 0b0000011100000000;
const INTERRUPT_SHIFT: u16 = 8;

const SR_MASK: u16 = 0b1010011100011111;

impl<'a> Cpu<'a> {
    pub fn boot<'b>(instrumented: bool) -> Cpu<'b> {
        let mut cpu = Cpu {
            a: [0, 0, 0, 0, 0, 0, 0, 0],
            ssp: 0,
            d: [0, 0, 0, 0, 0, 0, 0, 0],
            status: 0,
            pc: 0,
            internal_ram: vec![0; 0x10000].into_boxed_slice(),
            ticks: 0.0,
            instrumented,
            cycle_count: 0,
            speed_adj: 1.0,
            phantom: PhantomData,
        };

        cpu.reset(false);

        cpu
    }

    fn tick(&mut self) {}

    fn flag(&self, flag: u16) -> bool {
        self.status & flag > 0
    }

    fn set_flag(&mut self, flag: u16, val: bool) {
        if val {
            self.status |= flag
        } else {
            self.status &= !flag;
        }
    }

    fn check_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::True => true,
            Condition::False => false,
            Condition::Higher => !self.flag(CARRY) && !self.flag(ZERO),
            Condition::LowerOrSame => self.flag(CARRY) || self.flag(ZERO),
            Condition::CarryClear => !self.flag(CARRY),
            Condition::CarrySet => self.flag(CARRY),
            Condition::NotEqual => !self.flag(ZERO),
            Condition::Equal => self.flag(ZERO),
            Condition::OverflowClear => !self.flag(OVERFLOW),
            Condition::OverflowSet => self.flag(OVERFLOW),
            Condition::Plus => !self.flag(NEGATIVE),
            Condition::Minus => self.flag(NEGATIVE),
            Condition::GreaterOrEqual => self.flag(NEGATIVE) == self.flag(OVERFLOW),
            Condition::LessThan => self.flag(NEGATIVE) != self.flag(OVERFLOW),
            Condition::GreaterThan => !self.flag(ZERO) && (self.flag(NEGATIVE) == self.flag(OVERFLOW)),
            Condition::LessOrEqual => self.flag(ZERO) || (self.flag(NEGATIVE) != self.flag(OVERFLOW)),
            Condition::Illegal => panic!()
        }
    }

    fn set_interrupt_level(&mut self, level: u16) {
        assert!(level <= (INTERRUPT >> INTERRUPT_SHIFT));
        self.status = (self.status & (!INTERRUPT)) | (level << INTERRUPT_SHIFT)
    }

    fn read_addr<Size: DataSize>(&mut self, addr: u32) -> Size {
        self.read_addr_no_tick(addr)
    }

    fn read_addr_word_aligned<Size: DataSize>(&mut self, addr: u32) -> Size {
        self.read_addr_word_aligned_no_tick(addr)
    }

    fn read_addr_no_tick<Size: DataSize>(&mut self, addr: u32) -> Size {
        let addr = addr & 0xFFFFFF;
        Size::from_memory_bytes(&self.internal_ram[
            (addr as usize)..((addr + Size::address_size()) as usize)])
    }

    fn read_addr_word_aligned_no_tick<Size: DataSize>(&mut self, addr: u32) -> Size {
        let addr = addr & 0xFFFFFF;
        let addr_size = Size::word_aligned_address_size();
        let addr_offset = Size::word_aligned_address_size() - Size::address_size();
        Size::from_memory_bytes(&self.internal_ram[
            ((addr + addr_offset) as usize)..((addr + addr_size) as usize)])
    }

    fn write_addr<Size: DataSize>(&mut self, addr: u32, val: Size) {
        self.write_addr_no_tick(addr, val);
    }

    fn write_addr_word_aligned<Size: DataSize>(&mut self, addr: u32, val: Size) {
        self.write_addr_word_aligned_no_tick(addr, val);
    }

    fn write_addr_no_tick<Size: DataSize>(&mut self, addr: u32, val: Size) {
        let addr = addr & 0xFFFFFF;
        val.set_memory_bytes(&mut self.internal_ram[
            (addr as usize)..((addr + Size::address_size()) as usize)])
    }

    fn write_addr_word_aligned_no_tick<Size: DataSize>(&mut self, addr: u32, val: Size) {
        let addr = addr & 0xFFFFFF;
        let addr_size = Size::word_aligned_address_size();
        let addr_offset = Size::word_aligned_address_size() - Size::address_size();
        val.set_memory_bytes(&mut self.internal_ram[
            ((addr + addr_offset) as usize)..((addr + addr_size) as usize)])
    }

    fn addr_register(&self, register: usize) -> u32 {
        if register == 7 && self.flag(SUPERVISOR_MODE) {
            self.ssp
        } else {
            self.a[register]
        }
    }

    fn set_addr_register(&mut self, register: usize, val: u32) {
        if register == 7 && self.flag(SUPERVISOR_MODE) {
            self.ssp = val
        } else {
            self.a[register] = val
        }
    }

    fn inc_addr_register(&mut self, register: usize, val: u32) {
        if register == 7 && self.flag(SUPERVISOR_MODE) {
            self.ssp += val
        } else {
            self.a[register] += val
        }
    }

    fn dec_addr_register(&mut self, register: usize, val: u32) {
        if register == 7 && self.flag(SUPERVISOR_MODE) {
            self.ssp -= val
        } else {
            self.a[register] -= val
        }
    }

    fn push<Size: DataSize>(&mut self, val: Size) {
        self.dec_addr_register(7, Size::address_size());
        self.write_addr(self.addr_register(7), val);
    }

    fn pop<Size: DataSize>(&mut self) -> Size {
        let val = self.read_addr(self.addr_register(7));
        self.inc_addr_register(7, Size::address_size());
        val
    }

    fn set_status(&mut self, status: u16) {
        self.status = status & SR_MASK;
    }

    fn effective_addr(&mut self, mode: AddressingMode) -> u32 {
        match mode {
            AddressingMode::Address(register) => self.addr_register(register),
            AddressingMode::AddressWithDisplacement(register) => {
                let displacement: i16 = self.read_addr::<u16>(self.pc) as i16;
                self.pc += 2;
                self.addr_register(register).wrapping_add_signed(displacement as i32)
            }
            AddressingMode::AddressWithIndex(register) => {
                let extension = self.read_addr::<u16>(self.pc);
                self.pc += 2;
                let (ext_mode, size, index) = brief_extension_word(extension);
                let ext_register_value = match size {
                    Size::Word => self.read::<i16>(ext_mode) as i32,
                    Size::Long => self.read::<i32>(ext_mode),
                    _ => panic!(),
                };
                self.addr_register(register)
                    .wrapping_add_signed(ext_register_value)
                    .wrapping_add_signed(index as i32)
            }
            AddressingMode::ProgramCounterWithDisplacement => {
                let displacement: i16 = self.read_addr::<u16>(self.pc) as i16;
                let addr = self.pc.wrapping_add_signed(displacement as i32);
                self.pc += 2;
                addr
            }
            AddressingMode::ProgramCounterWithIndex => {
                let extension = self.read_addr::<u16>(self.pc);
                let (ext_mode, size, index) = brief_extension_word(extension);
                let ext_register_value = match size {
                    Size::Word => self.read::<i16>(ext_mode) as i32,
                    Size::Long => self.read::<i32>(ext_mode),
                    _ => panic!(),
                };
                let addr = self
                    .pc
                    .wrapping_add_signed(ext_register_value)
                    .wrapping_add_signed(index as i32);
                self.pc += 2;
                addr
            }
            AddressingMode::AbsoluteShort => {
                let extension = self.read_addr::<i16>(self.pc);
                self.pc += 2;
                if extension < 0 {
                    u32::MAX.sub((-(extension as i32) - 1) as u32)
                } else {
                    extension as u32
                }
            }
            AddressingMode::AbsoluteLong => {
                let addr = self.read_addr::<u32>(self.pc);
                self.pc += 4;
                addr
            }
            _ => panic!()
        }
    }

    fn read<Size: DataSize>(&mut self, mode: AddressingMode) -> Size {
        match mode {
            AddressingMode::DataRegister(register) => Size::from_register_value(self.d[register]),
            AddressingMode::AddressRegister(register) => Size::from_register_value(self.addr_register(register)),
            AddressingMode::Address(_)
            | AddressingMode::AddressWithDisplacement(_)
            | AddressingMode::AddressWithIndex(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::AbsoluteShort
            | AddressingMode::AbsoluteLong
            => {
                let addr = self.effective_addr(mode);
                self.read_addr(addr)
            }
            AddressingMode::AddressWithPostincrement(register) => {
                let val = self.read_addr(self.addr_register(register));
                self.inc_addr_register(register, if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                });
                val
            }
            AddressingMode::AddressWithPredecrement(register) => {
                self.dec_addr_register(register, if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                });
                self.read_addr(self.addr_register(register))
            }
            AddressingMode::Immediate => {
                let val = self.read_addr_word_aligned::<Size>(self.pc);
                self.pc += Size::word_aligned_address_size();
                val
            }
            AddressingMode::Illegal => panic!(),
        }
    }

    fn read_extension<Size: DataSize>(&mut self) -> Size {
        let val = self.read_addr_word_aligned(self.pc);
        self.pc += Size::word_aligned_address_size();
        val
    }

    fn write<Size: DataSize>(&mut self, mode: AddressingMode, val: Size) {
        match mode {
            AddressingMode::DataRegister(register) => {
                self.d[register] = val.apply_to_register(self.d[register])
            }
            AddressingMode::AddressRegister(register) => {
                self.set_addr_register(register, val.apply_to_register(self.addr_register(register)));
            }
            AddressingMode::Address(_)
            | AddressingMode::AddressWithDisplacement(_)
            | AddressingMode::AddressWithIndex(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::AbsoluteShort
            | AddressingMode::AbsoluteLong
            => {
                let addr = self.effective_addr(mode);
                self.write_addr(addr, val);
            }
            AddressingMode::AddressWithPostincrement(register) => {
                let addr = self.addr_register(register);
                self.inc_addr_register(register, if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                });
                self.write_addr(addr, val);
            }
            AddressingMode::AddressWithPredecrement(register) => {
                self.dec_addr_register(register, if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                });
                self.write_addr(self.addr_register(register), val);
            }
            AddressingMode::Immediate | AddressingMode::Illegal => panic!(),
        }
    }

    fn read_write<Size: DataSize>(&mut self, mode: AddressingMode, op: &mut dyn FnMut(&mut Self, Size) -> Size) {
        match mode {
            AddressingMode::DataRegister(register) => {
                let val = Size::from_register_value(self.d[register]);
                let new_val = op(self, val);
                self.d[register] = new_val.apply_to_register(self.d[register])
            }
            AddressingMode::AddressRegister(register) => {
                let val = Size::from_register_value(self.addr_register(register));
                let new_val = op(self, val);
                self.set_addr_register(register, new_val.apply_to_register(self.addr_register(register)));
            }
            AddressingMode::Address(_)
            | AddressingMode::AddressWithDisplacement(_)
            | AddressingMode::AddressWithIndex(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::AbsoluteShort
            | AddressingMode::AbsoluteLong
            => {
                let addr = self.effective_addr(mode);
                let val = self.read_addr(addr);
                let new_val = op(self, val);
                self.write_addr(addr, new_val);
            }
            AddressingMode::AddressWithPostincrement(register) => {
                let addr = self.addr_register(register);
                let val = self.read_addr(addr);
                let new_val = op(self, val);
                self.write_addr(addr, new_val);
                self.inc_addr_register(register, if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                });
            }
            AddressingMode::AddressWithPredecrement(register) => {
                self.dec_addr_register(register, if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                });
                let addr = self.addr_register(register);
                let val = self.read_addr(addr);
                let new_val = op(self, val);
                self.write_addr(addr, new_val);
            }
            AddressingMode::Immediate | AddressingMode::Illegal => panic!(),
        }
    }

    fn process_exception(&mut self, vector: u32) {
        self.push(self.pc);
        self.push(self.status);
        self.pc = self.read_addr(vector * 4);
        self.set_flag(SUPERVISOR_MODE, true);
    }

    fn and<Size: DataSize>(&mut self, mode: AddressingMode, register: usize, operand_direction: OperandDirection) {
        let operand = Size::from_register_value(self.d[register]);
        match operand_direction {
            OperandDirection::ToRegister => {
                let val: Size = self.read(mode);
                let result = val & operand;
                self.set_flag(NEGATIVE, result.is_negative());
                self.set_flag(ZERO, result.is_zero());
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
                self.d[register] = result.apply_to_register(self.d[register]);
            }
            OperandDirection::ToMemory => {
                self.read_write(mode, &mut |cpu, val| {
                    let result: Size = val & operand;
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    cpu.set_flag(ZERO, result.is_zero());
                    cpu.set_flag(OVERFLOW, false);
                    cpu.set_flag(CARRY, false);
                    result
                });
            }
        }
    }

    fn andi<Size: DataSize>(&mut self, mode: AddressingMode) {
        let operand: Size = self.read_extension();
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let result = val & operand;
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, false);
            result
        });
    }

    fn branch(&mut self, displacement: i8, do_branch: bool) {
        let pc = self.pc;
        let displacement = if displacement == 0 {
            self.read_extension::<i16>() as i32
        } else {
            displacement as i32
        };
        if do_branch {
            self.pc = pc.wrapping_add_signed(displacement);
        }
    }

    fn eor<Size: DataSize>(&mut self, mode: AddressingMode, register: usize, operand_direction: OperandDirection) {
        let operand = Size::from_register_value(self.d[register]);
        match operand_direction {
            OperandDirection::ToRegister => {
                let val: Size = self.read(mode);
                let result = val ^ operand;
                self.set_flag(NEGATIVE, result.is_negative());
                self.set_flag(ZERO, result.is_zero());
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
                self.d[register] = result.apply_to_register(self.d[register]);
            }
            OperandDirection::ToMemory => {
                self.read_write(mode, &mut |cpu, val| {
                    let result: Size = val ^ operand;
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    cpu.set_flag(ZERO, result.is_zero());
                    cpu.set_flag(OVERFLOW, false);
                    cpu.set_flag(CARRY, false);
                    result
                });
            }
        }
    }

    fn eori<Size: DataSize>(&mut self, mode: AddressingMode) {
        let operand: Size = self.read_extension();
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let result = val ^ operand;
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, false);
            result
        });
    }

    fn ext<From: DataSize, To: DataSize>(&mut self, register: usize) {
        let val = To::from(From::from_register_value(self.d[register])).unwrap();
        self.d[register] = val.apply_to_register(self.d[register]);
        self.set_flag(NEGATIVE, val.is_negative());
        self.set_flag(ZERO, val.is_zero());
    }

    fn move_<Size: DataSize>(&mut self, src_mode: AddressingMode, dest_mode: AddressingMode) {
        let val: Size = self.read(src_mode);
        self.set_flag(NEGATIVE, val.is_negative());
        self.set_flag(ZERO, val.is_zero());
        self.set_flag(OVERFLOW, false);
        self.set_flag(CARRY, false);
        self.write(dest_mode, val);
    }

    fn neg<Size: DataSize + Signed>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let overflow = val == Size::min_value();
            let result = if overflow { val } else { -val };
            cpu.set_flag(EXTEND, !result.is_zero());
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, overflow);
            cpu.set_flag(CARRY, !result.is_zero());
            result
        });
    }

    fn negx<Size: DataSize + Signed>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let overflow = val == Size::min_value()
                || (val == Size::max_value() && cpu.flag(EXTEND));
            let result = if overflow {
                val
            } else if cpu.flag(EXTEND) {
                -val - Size::from(1).unwrap()
            } else {
                -val
            };
            cpu.set_flag(EXTEND, !result.is_zero());
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, overflow);
            cpu.set_flag(CARRY, !result.is_zero());
            result
        });
    }

    fn not<Size: DataSize>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let result = !val;
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, false);
            result
        });
    }

    fn or<Size: DataSize>(&mut self, mode: AddressingMode, register: usize, operand_direction: OperandDirection) {
        let operand = Size::from_register_value(self.d[register]);
        match operand_direction {
            OperandDirection::ToRegister => {
                let val: Size = self.read(mode);
                let result = val | operand;
                self.set_flag(NEGATIVE, result.is_negative());
                self.set_flag(ZERO, result.is_zero());
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
                self.d[register] = result.apply_to_register(self.d[register]);
            }
            OperandDirection::ToMemory => {
                self.read_write(mode, &mut |cpu, val| {
                    let result: Size = val | operand;
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    cpu.set_flag(ZERO, result.is_zero());
                    cpu.set_flag(OVERFLOW, false);
                    cpu.set_flag(CARRY, false);
                    result
                });
            }
        }
    }

    fn ori<Size: DataSize>(&mut self, mode: AddressingMode) {
        let operand: Size = self.read_extension();
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let result = val | operand;
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, false);
            result
        });
    }

    fn tas(&mut self, mode: AddressingMode) {
        self.read_write::<u8>(mode, &mut |cpu, val| {
            cpu.set_flag(NEGATIVE, (val as i8).is_negative());
            cpu.set_flag(ZERO, val.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, false);
            val | 0b10000000
        });
    }

    fn tst<Size: DataSize>(&mut self, mode: AddressingMode) {
        let val: Size = self.read(mode);
        self.set_flag(NEGATIVE, val.is_negative());
        self.set_flag(ZERO, val.is_zero());
        self.set_flag(OVERFLOW, false);
        self.set_flag(CARRY, false);
    }

    fn execute_opcode(&mut self) {
        let opcode_pc = self.pc;
        let opcode_hex = self.read_addr(opcode_pc);
        self.pc += 2;

        let opcode = opcode(opcode_hex);

        match opcode {
            Opcode::AND { mode, size, operand_direction, register } => match size {
                Size::Byte => self.and::<u8>(mode, register, operand_direction),
                Size::Word => self.and::<u16>(mode, register, operand_direction),
                Size::Long => self.and::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!()
            }
            Opcode::ANDI { mode, size } => match size {
                Size::Byte => self.andi::<u8>(mode),
                Size::Word => self.andi::<u16>(mode),
                Size::Long => self.andi::<u32>(mode),
                Size::Illegal => panic!()
            },
            Opcode::ANDI_to_CCR => {
                let new_status = (self.status & 0xFF00)
                    | ((self.status & 0xFF) & (self.read_extension::<u8>()) as u16);
                self.set_status(new_status);
            }
            Opcode::ANDI_to_SR => {
                let new_status = self.status & self.read_extension::<u16>();
                self.set_status(new_status);
            }
            Opcode::BCHG { bit_num, mode }
            | Opcode::BCLR { bit_num, mode }
            | Opcode::BSET { bit_num, mode }
            | Opcode::BTST { bit_num, mode } => {
                let bit_mod = match mode {
                    AddressingMode::DataRegister(_) => 32,
                    _ => 8,
                };
                let bit = match bit_num {
                    BitNum::Immediate => {
                        let extension: u8 = self.read_addr_word_aligned(self.pc);
                        self.pc += 2;
                        extension as u32
                    }
                    BitNum::DataRegister(register) => self.d[register]
                } % bit_mod;
                if let Opcode::BTST { .. } = opcode {
                    match mode {
                        AddressingMode::DataRegister(_) => {
                            let val = self.read::<u32>(mode);
                            let bit_val = (val >> bit) & 0b1;
                            self.set_flag(ZERO, bit_val == 0);
                        }
                        _ => {
                            let val = self.read::<u8>(mode);
                            let bit_val = (val >> bit) & 0b1;
                            self.set_flag(ZERO, bit_val == 0);
                        }
                    };
                } else {
                    match mode {
                        AddressingMode::DataRegister(_) => self.read_write::<u32>(mode, &mut |cpu, val| {
                            let bit_val = (val >> bit) & 0b1;
                            cpu.set_flag(ZERO, bit_val == 0);
                            match opcode {
                                Opcode::BCHG { .. } => val ^ (1 << bit),
                                Opcode::BCLR { .. } => val & !(1 << bit),
                                Opcode::BSET { .. } => val | (1 << bit),
                                _ => panic!()
                            }
                        }),
                        _ => self.read_write::<u8>(mode, &mut |cpu, val| {
                            let bit_val = (val >> bit) & 0b1;
                            cpu.set_flag(ZERO, bit_val == 0);
                            match opcode {
                                Opcode::BCHG { .. } => val ^ (1 << bit),
                                Opcode::BCLR { .. } => val & !(1 << bit),
                                Opcode::BSET { .. } => val | (1 << bit),
                                _ => panic!()
                            }
                        }),
                    };
                }
            }
            Opcode::BRA { displacement } => {
                self.branch(displacement, true);
            }
            Opcode::BSR { displacement } => {
                self.push(self.pc + if displacement == 0 { 2 } else { 0 });
                self.branch(displacement, true);
            }
            Opcode::Bcc { displacement, condition } => {
                self.branch(displacement, self.check_condition(condition));
            }
            Opcode::CHK { register, mode } => {
                let bound = self.read::<i16>(mode);
                let val = i16::from_register_value(self.d[register]);
                self.set_flag(ZERO, val == 0);
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
                if val < 0 {
                    self.set_flag(NEGATIVE, true);
                    self.process_exception(6);
                } else if val > bound {
                    self.set_flag(NEGATIVE, false);
                    self.process_exception(6);
                }
            }
            Opcode::CLR { mode, size } => {
                match size {
                    Size::Byte => self.write::<u8>(mode, 0),
                    Size::Word => self.write::<u16>(mode, 0),
                    Size::Long => self.write::<u32>(mode, 0),
                    Size::Illegal => panic!(),
                }
                self.set_flag(NEGATIVE, false);
                self.set_flag(ZERO, true);
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
            }
            Opcode::EOR { mode, size, operand_direction, register } => match size {
                Size::Byte => self.eor::<u8>(mode, register, operand_direction),
                Size::Word => self.eor::<u16>(mode, register, operand_direction),
                Size::Long => self.eor::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!()
            },
            Opcode::EORI { mode, size } => match size {
                Size::Byte => self.eori::<u8>(mode),
                Size::Word => self.eori::<u16>(mode),
                Size::Long => self.eori::<u32>(mode),
                Size::Illegal => panic!()
            },
            Opcode::EORI_to_CCR => {
                let new_status = (self.status & 0xFF00)
                    | ((self.status & 0xFF) ^ (self.read_extension::<u8>()) as u16);
                self.set_status(new_status);
            }
            Opcode::EORI_to_SR => {
                let new_status = self.status ^ self.read_extension::<u16>();
                self.set_status(new_status);
            }
            Opcode::EXG { mode, src_register, dest_register } => {
                match mode {
                    ExchangeMode::DataRegisters => {
                        let tmp = self.d[src_register];
                        self.d[src_register] = self.d[dest_register];
                        self.d[dest_register] = tmp;
                    }
                    ExchangeMode::AddressRegisters => {
                        let tmp = self.addr_register(src_register);
                        self.set_addr_register(src_register, self.addr_register(dest_register));
                        self.set_addr_register(dest_register, tmp);
                    }
                    ExchangeMode::DataRegisterAndAddressRegister => {
                        let tmp = self.d[src_register];
                        self.d[src_register] = self.addr_register(dest_register);
                        self.set_addr_register(dest_register, tmp);
                    }
                    ExchangeMode::Illegal => panic!()
                };
            }
            Opcode::EXT { register, size } => {
                match size {
                    Size::Word => self.ext::<i8, i16>(register),
                    Size::Long => self.ext::<i16, i32>(register),
                    Size::Byte | Size::Illegal => panic!(),
                }
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
            }
            Opcode::ILLEGAL => self.process_exception(4),
            Opcode::JMP { mode } => self.pc = self.effective_addr(mode),
            Opcode::JSR { mode } => {
                let addr = self.effective_addr(mode);
                self.write(AddressingMode::AddressWithPredecrement(7), self.pc);
                self.pc = addr;
            }
            Opcode::LEA { register, mode } => {
                let val = self.effective_addr(mode);
                self.set_addr_register(register, val);
            }
            Opcode::LINK { register } => {
                self.push(self.addr_register(register) - if register == 7 { 4 } else { 0 });
                self.set_addr_register(register, self.addr_register(7));
                let displacement = self.read_extension::<i16>();
                self.set_addr_register(7,
                                       self.addr_register(7).wrapping_add_signed(displacement as i32));
            }
            Opcode::MOVE { src_mode, dest_mode, size } => match size {
                Size::Byte => self.move_::<i8>(src_mode, dest_mode),
                Size::Word => self.move_::<i16>(src_mode, dest_mode),
                Size::Long => self.move_::<i32>(src_mode, dest_mode),
                Size::Illegal => panic!()
            },
            Opcode::MOVEA { src_mode, dest_mode, size } => match size {
                Size::Word => self.move_::<i16>(src_mode, dest_mode),
                Size::Long => self.move_::<i32>(src_mode, dest_mode),
                Size::Byte | Size::Illegal => panic!()
            }
            Opcode::MOVEP { data_register, address_register, direction, size } => {
                let addr = self.effective_addr(AddressingMode::AddressWithDisplacement(address_register));
                match direction {
                    Direction::RegisterToMemory => {
                        let val = self.d[data_register];
                        match size {
                            Size::Word => {
                                self.write_addr(addr, ((val >> 8) & 0xFF) as u8);
                                self.write_addr(addr + 2, (val & 0xFF) as u8);
                            }
                            Size::Long => {
                                self.write_addr(addr, ((val >> 24) & 0xFF) as u8);
                                self.write_addr(addr + 2, ((val >> 16) & 0xFF) as u8);
                                self.write_addr(addr + 4, ((val >> 8) & 0xFF) as u8);
                                self.write_addr(addr + 6, (val & 0xFF) as u8);
                            }
                            _ => panic!()
                        }
                    }
                    Direction::MemoryToRegister => {
                        match size {
                            Size::Word => {
                                let mut val = 0;
                                val += (self.read_addr::<u8>(addr) as u16) << 8;
                                val += self.read_addr::<u8>(addr + 2) as u16;
                                self.d[data_register] = val.apply_to_register(self.d[data_register]);
                            }
                            Size::Long => {
                                let mut val = 0;
                                val += (self.read_addr::<u8>(addr) as u32) << 24;
                                val += (self.read_addr::<u8>(addr + 2) as u32) << 16;
                                val += (self.read_addr::<u8>(addr + 4) as u32) << 8;
                                val += self.read_addr::<u8>(addr + 6) as u32;
                                self.d[data_register] = val.apply_to_register(self.d[data_register]);
                            }
                            _ => panic!()
                        }
                    }
                }
            }
            Opcode::MOVEQ { register, data } => {
                self.set_flag(NEGATIVE, data < 0);
                self.set_flag(ZERO, data == 0);
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
                self.d[register] = data as u32;
            }
            Opcode::MOVE_to_CCR { mode } => {
                let val = self.read::<u16>(mode);
                let low_byte = val & 0xFF;
                self.set_status((self.status & 0xFF00) | low_byte);
            }
            Opcode::MOVE_to_SR { mode } => {
                let val = self.read::<u16>(mode);
                self.set_status(val);
            }
            Opcode::MOVE_from_SR { mode } => {
                self.write(mode, self.status);
            }
            Opcode::MOVE_USP { register, direction } => match direction {
                Direction::RegisterToMemory => self.set_addr_register(register, self.a[7]),
                Direction::MemoryToRegister => self.a[7] = self.addr_register(register),
            }
            Opcode::NEG { mode, size } => match size {
                Size::Byte => self.neg::<i8>(mode),
                Size::Word => self.neg::<i16>(mode),
                Size::Long => self.neg::<i32>(mode),
                Size::Illegal => panic!()
            }
            Opcode::NEGX { mode, size } => match size {
                Size::Byte => self.negx::<i8>(mode),
                Size::Word => self.negx::<i16>(mode),
                Size::Long => self.negx::<i32>(mode),
                Size::Illegal => panic!()
            }
            Opcode::NOP => {}
            Opcode::NOT { mode, size } => match size {
                Size::Byte => self.not::<u8>(mode),
                Size::Word => self.not::<u16>(mode),
                Size::Long => self.not::<u32>(mode),
                Size::Illegal => panic!()
            },
            Opcode::OR { mode, size, operand_direction, register } => match size {
                Size::Byte => self.or::<u8>(mode, register, operand_direction),
                Size::Word => self.or::<u16>(mode, register, operand_direction),
                Size::Long => self.or::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!()
            },
            Opcode::ORI { mode, size } => match size {
                Size::Byte => self.ori::<u8>(mode),
                Size::Word => self.ori::<u16>(mode),
                Size::Long => self.ori::<u32>(mode),
                Size::Illegal => panic!()
            },
            Opcode::ORI_to_CCR => {
                let new_status = (self.status & 0xFF00)
                    | ((self.status & 0xFF) | (self.read_extension::<u8>()) as u16);
                self.set_status(new_status);
            }
            Opcode::ORI_to_SR => {
                let new_status = self.status | self.read_extension::<u16>();
                self.set_status(new_status);
            }
            Opcode::SWAP { register } => {
                let val = self.d[register];
                let result = (val << 16) | (val >> 16);
                self.d[register] = result;
                self.set_flag(ZERO, result == 0);
                self.set_flag(NEGATIVE, result >> 31 == 1);
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
            }
            Opcode::TAS { mode } => self.tas(mode),
            Opcode::TST { mode, size } => match size {
                Size::Byte => self.tst::<i8>(mode),
                Size::Word => self.tst::<i16>(mode),
                Size::Long => self.tst::<i32>(mode),
                Size::Illegal => panic!()
            }
            Opcode::UNLK { register } => {
                self.set_addr_register(7, self.addr_register(register));
                let val = self.pop();
                self.set_addr_register(register, val);
            }
            _ => {
                unimplemented!("{:04X} {:?}", opcode_hex, opcode)
            }
            // Opcode::ABCD { .. } => {}
            // Opcode::ADD { .. } => {}
            // Opcode::ADDA { .. } => {}
            // Opcode::ADDI { .. } => {}
            // Opcode::ADDQ { .. } => {}
            // Opcode::ADDX { .. } => {}
            // Opcode::ASL { .. } => {}
            // Opcode::ASR { .. } => {}
            // Opcode::CMP { .. } => {}
            // Opcode::CMPA { .. } => {}
            // Opcode::CMPI { .. } => {}
            // Opcode::CMPM { .. } => {}
            // Opcode::DBcc { .. } => {}
            // Opcode::DIVS { .. } => {}
            // Opcode::DIVU { .. } => {}
            // Opcode::LSL { .. } => {}
            // Opcode::LSR { .. } => {}
            // Opcode::MOVEM { .. } => {}
            // Opcode::MULS { .. } => {}
            // Opcode::MULU { .. } => {}
            // Opcode::NBCD { .. } => {}
            // Opcode::PEA { .. } => {}
            // Opcode::RESET => {}
            // Opcode::ROL { .. } => {}
            // Opcode::ROR { .. } => {}
            // Opcode::ROXL { .. } => {}
            // Opcode::ROXR { .. } => {}
            // Opcode::RTE => {}
            // Opcode::RTR => {}
            // Opcode::RTS => {}
            // Opcode::SBCD { .. } => {}
            // Opcode::Scc { .. } => {}
            // Opcode::STOP => {}
            // Opcode::SUB { .. } => {}
            // Opcode::SUBA { .. } => {}
            // Opcode::SUBI { .. } => {}
            // Opcode::SUBQ { .. } => {}
            // Opcode::SUBX { .. } => {}
            // Opcode::TRAP { .. } => {}
            // Opcode::TRAPV => {}
        }
    }

    pub fn next_operation(&mut self, _inputs: &[ControllerState<8>; 2]) {
        self.execute_opcode();
    }

    pub fn do_frame(&mut self, time_secs: f64, inputs: &[ControllerState<8>; 2]) {
        self.ticks += time_secs * CPU_TICKS_PER_SECOND * self.speed_adj;

        while self.ticks > 0.0 {
            self.next_operation(inputs);
        }
    }

    pub fn reset(&mut self, _soft: bool) {
        self.ssp = self.read_addr_no_tick(0x000000);
        self.pc = self.read_addr_no_tick(0x000004);
        self.set_interrupt_level(7);
        self.set_flag(SUPERVISOR_MODE, true);
    }

    pub fn close(&mut self) {}
}

#[cfg(feature = "test")]
#[allow(dead_code)]
pub mod testing {
    use m68k::Cpu;
    use m68k::opcodes::{Opcode, opcode};

    impl Cpu<'_> {
        pub fn expand_ram(&mut self, amount: usize) {
            self.internal_ram = vec![0; amount].into_boxed_slice();
        }

        pub fn init_state(&mut self, pc: u32, sr: u16, d: [u32; 8], a: [u32; 8], ssp: u32) {
            self.pc = pc;
            self.status = sr;
            self.d = d;
            self.a = a;
            self.ssp = ssp;
        }

        pub fn verify_state(&self, pc: u32, sr: u16, d: [u32; 8], a: [u32; 8], ssp: u32,
                            sr_mask: u16) {
            assert_eq!(self.pc, pc, "PC");
            assert_eq!(self.status & sr_mask, sr & sr_mask, "SR {:016b} {:016b}", self.status, sr);
            for i in 0..8 {
                assert_eq!(self.d[i], d[i], "D{}", i);
                assert_eq!(self.a[i], a[i], "A{}", i);
            }
            assert_eq!(self.ssp, ssp, "SSP");
        }

        pub fn poke_ram(&mut self, addr: usize, val: u8) {
            self.internal_ram[addr] = val;
        }

        pub fn peek_opcode(&mut self) -> Opcode {
            opcode(self.read_addr(self.pc))
        }

        pub fn verify_ram(&self, addr: usize, val: u8) {
            assert_eq!(self.internal_ram[addr & 0xFFFFFF], val, "{:06X}", addr);
        }
    }
}