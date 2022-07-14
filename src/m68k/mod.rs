use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::Sub;

use input::ControllerState;
use m68k::opcodes::{AddressingMode, BitNum, brief_extension_word, Direction, opcode, Opcode, Size};

pub mod opcodes;

const CPU_TICKS_PER_SECOND: f64 = 7_670_454.0;

trait DataSize: TryFrom<u32> + Copy {
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

    fn is_negative(self) -> bool { panic!() }

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

    fn is_negative(self) -> bool { panic!() }

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

    fn is_negative(self) -> bool { panic!() }

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
    supervisor_mode: bool,
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

const INTERRUPT: u16 = 0b0000011100000000;
const INTERRUPT_SHIFT: u16 = 8;

impl<'a> Cpu<'a> {
    pub fn boot<'b>(instrumented: bool) -> Cpu<'b> {
        let mut cpu = Cpu {
            a: [0, 0, 0, 0, 0, 0, 0, 0],
            ssp: 0,
            d: [0, 0, 0, 0, 0, 0, 0, 0],
            status: 0,
            pc: 0,
            internal_ram: vec![0; 0x10000].into_boxed_slice(),
            supervisor_mode: true,
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
        if register == 7 && self.supervisor_mode {
            self.ssp
        } else {
            self.a[register]
        }
    }

    fn set_addr_register(&mut self, register: usize, val: u32) {
        if register == 7 && self.supervisor_mode {
            self.ssp = val
        } else {
            self.a[register] = val
        }
    }

    fn inc_addr_register(&mut self, register: usize, val: u32) {
        if register == 7 && self.supervisor_mode {
            self.ssp += val
        } else {
            self.a[register] += val
        }
    }

    fn dec_addr_register(&mut self, register: usize, val: u32) {
        if register == 7 && self.supervisor_mode {
            self.ssp -= val
        } else {
            self.a[register] -= val
        }
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
                    opcodes::Size::Word => self.read::<i16>(ext_mode) as i32,
                    opcodes::Size::Long => self.read::<i32>(ext_mode),
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
                    opcodes::Size::Word => self.read::<i16>(ext_mode) as i32,
                    opcodes::Size::Long => self.read::<i32>(ext_mode),
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
                    self.internal_ram.len().sub((-extension) as usize) as u32
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

    fn move_<Size: DataSize>(&mut self, src_mode: AddressingMode, dest_mode: AddressingMode) {
        let val: Size = self.read(src_mode);
        self.set_flag(NEGATIVE, val.is_negative());
        self.set_flag(ZERO, val.is_zero());
        self.set_flag(OVERFLOW, false);
        self.set_flag(CARRY, false);
        self.write(dest_mode, val);
    }

    fn execute_opcode(&mut self) {
        let opcode_pc = self.pc;
        let opcode_hex = self.read_addr(opcode_pc);
        self.pc += 2;

        let opcode = opcode(opcode_hex);

        match opcode {
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
            Opcode::JMP { mode } => self.pc = self.effective_addr(mode),
            Opcode::JSR { mode } => {
                let addr = self.effective_addr(mode);
                self.write(AddressingMode::AddressWithPredecrement(7), self.pc);
                self.pc = addr;
            }
            Opcode::MOVE { src_mode, dest_mode, size } => {
                match size {
                    Size::Byte => self.move_::<i8>(src_mode, dest_mode),
                    Size::Word => self.move_::<i16>(src_mode, dest_mode),
                    Size::Long => self.move_::<i32>(src_mode, dest_mode),
                    Size::Illegal => panic!()
                }
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
            Opcode::NOP => {}
            _ => {
                unimplemented!("{:04X} {:?}", opcode_hex, opcode)
            }
        }
    }

    pub fn next_operation(&mut self, inputs: &[ControllerState<8>; 2]) {
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
        self.supervisor_mode = true;
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

        pub fn verify_state(&self, pc: u32, sr: u16, d: [u32; 8], a: [u32; 8], ssp: u32) {
            assert_eq!(self.pc, pc, "PC");
            assert_eq!(self.status, sr, "SR {:016b} {:016b}", self.status, sr);
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