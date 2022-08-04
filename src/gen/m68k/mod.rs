use std::cell::RefCell;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::{AddAssign, Shl, Shr, Sub, SubAssign};

use bytes::Buf;
use num_integer::Integer;
use num_traits::{PrimInt, Signed, WrappingAdd, WrappingSub};
use piston_window::*;

use gen::m68k::opcodes::{
    AddressingMode, BitNum, brief_extension_word, Condition, Direction, ExchangeMode, opcode,
    Opcode, OperandDirection, OperandMode, Size,
};
use gen::vdp::bus::VdpBus;
use gen::vdp::Vdp;
use gfx_device_gl::Device;
use input::ControllerState;
use window;
use window::Cpu as wcpu;

pub mod opcodes;

const CPU_TICKS_PER_SECOND: f64 = 7_670_454.0;

trait DataSize: TryFrom<u32> + PrimInt {
    fn address_size() -> u32;
    fn bits() -> usize;
    fn word_aligned_address_size() -> u32;
    fn from_register_value(value: u32) -> Self;
    fn to_register_value(self) -> u32;
    fn from_byte(byte: u8) -> Self;
    fn from_memory_bytes(bytes: &[u8]) -> Self;
    fn set_memory_bytes(self, bytes: &mut [u8]);
    fn apply_to_register(self, register_val: u32) -> u32;
    fn is_negative(self) -> bool;
    fn is_zero(self) -> bool;
    fn read_from_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32) -> Self;
    fn write_to_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32, val: Self);
}

impl DataSize for u8 {
    fn address_size() -> u32 {
        1
    }
    fn bits() -> usize {
        8
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFF) as Self
    }

    fn to_register_value(self) -> u32 {
        ((self as i8) as i32) as u32
    }

    fn from_byte(byte: u8) -> Self {
        byte as Self
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

    fn is_negative(self) -> bool {
        self >> 7 == 1
    }

    fn is_zero(self) -> bool {
        self == 0
    }

    fn read_from_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32) -> Self {
        vdp_bus.borrow_mut().read_byte(addr)
    }

    fn write_to_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32, val: Self) {
        vdp_bus.borrow_mut().write_byte(addr, val);
    }
}

impl DataSize for i8 {
    fn address_size() -> u32 {
        1
    }
    fn bits() -> usize {
        8
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFF) as Self
    }

    fn to_register_value(self) -> u32 {
        (self as i32) as u32
    }

    fn from_byte(byte: u8) -> Self {
        byte as Self
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

    fn is_negative(self) -> bool {
        self < 0
    }

    fn is_zero(self) -> bool {
        self == 0
    }

    fn read_from_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32) -> Self {
        vdp_bus.borrow_mut().read_byte(addr) as Self
    }

    fn write_to_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32, val: Self) {
        vdp_bus.borrow_mut().write_byte(addr, val as u8);
    }
}

impl DataSize for u16 {
    fn address_size() -> u32 {
        2
    }
    fn bits() -> usize {
        16
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFFFF) as Self
    }

    fn to_register_value(self) -> u32 {
        ((self as i16) as i32) as u32
    }

    fn from_byte(byte: u8) -> Self {
        byte as Self
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

    fn is_negative(self) -> bool {
        self >> 15 == 1
    }

    fn is_zero(self) -> bool {
        self == 0
    }

    fn read_from_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32) -> Self {
        vdp_bus.borrow_mut().read_word(addr)
    }

    fn write_to_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32, val: Self) {
        vdp_bus.borrow_mut().write_word(addr, val);
    }
}

impl DataSize for i16 {
    fn address_size() -> u32 {
        2
    }
    fn bits() -> usize {
        16
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn from_register_value(value: u32) -> Self {
        (value & 0xFFFF) as Self
    }

    fn to_register_value(self) -> u32 {
        (self as i32) as u32
    }

    fn from_byte(byte: u8) -> Self {
        byte as Self
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

    fn is_negative(self) -> bool {
        self < 0
    }

    fn is_zero(self) -> bool {
        self == 0
    }

    fn read_from_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32) -> Self {
        vdp_bus.borrow_mut().read_word(addr) as Self
    }

    fn write_to_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32, val: Self) {
        vdp_bus.borrow_mut().write_word(addr, val as u16);
    }
}

impl DataSize for u32 {
    fn address_size() -> u32 {
        4
    }
    fn bits() -> usize {
        32
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

    fn from_byte(byte: u8) -> Self {
        byte as Self
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        ((bytes[0] as u32) << 24)
            | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8)
            | (bytes[3] as u32)
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

    fn is_negative(self) -> bool {
        self >> 31 == 1
    }

    fn is_zero(self) -> bool {
        self == 0
    }

    fn read_from_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32) -> Self {
        vdp_bus.borrow_mut().read_long(addr)
    }

    fn write_to_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32, val: Self) {
        vdp_bus.borrow_mut().write_long(addr, val);
    }
}

impl DataSize for i32 {
    fn address_size() -> u32 {
        4
    }
    fn bits() -> usize {
        32
    }
    fn word_aligned_address_size() -> u32 {
        4
    }

    fn from_register_value(value: u32) -> Self {
        value as Self
    }

    fn to_register_value(self) -> u32 {
        self as u32
    }

    fn from_byte(byte: u8) -> Self {
        byte as Self
    }

    fn from_memory_bytes(bytes: &[u8]) -> Self {
        (((bytes[0] as u32) << 24)
            | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8)
            | (bytes[3] as u32)) as i32
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

    fn is_negative(self) -> bool {
        self < 0
    }

    fn is_zero(self) -> bool {
        self == 0
    }

    fn read_from_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32) -> Self {
        vdp_bus.borrow_mut().read_long(addr) as Self
    }

    fn write_to_vdp_bus(vdp_bus: &RefCell<VdpBus>, addr: u32, val: Self) {
        vdp_bus.borrow_mut().write_long(addr, val as u32);
    }
}

pub struct Cpu<'a> {
    a: [u32; 8],
    ssp: u32,
    d: [u32; 8],
    status: u16,
    pc: u32,
    cartridge: &'a Box<[u8]>,
    internal_ram: Box<[u8]>,
    ticks: f64,
    instrumented: bool,
    cycle_count: u64,
    stopped: bool,

    pub speed_adj: f64,

    vdp: Vdp<'a>,
    vdp_bus: &'a RefCell<VdpBus>,

    test_ram_only: bool,

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
    pub fn boot<'b>(
        cartridge: &'b Box<[u8]>,
        vdp: Vdp<'b>,
        vdp_bus: &'b RefCell<VdpBus>,
        instrumented: bool,
    ) -> Cpu<'b> {
        let mut cpu = Cpu {
            a: [0, 0, 0, 0, 0, 0, 0, 0],
            ssp: 0,
            d: [0, 0, 0, 0, 0, 0, 0, 0],
            status: 0,
            pc: 0,
            cartridge,
            internal_ram: vec![0; 0x10000].into_boxed_slice(),
            ticks: 0.0,
            instrumented,
            cycle_count: 0,
            stopped: false,
            speed_adj: 1.0,
            vdp,
            vdp_bus,
            test_ram_only: false,
            phantom: PhantomData,
        };

        cpu.reset(false);

        cpu
    }

    fn tick(&mut self, cycle_count: u8) {
        for _ in 0..cycle_count {
            self.vdp.cpu_tick(&self.cartridge, &self.internal_ram);
            self.ticks -= 1.0;
            self.cycle_count = self.cycle_count.wrapping_add(1);
        }
    }

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
            Condition::GreaterThan => {
                !self.flag(ZERO) && (self.flag(NEGATIVE) == self.flag(OVERFLOW))
            }
            Condition::LessOrEqual => {
                self.flag(ZERO) || (self.flag(NEGATIVE) != self.flag(OVERFLOW))
            }
            Condition::Illegal => panic!(),
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
        self.read_addr_offset_size(addr & 0xFFFFFF, 0, Size::word_aligned_address_size())
    }

    fn read_addr_word_aligned_no_tick<Size: DataSize>(&mut self, addr: u32) -> Size {
        self.read_addr_offset_size(
            addr & 0xFFFFFF,
            Size::word_aligned_address_size() - Size::address_size(),
            Size::word_aligned_address_size(),
        )
    }

    fn read_addr_offset_size<Size: DataSize>(&mut self, addr: u32, offset: u32, size: u32) -> Size {
        if self.test_ram_only {
            Size::from_memory_bytes(
                &self.internal_ram[((addr + offset) as usize)..((addr + size) as usize)],
            )
        } else {
            match addr {
                0x000000..=0x3FFFFF => Size::from_memory_bytes(
                    &self.cartridge[((addr + offset) as usize)..((addr + size) as usize)],
                ),
                0x400000..=0x7FFFFF => Size::from(0).unwrap(), // Expansion port
                0xA00000..=0xA0FFFF => Size::from(0).unwrap(), // Z80 Area
                0xA10001 => Size::from_byte(0b10100000),
                0xA10000..=0xA10FFF => Size::from(0).unwrap(), // IO Registers
                0xA11000..=0xA11FFF => Size::from(0).unwrap(), // Z80 Control
                0xC00000..=0xDFFFFF => Size::read_from_vdp_bus(self.vdp_bus, addr),
                0xE00000..=0xFFFFFF => {
                    let ram_addr = addr & 0xFFFF;
                    Size::from_memory_bytes(
                        &self.internal_ram
                            [((ram_addr + offset) as usize)..((ram_addr + size) as usize)],
                    )
                }
                _ => panic!(),
            }
        }
    }

    fn write_addr<Size: DataSize>(&mut self, addr: u32, val: Size) {
        self.write_addr_no_tick(addr, val);
    }

    fn write_addr_no_tick<Size: DataSize>(&mut self, addr: u32, val: Size) {
        self.write_addr_offset_size(addr & 0xFFFFFF, 0, Size::address_size(), val);
    }

    fn write_addr_offset_size<Size: DataSize>(
        &mut self,
        addr: u32,
        offset: u32,
        size: u32,
        val: Size,
    ) {
        if self.test_ram_only {
            val.set_memory_bytes(
                &mut self.internal_ram[((addr + offset) as usize)..((addr + size) as usize)],
            );
        } else {
            match addr {
                0x000000..=0x3FFFFF => {} // Vector table, ROM Cartridge
                0x400000..=0x7FFFFF => {} // Expansion port
                0xA00000..=0xA0FFFF => {} // Z80 Area
                0xA10000..=0xA10FFF => {} // IO Registers
                0xA11000..=0xA11FFF => {} // Z80 Control
                0xC00000..=0xDFFFFF => Size::write_to_vdp_bus(self.vdp_bus, addr, val),
                0xE00000..=0xFFFFFF => {
                    let ram_addr = addr & 0xFFFF;
                    val.set_memory_bytes(
                        &mut self.internal_ram
                            [((ram_addr + offset) as usize)..((ram_addr + size) as usize)],
                    );
                }
                _ => panic!(),
            }
        }
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

    fn inc_addr_register<Size: DataSize>(&mut self, register: usize) {
        let val = if register == 7 {
            Size::word_aligned_address_size()
        } else {
            Size::address_size()
        };
        if register == 7 && self.flag(SUPERVISOR_MODE) {
            self.ssp = self.ssp.wrapping_add(val);
        } else {
            self.a[register] = self.a[register].wrapping_add(val);
        }
    }

    fn dec_addr_register<Size: DataSize>(&mut self, register: usize) {
        let val = if register == 7 {
            Size::word_aligned_address_size()
        } else {
            Size::address_size()
        };
        if register == 7 && self.flag(SUPERVISOR_MODE) {
            self.ssp = self.ssp.wrapping_sub(val);
        } else {
            self.a[register] = self.a[register].wrapping_sub(val);
        }
    }

    fn push<Size: DataSize>(&mut self, val: Size) {
        self.dec_addr_register::<Size>(7);
        self.write_addr(self.addr_register(7), val);
    }

    fn pop<Size: DataSize>(&mut self) -> Size {
        let val = self.read_addr(self.addr_register(7));
        self.inc_addr_register::<Size>(7);
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
                self.addr_register(register)
                    .wrapping_add_signed(displacement as i32)
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
            _ => panic!(),
        }
    }

    fn read<Size: DataSize>(&mut self, mode: AddressingMode) -> Size {
        match mode {
            AddressingMode::DataRegister(register) => Size::from_register_value(self.d[register]),
            AddressingMode::AddressRegister(register) => {
                Size::from_register_value(self.addr_register(register))
            }
            AddressingMode::Address(_)
            | AddressingMode::AddressWithDisplacement(_)
            | AddressingMode::AddressWithIndex(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::AbsoluteShort
            | AddressingMode::AbsoluteLong => {
                let addr = self.effective_addr(mode);
                self.read_addr(addr)
            }
            AddressingMode::AddressWithPostincrement(register) => {
                let val = self.read_addr(self.addr_register(register));
                self.inc_addr_register::<Size>(register);
                val
            }
            AddressingMode::AddressWithPredecrement(register) => {
                self.dec_addr_register::<Size>(register);
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
                self.set_addr_register(
                    register,
                    val.apply_to_register(self.addr_register(register)),
                );
            }
            AddressingMode::Address(_)
            | AddressingMode::AddressWithDisplacement(_)
            | AddressingMode::AddressWithIndex(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::AbsoluteShort
            | AddressingMode::AbsoluteLong => {
                let addr = self.effective_addr(mode);
                self.write_addr(addr, val);
            }
            AddressingMode::AddressWithPostincrement(register) => {
                let addr = self.addr_register(register);
                self.inc_addr_register::<Size>(register);
                self.write_addr(addr, val);
            }
            AddressingMode::AddressWithPredecrement(register) => {
                self.dec_addr_register::<Size>(register);
                self.write_addr(self.addr_register(register), val);
            }
            AddressingMode::Immediate | AddressingMode::Illegal => panic!(),
        }
    }

    fn read_write<Size: DataSize>(
        &mut self,
        mode: AddressingMode,
        op: &mut dyn FnMut(&mut Self, Size) -> Size,
    ) {
        match mode {
            AddressingMode::DataRegister(register) => {
                let val = Size::from_register_value(self.d[register]);
                let new_val = op(self, val);
                self.d[register] = new_val.apply_to_register(self.d[register])
            }
            AddressingMode::AddressRegister(register) => {
                let val = Size::from_register_value(self.addr_register(register));
                let new_val = op(self, val);
                self.set_addr_register(
                    register,
                    new_val.apply_to_register(self.addr_register(register)),
                );
            }
            AddressingMode::Address(_)
            | AddressingMode::AddressWithDisplacement(_)
            | AddressingMode::AddressWithIndex(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::AbsoluteShort
            | AddressingMode::AbsoluteLong => {
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
                self.inc_addr_register::<Size>(register);
            }
            AddressingMode::AddressWithPredecrement(register) => {
                self.dec_addr_register::<Size>(register);
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
        self.tick(match vector {
            2 | 3 => 50,
            6 => 40,
            4 | 7 | 8 | 32..=47 => 34,
            15..=31 => 44,
            5 => 38,
            _ => 0,
        })
    }

    fn add<Size: DataSize + WrappingAdd>(
        &mut self,
        mode: AddressingMode,
        register: usize,
        operand_direction: OperandDirection,
    ) {
        let operand = Size::from_register_value(self.d[register]);
        match operand_direction {
            OperandDirection::ToRegister => {
                let val: Size = self.read(mode);
                let (carry, result) = match val.checked_add(&operand) {
                    Some(result) => (false, result),
                    None => (true, val.wrapping_add(&operand)),
                };
                let overflow = operand.is_negative() == val.is_negative()
                    && operand.is_negative() != result.is_negative();
                self.set_flag(EXTEND, carry);
                self.set_flag(NEGATIVE, result.is_negative());
                self.set_flag(ZERO, result.is_zero());
                self.set_flag(OVERFLOW, overflow);
                self.set_flag(CARRY, carry);
                self.d[register] = result.apply_to_register(self.d[register]);
            }
            OperandDirection::ToMemory => {
                self.read_write(mode, &mut |cpu, val: Size| {
                    let (carry, result) = match val.checked_add(&operand) {
                        Some(result) => (false, result),
                        None => (true, val.wrapping_add(&operand)),
                    };
                    let overflow = operand.is_negative() == val.is_negative()
                        && operand.is_negative() != result.is_negative();
                    cpu.set_flag(EXTEND, carry);
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    cpu.set_flag(ZERO, result.is_zero());
                    cpu.set_flag(OVERFLOW, overflow);
                    cpu.set_flag(CARRY, carry);
                    result
                });
            }
        }
    }

    fn addi<Size: DataSize + WrappingAdd>(&mut self, mode: AddressingMode) {
        let operand = self.read_extension::<Size>();
        self.read_write(mode, &mut |cpu, val: Size| {
            let (carry, result) = match val.checked_add(&operand) {
                Some(result) => (false, result),
                None => (true, val.wrapping_add(&operand)),
            };
            let overflow = operand.is_negative() == val.is_negative()
                && operand.is_negative() != result.is_negative();
            cpu.set_flag(EXTEND, carry);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, overflow);
            cpu.set_flag(CARRY, carry);
            result
        });
    }

    fn addq<Size: DataSize + WrappingAdd>(&mut self, mode: AddressingMode, data: u8) {
        match mode {
            AddressingMode::AddressRegister(register) => {
                let operand = data as u32;
                let val = self.addr_register(register);
                self.set_addr_register(register, val.wrapping_add(operand));
            }
            _ => {
                let operand = Size::from(data).unwrap();
                self.read_write(mode, &mut |cpu, val: Size| {
                    let (carry, result) = match val.checked_add(&operand) {
                        Some(result) => (false, result),
                        None => (true, val.wrapping_add(&operand)),
                    };
                    let overflow = operand.is_negative() == val.is_negative()
                        && operand.is_negative() != result.is_negative();
                    cpu.set_flag(EXTEND, carry);
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    cpu.set_flag(ZERO, result.is_zero());
                    cpu.set_flag(OVERFLOW, overflow);
                    cpu.set_flag(CARRY, carry);
                    result
                });
            }
        }
    }

    fn addx<Size: DataSize + WrappingAdd + AddAssign<Size>>(
        &mut self,
        operand_mode: OperandMode,
        src_register: usize,
        dest_register: usize,
    ) {
        let addx = &mut |cpu: &mut Cpu, val: Size, operand: Size| {
            let (mut carry, mut result) = match val.checked_add(&operand) {
                Some(result) => (false, result),
                None => (true, val.wrapping_add(&operand)),
            };
            if cpu.flag(EXTEND) {
                if result < Size::max_value() {
                    result += Size::from(1).unwrap();
                } else {
                    carry = true;
                    result = Size::min_value();
                }
            }
            let overflow = operand.is_negative() == val.is_negative()
                && operand.is_negative() != result.is_negative();
            cpu.set_flag(EXTEND, carry);
            cpu.set_flag(NEGATIVE, result.is_negative());
            if !result.is_zero() {
                cpu.set_flag(ZERO, false);
            }
            cpu.set_flag(OVERFLOW, overflow);
            cpu.set_flag(CARRY, carry);
            result
        };
        match operand_mode {
            OperandMode::RegisterToRegister => {
                let operand = Size::from_register_value(self.d[src_register]);
                let val = Size::from_register_value(self.d[dest_register]);
                let result = addx(self, val, operand);
                self.d[dest_register] = result.apply_to_register(self.d[dest_register]);
            }
            OperandMode::MemoryToMemory => {
                let operand = self.read(AddressingMode::AddressWithPredecrement(src_register));
                self.read_write(
                    AddressingMode::AddressWithPredecrement(dest_register),
                    &mut |cpu, val| addx(cpu, val, operand),
                );
            }
        }
    }

    fn and<Size: DataSize>(
        &mut self,
        mode: AddressingMode,
        register: usize,
        operand_direction: OperandDirection,
    ) {
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

    fn asl_memory<Size: DataSize + Shl>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let negative = val.is_negative();
            let result = val << 1;
            cpu.set_flag(EXTEND, negative);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, negative != result.is_negative());
            cpu.set_flag(CARRY, negative);
            result
        });
    }

    fn asl_register<Size: DataSize + Shl<u8, Output=Size>>(
        &mut self,
        register: usize,
        count: u8,
    ) {
        let val = Size::from_register_value(self.d[register]);
        let result = if count > 0 {
            let result = if count >= Size::bits() as u8 {
                Size::from(0).unwrap()
            } else {
                val << count
            };
            let result_with_last_bit = val << (count - 1);
            let carry = result_with_last_bit.is_negative();
            let sigs = val.unsigned_shr(Size::bits() as u32 - count as u32 - 1);
            let sigs_ones = sigs.count_ones();
            self.set_flag(EXTEND, carry);
            self.set_flag(OVERFLOW, sigs_ones > 0 && sigs_ones < (count as u32) + 1);
            self.set_flag(CARRY, carry);
            result
        } else {
            self.set_flag(CARRY, false);
            val
        };
        self.d[register] = result.apply_to_register(self.d[register]);
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(ZERO, result.is_zero());
        self.tick(2 * count);
    }

    fn asr_lsr_memory<Size: DataSize + Shr>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let last_bit = val & Size::from(0b1).unwrap();
            let result = val >> 1;
            cpu.set_flag(EXTEND, !last_bit.is_zero());
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, !last_bit.is_zero());
            result
        });
    }

    fn asr_lsr_register<Size: DataSize + Shr<u8, Output=Size>>(
        &mut self,
        register: usize,
        count: u8,
    ) {
        let val = Size::from_register_value(self.d[register]);
        let result = if count > 0 {
            let result = val >> count;
            let result_with_last_bit = val >> (count - 1);
            let carry = !(result_with_last_bit & Size::from(0b1).unwrap()).is_zero();
            self.set_flag(EXTEND, carry);
            self.set_flag(CARRY, carry);
            result
        } else {
            self.set_flag(CARRY, false);
            val
        };
        self.d[register] = result.apply_to_register(self.d[register]);
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(ZERO, result.is_zero());
        self.set_flag(OVERFLOW, false);
        self.tick(2 * count);
    }

    fn branch(&mut self, displacement: i8, do_branch: bool) {
        let pc = self.pc;
        let word_displacement = displacement == 0;
        let displacement = if word_displacement {
            self.read_extension::<i16>() as i32
        } else {
            displacement as i32
        };
        if do_branch {
            self.pc = pc.wrapping_add_signed(displacement);
            self.tick(2);
        } else if word_displacement {
            self.tick(4);
        }
    }

    fn do_cmp<Size: DataSize + WrappingSub>(&mut self, operand: Size, val: Size) {
        let (carry, result) = match val.checked_sub(&operand) {
            Some(result) => (false, result),
            None => (true, val.wrapping_sub(&operand)),
        };
        let overflow = operand.is_negative() == result.is_negative()
            && operand.is_negative() != val.is_negative();
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(ZERO, result.is_zero());
        self.set_flag(OVERFLOW, overflow);
        self.set_flag(CARRY, carry);
    }

    fn cmp<Size: DataSize + WrappingSub>(&mut self, mode: AddressingMode, register: usize) {
        let operand = self.read::<Size>(mode);
        let val = Size::from_register_value(self.d[register]);
        self.do_cmp(operand, val);
    }

    fn cmpa<Size: DataSize + WrappingSub>(&mut self, mode: AddressingMode, register: usize) {
        let operand = Size::to_register_value(self.read::<Size>(mode));
        let val = self.addr_register(register);
        self.do_cmp(operand, val);
    }

    fn cmpi<Size: DataSize + WrappingSub>(&mut self, mode: AddressingMode) {
        let operand = self.read_extension::<Size>();
        let val = self.read::<Size>(mode);
        self.do_cmp(operand, val);
    }

    fn cmpm<Size: DataSize + WrappingSub>(&mut self, src_register: usize, dest_register: usize) {
        let operand = self.read::<Size>(AddressingMode::AddressWithPostincrement(src_register));
        let val = self.read::<Size>(AddressingMode::AddressWithPostincrement(dest_register));
        self.do_cmp(operand, val);
    }

    fn eor<Size: DataSize>(
        &mut self,
        mode: AddressingMode,
        register: usize,
        operand_direction: OperandDirection,
    ) {
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

    fn lsl_memory<Size: DataSize + Shl>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let negative = val.is_negative();
            let result = val << 1;
            cpu.set_flag(EXTEND, negative);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, negative);
            result
        });
    }

    fn lsl_register<Size: DataSize + Shl<u8, Output=Size>>(
        &mut self,
        register: usize,
        count: u8,
    ) {
        let val = Size::from_register_value(self.d[register]);
        let result = if count > 0 {
            let result = if count >= Size::bits() as u8 {
                Size::from(0).unwrap()
            } else {
                val << count
            };
            let result_with_last_bit = val << (count - 1);
            let carry = result_with_last_bit.is_negative();
            self.set_flag(EXTEND, carry);
            self.set_flag(CARRY, carry);
            result
        } else {
            self.set_flag(CARRY, false);
            val
        };
        self.d[register] = result.apply_to_register(self.d[register]);
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(OVERFLOW, false);
        self.set_flag(ZERO, result.is_zero());
        self.tick(2 * count);
    }

    fn move_<Size: DataSize>(&mut self, src_mode: AddressingMode, dest_mode: AddressingMode) {
        let val: Size = self.read(src_mode);
        self.set_flag(NEGATIVE, val.is_negative());
        self.set_flag(ZERO, val.is_zero());
        self.set_flag(OVERFLOW, false);
        self.set_flag(CARRY, false);
        self.write(dest_mode, val);
    }

    fn movem<Size: DataSize>(
        &mut self,
        mode: AddressingMode,
        direction: Direction,
        cycle_count_per_move: u8,
    ) {
        let register_list_mask = self.read_extension::<u16>();
        let mask_reversed = if let AddressingMode::AddressWithPredecrement(_) = mode {
            true
        } else {
            false
        };
        let mut stored_addr_register_val = 0;
        let mut addr = match mode {
            AddressingMode::AddressWithPredecrement(register) => {
                stored_addr_register_val = self.addr_register(register);
                self.addr_register(register)
            }
            AddressingMode::AddressWithPostincrement(register) => self.addr_register(register),
            _ => self.effective_addr(mode),
        };
        for i in 0usize..16 {
            if (register_list_mask >> i) & 0b1 == 0b1 {
                match mode {
                    AddressingMode::AddressWithPredecrement(register) => {
                        self.dec_addr_register::<Size>(register);
                        addr = self.addr_register(register);
                    }
                    AddressingMode::AddressWithPostincrement(register) => {
                        addr = self.addr_register(register);
                        self.inc_addr_register::<Size>(register);
                    }
                    _ => {}
                }
                match direction {
                    Direction::RegisterToMemory => {
                        let val = Size::from_register_value(if mask_reversed {
                            if i < 8 {
                                match mode {
                                    AddressingMode::AddressWithPredecrement(n) if 7 - n == i => {
                                        stored_addr_register_val
                                    }
                                    _ => self.addr_register(7 - i),
                                }
                            } else {
                                self.d[15 - i]
                            }
                        } else {
                            if i < 8 {
                                self.d[i]
                            } else {
                                self.addr_register(i - 8)
                            }
                        });
                        self.write_addr(addr, val)
                    }
                    Direction::MemoryToRegister => {
                        let val = self.read_addr::<Size>(addr).to_register_value();
                        if i < 8 {
                            self.d[i] = val;
                        } else {
                            match mode {
                                AddressingMode::AddressWithPostincrement(n) if n == i - 8 => {}
                                _ => self.set_addr_register(i - 8, val),
                            }
                        }
                    }
                }
                addr = match mode {
                    AddressingMode::AddressWithPredecrement(_)
                    | AddressingMode::AddressWithPostincrement(_) => addr,
                    _ => addr + Size::address_size(),
                };
                self.tick(cycle_count_per_move);
            }
        }
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
            let mut overflow = false;
            let result = if val == Size::min_value() && !cpu.flag(EXTEND) {
                overflow = true;
                val
            } else if val == Size::max_value() && cpu.flag(EXTEND) {
                Size::min_value()
            } else if val == Size::min_value() && cpu.flag(EXTEND) {
                Size::max_value()
            } else if cpu.flag(EXTEND) {
                -val - Size::from(1).unwrap()
            } else {
                -val
            };
            cpu.set_flag(EXTEND, !result.is_zero());
            cpu.set_flag(NEGATIVE, result.is_negative());
            if !result.is_zero() {
                cpu.set_flag(ZERO, false);
            }
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

    fn or<Size: DataSize>(
        &mut self,
        mode: AddressingMode,
        register: usize,
        operand_direction: OperandDirection,
    ) {
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

    fn rol_memory<Size: DataSize>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let negative = val.is_negative();
            let result = val.rotate_left(1);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, negative);
            result
        });
    }

    fn rol_register<Size: DataSize>(&mut self, register: usize, count: u8) {
        let val = Size::from_register_value(self.d[register]);
        let result = if count > 0 {
            let result = val.rotate_left(count as u32);
            let result_with_last_bit = val.rotate_left((count - 1) as u32);
            let carry = result_with_last_bit.is_negative();
            self.set_flag(CARRY, carry);
            result
        } else {
            self.set_flag(CARRY, false);
            val
        };
        self.d[register] = result.apply_to_register(self.d[register]);
        self.set_flag(OVERFLOW, false);
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(ZERO, result.is_zero());
        self.tick(2 * count);
    }

    fn ror_memory<Size: DataSize>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let last_bit = val & Size::from(0b1).unwrap();
            let result = val.rotate_right(1);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, !last_bit.is_zero());
            result
        });
    }

    fn ror_register<Size: DataSize>(&mut self, register: usize, count: u8) {
        let val = Size::from_register_value(self.d[register]);
        let result = if count > 0 {
            let result = val.rotate_right(count as u32);
            let last_bit = val.rotate_right((count - 1) as u32) & Size::from(0b1).unwrap();
            self.set_flag(CARRY, !last_bit.is_zero());
            result
        } else {
            self.set_flag(CARRY, false);
            val
        };
        self.d[register] = result.apply_to_register(self.d[register]);
        self.set_flag(OVERFLOW, false);
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(ZERO, result.is_zero());
        self.tick(2 * count);
    }

    fn roxl_memory<Size: DataSize>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let first_bit = val.is_negative();
            let result = (val << 1)
                | if cpu.flag(EXTEND) {
                Size::from(0b1).unwrap()
            } else {
                Size::from(0b0).unwrap()
            };
            cpu.set_flag(EXTEND, first_bit);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, first_bit);
            result
        });
    }

    fn roxl_register<Size: DataSize>(&mut self, register: usize, count: u8) {
        let val = Size::from_register_value(self.d[register]);
        let result = if count > 0 {
            let mut result = val;
            for _ in 0..count {
                let first_bit = result.is_negative();
                result = (result << 1)
                    | if self.flag(EXTEND) {
                    Size::from(0b1).unwrap()
                } else {
                    Size::from(0b0).unwrap()
                };
                self.set_flag(EXTEND, first_bit);
                self.set_flag(CARRY, first_bit);
            }
            result
        } else {
            self.set_flag(CARRY, self.flag(EXTEND));
            val
        };
        self.d[register] = result.apply_to_register(self.d[register]);
        self.set_flag(OVERFLOW, false);
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(ZERO, result.is_zero());
        self.tick(2 * count);
    }

    fn roxr_memory<Size: DataSize>(&mut self, mode: AddressingMode) {
        self.read_write::<Size>(mode, &mut |cpu, val| {
            let last_bit = !(val & Size::from(0b1).unwrap()).is_zero();
            let result = (val >> 1)
                | if cpu.flag(EXTEND) {
                Size::from(Size::from(0b1).unwrap() << (Size::bits() - 1)).unwrap()
            } else {
                Size::from(0b0).unwrap()
            };
            cpu.set_flag(EXTEND, last_bit);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, false);
            cpu.set_flag(CARRY, last_bit);
            result
        });
    }

    fn roxr_register<Size: DataSize>(&mut self, register: usize, count: u8) {
        let val = Size::from_register_value(self.d[register]);
        let result = if count > 0 {
            let mut result = val;
            for _ in 0..count {
                let last_bit = !(result & Size::from(0b1).unwrap()).is_zero();
                result = (result >> 1)
                    | if self.flag(EXTEND) {
                    Size::from(Size::from(0b1).unwrap() << (Size::bits() - 1)).unwrap()
                } else {
                    Size::from(0b0).unwrap()
                };
                self.set_flag(EXTEND, last_bit);
                self.set_flag(CARRY, last_bit);
            }
            result
        } else {
            self.set_flag(CARRY, self.flag(EXTEND));
            val
        };
        self.d[register] = result.apply_to_register(self.d[register]);
        self.set_flag(OVERFLOW, false);
        self.set_flag(NEGATIVE, result.is_negative());
        self.set_flag(ZERO, result.is_zero());
        self.tick(2 * count);
    }

    fn sub<Size: DataSize + WrappingSub>(
        &mut self,
        mode: AddressingMode,
        register: usize,
        operand_direction: OperandDirection,
    ) {
        match operand_direction {
            OperandDirection::ToRegister => {
                let operand: Size = self.read(mode);
                let val = Size::from_register_value(self.d[register]);
                let (carry, result) = match val.checked_sub(&operand) {
                    Some(result) => (false, result),
                    None => (true, val.wrapping_sub(&operand)),
                };
                let overflow = operand.is_negative() == result.is_negative()
                    && operand.is_negative() != val.is_negative();
                self.set_flag(EXTEND, carry);
                self.set_flag(NEGATIVE, result.is_negative());
                self.set_flag(ZERO, result.is_zero());
                self.set_flag(OVERFLOW, overflow);
                self.set_flag(CARRY, carry);
                self.d[register] = result.apply_to_register(self.d[register]);
            }
            OperandDirection::ToMemory => {
                let operand = Size::from_register_value(self.d[register]);
                self.read_write(mode, &mut |cpu, val: Size| {
                    let (carry, result) = match val.checked_sub(&operand) {
                        Some(result) => (false, result),
                        None => (true, val.wrapping_sub(&operand)),
                    };
                    let overflow = operand.is_negative() == result.is_negative()
                        && operand.is_negative() != val.is_negative();
                    cpu.set_flag(EXTEND, carry);
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    cpu.set_flag(ZERO, result.is_zero());
                    cpu.set_flag(OVERFLOW, overflow);
                    cpu.set_flag(CARRY, carry);
                    result
                });
            }
        }
    }

    fn subi<Size: DataSize + WrappingSub>(&mut self, mode: AddressingMode) {
        let operand = self.read_extension::<Size>();
        self.read_write(mode, &mut |cpu, val: Size| {
            let (carry, result) = match val.checked_sub(&operand) {
                Some(result) => (false, result),
                None => (true, val.wrapping_sub(&operand)),
            };
            let overflow = operand.is_negative() == result.is_negative()
                && operand.is_negative() != val.is_negative();
            cpu.set_flag(EXTEND, carry);
            cpu.set_flag(NEGATIVE, result.is_negative());
            cpu.set_flag(ZERO, result.is_zero());
            cpu.set_flag(OVERFLOW, overflow);
            cpu.set_flag(CARRY, carry);
            result
        });
    }

    fn subq<Size: DataSize + WrappingSub>(&mut self, mode: AddressingMode, data: u8) {
        match mode {
            AddressingMode::AddressRegister(register) => {
                let operand = data as u32;
                let val = self.addr_register(register);
                self.set_addr_register(register, val.wrapping_sub(operand));
            }
            _ => {
                let operand = Size::from(data).unwrap();
                self.read_write(mode, &mut |cpu, val: Size| {
                    let (carry, result) = match val.checked_sub(&operand) {
                        Some(result) => (false, result),
                        None => (true, val.wrapping_sub(&operand)),
                    };
                    let overflow = operand.is_negative() == result.is_negative()
                        && operand.is_negative() != val.is_negative();
                    cpu.set_flag(EXTEND, carry);
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    cpu.set_flag(ZERO, result.is_zero());
                    cpu.set_flag(OVERFLOW, overflow);
                    cpu.set_flag(CARRY, carry);
                    result
                });
            }
        }
    }

    fn subx<Size: DataSize + WrappingSub + SubAssign<Size>>(
        &mut self,
        operand_mode: OperandMode,
        src_register: usize,
        dest_register: usize,
    ) {
        let subx = &mut |cpu: &mut Cpu, val: Size, operand: Size| {
            let (mut carry, mut result) = match val.checked_sub(&operand) {
                Some(result) => (false, result),
                None => (true, val.wrapping_sub(&operand)),
            };
            if cpu.flag(EXTEND) {
                if result > Size::min_value() {
                    result -= Size::from(1).unwrap();
                } else {
                    carry = true;
                    result = Size::max_value();
                }
            }
            let overflow = operand.is_negative() == result.is_negative()
                && operand.is_negative() != val.is_negative();
            cpu.set_flag(EXTEND, carry);
            cpu.set_flag(NEGATIVE, result.is_negative());
            if !result.is_zero() {
                cpu.set_flag(ZERO, false);
            }
            cpu.set_flag(OVERFLOW, overflow);
            cpu.set_flag(CARRY, carry);
            result
        };
        match operand_mode {
            OperandMode::RegisterToRegister => {
                let operand = Size::from_register_value(self.d[src_register]);
                let val = Size::from_register_value(self.d[dest_register]);
                let result = subx(self, val, operand);
                self.d[dest_register] = result.apply_to_register(self.d[dest_register]);
            }
            OperandMode::MemoryToMemory => {
                let operand = self.read(AddressingMode::AddressWithPredecrement(src_register));
                self.read_write(
                    AddressingMode::AddressWithPredecrement(dest_register),
                    &mut |cpu, val| subx(cpu, val, operand),
                );
            }
        }
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

        if self.instrumented {
            debug!(target: "cpu", "{:08X}\t{:04X}\t{}",
            opcode_pc,
            opcode_hex,
            opcode);
        }

        match opcode {
            Opcode::ABCD {
                operand_mode,
                src_register,
                dest_register,
            } => {
                let abcd = |cpu: &mut Cpu, val: u8, operand: u8| {
                    let binary_result = val
                        .wrapping_add(operand)
                        .wrapping_add(if cpu.flag(EXTEND) { 1 } else { 0 });
                    let binary_carry =
                        ((val & operand) | (val & !binary_result) | (operand & !binary_result))
                            & 0x88;
                    let decimal_carry =
                        (((((binary_result as u16) + 0x66) ^ (binary_result as u16)) & 0x110) >> 1)
                            as u8;
                    let correction_factor =
                        (binary_carry | decimal_carry) - ((binary_carry | decimal_carry) >> 2);
                    let result = binary_result.wrapping_add(correction_factor);
                    let carry = binary_carry.is_negative()
                        || (binary_result.is_negative() && !result.is_negative());
                    cpu.set_flag(EXTEND, carry);
                    cpu.set_flag(CARRY, carry);
                    cpu.set_flag(
                        OVERFLOW,
                        !binary_result.is_negative() && result.is_negative(),
                    );
                    if result != 0 {
                        cpu.set_flag(ZERO, false);
                    }
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    result
                };
                match operand_mode {
                    OperandMode::RegisterToRegister => {
                        let val = u8::from_register_value(self.d[dest_register]);
                        let operand = u8::from_register_value(self.d[src_register]);
                        let result = abcd(self, val, operand);
                        self.d[dest_register] = result.apply_to_register(self.d[dest_register]);
                    }
                    OperandMode::MemoryToMemory => {
                        let operand =
                            self.read::<u8>(AddressingMode::AddressWithPredecrement(src_register));
                        self.read_write::<u8>(
                            AddressingMode::AddressWithPredecrement(dest_register),
                            &mut |cpu, val| abcd(cpu, val, operand),
                        );
                    }
                }
            }
            Opcode::ADD {
                mode,
                size,
                operand_direction,
                register,
            } => match size {
                Size::Byte => self.add::<u8>(mode, register, operand_direction),
                Size::Word => self.add::<u16>(mode, register, operand_direction),
                Size::Long => self.add::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!(),
            },
            Opcode::ADDA {
                mode,
                size,
                register,
            } => {
                let operand = match size {
                    Size::Word => self.read::<i16>(mode) as i32,
                    Size::Long => self.read(mode),
                    Size::Byte | Size::Illegal => panic!(),
                };
                let result = self.addr_register(register).wrapping_add_signed(operand);
                self.set_addr_register(register, result);
            }
            Opcode::ADDI { mode, size } => match size {
                Size::Byte => self.addi::<u8>(mode),
                Size::Word => self.addi::<u16>(mode),
                Size::Long => self.addi::<u32>(mode),
                Size::Illegal => panic!(),
            },
            Opcode::ADDQ { mode, size, data } => match size {
                Size::Byte => self.addq::<u8>(mode, data),
                Size::Word => self.addq::<u16>(mode, data),
                Size::Long => self.addq::<u32>(mode, data),
                Size::Illegal => panic!(),
            },
            Opcode::ADDX {
                operand_mode,
                size,
                src_register,
                dest_register,
            } => match size {
                Size::Byte => self.addx::<u8>(operand_mode, src_register, dest_register),
                Size::Word => self.addx::<u16>(operand_mode, src_register, dest_register),
                Size::Long => self.addx::<u32>(operand_mode, src_register, dest_register),
                Size::Illegal => panic!(),
            },
            Opcode::AND {
                mode,
                size,
                operand_direction,
                register,
            } => match size {
                Size::Byte => self.and::<u8>(mode, register, operand_direction),
                Size::Word => self.and::<u16>(mode, register, operand_direction),
                Size::Long => self.and::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!(),
            },
            Opcode::ANDI { mode, size } => match size {
                Size::Byte => self.andi::<u8>(mode),
                Size::Word => self.andi::<u16>(mode),
                Size::Long => self.andi::<u32>(mode),
                Size::Illegal => panic!(),
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
            Opcode::ASL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.asl_register::<i8>(register, count),
                            Size::Word => self.asl_register::<i16>(register, count),
                            Size::Long => self.asl_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.asl_register::<i8>(register, count),
                            Size::Word => self.asl_register::<i16>(register, count),
                            Size::Long => self.asl_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.asl_memory::<i8>(mode),
                        Size::Word => self.asl_memory::<i16>(mode),
                        Size::Long => self.asl_memory::<i32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
            }
            Opcode::ASR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.asr_lsr_register::<i8>(register, count),
                            Size::Word => self.asr_lsr_register::<i16>(register, count),
                            Size::Long => self.asr_lsr_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.asr_lsr_register::<i8>(register, count),
                            Size::Word => self.asr_lsr_register::<i16>(register, count),
                            Size::Long => self.asr_lsr_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.asr_lsr_memory::<i8>(mode),
                        Size::Word => self.asr_lsr_memory::<i16>(mode),
                        Size::Long => self.asr_lsr_memory::<i32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
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
                    BitNum::DataRegister(register) => self.d[register],
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
                        AddressingMode::DataRegister(_) => {
                            self.read_write::<u32>(mode, &mut |cpu, val| {
                                let bit_val = (val >> bit) & 0b1;
                                cpu.set_flag(ZERO, bit_val == 0);
                                match opcode {
                                    Opcode::BCHG { .. } => val ^ (1 << bit),
                                    Opcode::BCLR { .. } => val & !(1 << bit),
                                    Opcode::BSET { .. } => val | (1 << bit),
                                    _ => panic!(),
                                }
                            })
                        }
                        _ => self.read_write::<u8>(mode, &mut |cpu, val| {
                            let bit_val = (val >> bit) & 0b1;
                            cpu.set_flag(ZERO, bit_val == 0);
                            match opcode {
                                Opcode::BCHG { .. } => val ^ (1 << bit),
                                Opcode::BCLR { .. } => val & !(1 << bit),
                                Opcode::BSET { .. } => val | (1 << bit),
                                _ => panic!(),
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
            Opcode::Bcc {
                displacement,
                condition,
            } => {
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
            Opcode::CMP {
                mode,
                size,
                register,
            } => match size {
                Size::Byte => self.cmp::<u8>(mode, register),
                Size::Word => self.cmp::<u16>(mode, register),
                Size::Long => self.cmp::<u32>(mode, register),
                Size::Illegal => panic!(),
            },
            Opcode::CMPA {
                mode,
                size,
                register,
            } => match size {
                Size::Byte => self.cmpa::<u8>(mode, register),
                Size::Word => self.cmpa::<u16>(mode, register),
                Size::Long => self.cmpa::<u32>(mode, register),
                Size::Illegal => panic!(),
            },
            Opcode::CMPI { mode, size } => match size {
                Size::Byte => self.cmpi::<u8>(mode),
                Size::Word => self.cmpi::<u16>(mode),
                Size::Long => self.cmpi::<u32>(mode),
                Size::Illegal => panic!(),
            },
            Opcode::CMPM {
                size,
                src_register,
                dest_register,
            } => match size {
                Size::Byte => self.cmpm::<u8>(src_register, dest_register),
                Size::Word => self.cmpm::<u16>(src_register, dest_register),
                Size::Long => self.cmpm::<u32>(src_register, dest_register),
                Size::Illegal => panic!(),
            },
            Opcode::DBcc {
                condition,
                register,
            } => {
                let displacement = self.read_extension::<i16>();
                if !self.check_condition(condition) {
                    let dec_value = ((self.d[register] & 0xFFFF) as i16).saturating_sub(1);
                    self.d[register] = dec_value.apply_to_register(self.d[register]);
                    if dec_value != -1 {
                        self.pc = self.pc.wrapping_add_signed((displacement - 2) as i32);
                    } else {
                        self.tick(4);
                    }
                } else {
                    self.tick(2);
                }
            }
            Opcode::DIVS { mode, register } => {
                let val = self.d[register] as i32;
                let operand = self.read::<i16>(mode) as i32;
                self.set_flag(CARRY, false);
                if operand == 0 {
                    self.process_exception(5);
                } else {
                    let (quotient, remainder) = val.div_rem(&operand);
                    let remainder = if val < 0 { -remainder } else { remainder };
                    if quotient > i16::MAX as i32 || quotient < i16::MIN as i32 {
                        self.set_flag(OVERFLOW, true);
                        self.set_flag(NEGATIVE, true);
                    } else {
                        self.set_flag(OVERFLOW, false);
                        self.set_flag(NEGATIVE, (quotient as i16).is_negative());
                        self.set_flag(ZERO, quotient.is_zero());
                        self.d[register] =
                            ((remainder as u32) << 16) | ((quotient as u32) & 0xFFFF);
                    }
                }
            }
            Opcode::DIVU { mode, register } => {
                let val = self.d[register];
                let operand = self.read::<u16>(mode) as u32;
                self.set_flag(CARRY, false);
                if operand == 0 {
                    self.process_exception(5);
                } else {
                    let (quotient, remainder) = val.div_rem(&operand);
                    if quotient > u16::MAX as u32 {
                        self.set_flag(OVERFLOW, true);
                        self.set_flag(NEGATIVE, true);
                    } else {
                        self.set_flag(OVERFLOW, false);
                        self.set_flag(NEGATIVE, (quotient as u16).is_negative());
                        self.set_flag(ZERO, quotient.is_zero());
                        self.d[register] = (remainder << 16) | quotient;
                    }
                }
            }
            Opcode::EOR {
                mode,
                size,
                operand_direction,
                register,
            } => match size {
                Size::Byte => self.eor::<u8>(mode, register, operand_direction),
                Size::Word => self.eor::<u16>(mode, register, operand_direction),
                Size::Long => self.eor::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!(),
            },
            Opcode::EORI { mode, size } => match size {
                Size::Byte => self.eori::<u8>(mode),
                Size::Word => self.eori::<u16>(mode),
                Size::Long => self.eori::<u32>(mode),
                Size::Illegal => panic!(),
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
            Opcode::EXG {
                mode,
                src_register,
                dest_register,
            } => {
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
                    ExchangeMode::Illegal => panic!(),
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
                self.push(self.pc);
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
                self.set_addr_register(
                    7,
                    self.addr_register(7)
                        .wrapping_add_signed(displacement as i32),
                );
            }
            Opcode::LSL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.lsl_register::<u8>(register, count),
                            Size::Word => self.lsl_register::<u16>(register, count),
                            Size::Long => self.lsl_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.lsl_register::<u8>(register, count),
                            Size::Word => self.lsl_register::<u16>(register, count),
                            Size::Long => self.lsl_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.lsl_memory::<u8>(mode),
                        Size::Word => self.lsl_memory::<u16>(mode),
                        Size::Long => self.lsl_memory::<u32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
            }
            Opcode::LSR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.asr_lsr_register::<u8>(register, count),
                            Size::Word => self.asr_lsr_register::<u16>(register, count),
                            Size::Long => self.asr_lsr_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.asr_lsr_register::<u8>(register, count),
                            Size::Word => self.asr_lsr_register::<u16>(register, count),
                            Size::Long => self.asr_lsr_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.asr_lsr_memory::<u8>(mode),
                        Size::Word => self.asr_lsr_memory::<u16>(mode),
                        Size::Long => self.asr_lsr_memory::<u32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
            }
            Opcode::MOVE {
                src_mode,
                dest_mode,
                size,
            } => match size {
                Size::Byte => self.move_::<i8>(src_mode, dest_mode),
                Size::Word => self.move_::<i16>(src_mode, dest_mode),
                Size::Long => self.move_::<i32>(src_mode, dest_mode),
                Size::Illegal => panic!(),
            },
            Opcode::MOVEA {
                src_mode,
                dest_mode,
                size,
            } => match size {
                Size::Word => self.move_::<i16>(src_mode, dest_mode),
                Size::Long => self.move_::<i32>(src_mode, dest_mode),
                Size::Byte | Size::Illegal => panic!(),
            },
            Opcode::MOVEM {
                mode,
                size,
                direction,
            } => match size {
                Size::Word => self.movem::<u16>(mode, direction, 4),
                Size::Long => self.movem::<u32>(mode, direction, 8),
                Size::Byte | Size::Illegal => panic!(),
            },
            Opcode::MOVEP {
                data_register,
                address_register,
                direction,
                size,
            } => {
                let addr =
                    self.effective_addr(AddressingMode::AddressWithDisplacement(address_register));
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
                            _ => panic!(),
                        }
                    }
                    Direction::MemoryToRegister => match size {
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
                        _ => panic!(),
                    },
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
            Opcode::MOVE_USP {
                register,
                direction,
            } => match direction {
                Direction::MemoryToRegister => self.set_addr_register(register, self.a[7]),
                Direction::RegisterToMemory => self.a[7] = self.addr_register(register),
            },
            Opcode::MULS { mode, register } => {
                let val = self.read::<i16>(mode) as i32;
                let operand = i16::from_register_value(self.d[register]) as i32;
                let result = val * operand;
                self.d[register] = result as u32;
                self.set_flag(NEGATIVE, result.is_negative());
                self.set_flag(ZERO, result.is_zero());
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
            }
            Opcode::MULU { mode, register } => {
                let val = self.read::<u16>(mode) as u32;
                let operand = u16::from_register_value(self.d[register]) as u32;
                let result = val * operand;
                self.d[register] = result;
                self.set_flag(NEGATIVE, result.is_negative());
                self.set_flag(ZERO, result.is_zero());
                self.set_flag(OVERFLOW, false);
                self.set_flag(CARRY, false);
            }
            Opcode::NBCD { mode } => {
                self.read_write::<u8>(mode, &mut |cpu, val| {
                    let binary_result =
                        0u8.wrapping_sub(val)
                            .wrapping_sub(if cpu.flag(EXTEND) { 1 } else { 0 });
                    let binary_carry = (val | binary_result) & 0x88;
                    let correction_factor = binary_carry - (binary_carry >> 2);
                    let result = binary_result.wrapping_sub(correction_factor);
                    let carry = binary_carry.is_negative()
                        || (!binary_result.is_negative() && result.is_negative());
                    cpu.set_flag(EXTEND, carry);
                    cpu.set_flag(CARRY, carry);
                    cpu.set_flag(
                        OVERFLOW,
                        binary_result.is_negative() && !result.is_negative(),
                    );
                    if result != 0 {
                        cpu.set_flag(ZERO, false);
                    }
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    result
                });
            }
            Opcode::NEG { mode, size } => match size {
                Size::Byte => self.neg::<i8>(mode),
                Size::Word => self.neg::<i16>(mode),
                Size::Long => self.neg::<i32>(mode),
                Size::Illegal => panic!(),
            },
            Opcode::NEGX { mode, size } => match size {
                Size::Byte => self.negx::<i8>(mode),
                Size::Word => self.negx::<i16>(mode),
                Size::Long => self.negx::<i32>(mode),
                Size::Illegal => panic!(),
            },
            Opcode::NOP => {}
            Opcode::NOT { mode, size } => match size {
                Size::Byte => self.not::<u8>(mode),
                Size::Word => self.not::<u16>(mode),
                Size::Long => self.not::<u32>(mode),
                Size::Illegal => panic!(),
            },
            Opcode::OR {
                mode,
                size,
                operand_direction,
                register,
            } => match size {
                Size::Byte => self.or::<u8>(mode, register, operand_direction),
                Size::Word => self.or::<u16>(mode, register, operand_direction),
                Size::Long => self.or::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!(),
            },
            Opcode::ORI { mode, size } => match size {
                Size::Byte => self.ori::<u8>(mode),
                Size::Word => self.ori::<u16>(mode),
                Size::Long => self.ori::<u32>(mode),
                Size::Illegal => panic!(),
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
            Opcode::PEA { mode } => {
                let val = self.effective_addr(mode);
                self.push(val);
            }
            Opcode::RESET => {}
            Opcode::ROL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.rol_register::<i8>(register, count),
                            Size::Word => self.rol_register::<i16>(register, count),
                            Size::Long => self.rol_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.rol_register::<i8>(register, count),
                            Size::Word => self.rol_register::<i16>(register, count),
                            Size::Long => self.rol_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.rol_memory::<i8>(mode),
                        Size::Word => self.rol_memory::<i16>(mode),
                        Size::Long => self.rol_memory::<i32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
            }
            Opcode::ROR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.ror_register::<i8>(register, count),
                            Size::Word => self.ror_register::<i16>(register, count),
                            Size::Long => self.ror_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.ror_register::<i8>(register, count),
                            Size::Word => self.ror_register::<i16>(register, count),
                            Size::Long => self.ror_register::<i32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.ror_memory::<i8>(mode),
                        Size::Word => self.ror_memory::<i16>(mode),
                        Size::Long => self.ror_memory::<i32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
            }
            Opcode::ROXL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.roxl_register::<u8>(register, count),
                            Size::Word => self.roxl_register::<u16>(register, count),
                            Size::Long => self.roxl_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.roxl_register::<u8>(register, count),
                            Size::Word => self.roxl_register::<u16>(register, count),
                            Size::Long => self.roxl_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.roxl_memory::<u8>(mode),
                        Size::Word => self.roxl_memory::<u16>(mode),
                        Size::Long => self.roxl_memory::<u32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
            }
            Opcode::ROXR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => {
                if let Some(register) = register {
                    if let Some(count) = shift_count {
                        match size {
                            Size::Byte => self.roxr_register::<u8>(register, count),
                            Size::Word => self.roxr_register::<u16>(register, count),
                            Size::Long => self.roxr_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else if let Some(shift_register) = shift_register {
                        let count = (self.d[shift_register] % 64) as u8;
                        match size {
                            Size::Byte => self.roxr_register::<u8>(register, count),
                            Size::Word => self.roxr_register::<u16>(register, count),
                            Size::Long => self.roxr_register::<u32>(register, count),
                            Size::Illegal => panic!(),
                        }
                    } else {
                        panic!()
                    }
                } else {
                    match size {
                        Size::Byte => self.roxr_memory::<u8>(mode),
                        Size::Word => self.roxr_memory::<u16>(mode),
                        Size::Long => self.roxr_memory::<u32>(mode),
                        Size::Illegal => panic!(),
                    }
                }
            }
            Opcode::RTE => {
                let new_status = self.pop();
                self.set_status(new_status);
                self.pc = self.pop();
            }
            Opcode::RTR => {
                let new_status = (self.status & 0xFF00) | (self.pop::<u16>() & 0xFF);
                self.set_status(new_status);
                self.pc = self.pop();
            }
            Opcode::RTS => {
                self.pc = self.pop();
            }
            Opcode::SBCD {
                operand_mode,
                src_register,
                dest_register,
            } => {
                let abcd = |cpu: &mut Cpu, val: u8, operand: u8| {
                    let binary_result = val
                        .wrapping_sub(operand)
                        .wrapping_sub(if cpu.flag(EXTEND) { 1 } else { 0 });
                    let binary_carry =
                        ((!val & operand) | (!val & binary_result) | (operand & binary_result))
                            & 0x88;
                    let correction_factor = binary_carry - (binary_carry >> 2);
                    let result = binary_result.wrapping_sub(correction_factor);
                    let carry = binary_carry.is_negative()
                        || (!binary_result.is_negative() && result.is_negative());
                    cpu.set_flag(EXTEND, carry);
                    cpu.set_flag(CARRY, carry);
                    cpu.set_flag(
                        OVERFLOW,
                        binary_result.is_negative() && !result.is_negative(),
                    );
                    if result != 0 {
                        cpu.set_flag(ZERO, false);
                    }
                    cpu.set_flag(NEGATIVE, result.is_negative());
                    result
                };
                match operand_mode {
                    OperandMode::RegisterToRegister => {
                        let val = u8::from_register_value(self.d[dest_register]);
                        let operand = u8::from_register_value(self.d[src_register]);
                        let result = abcd(self, val, operand);
                        self.d[dest_register] = result.apply_to_register(self.d[dest_register]);
                    }
                    OperandMode::MemoryToMemory => {
                        let operand =
                            self.read::<u8>(AddressingMode::AddressWithPredecrement(src_register));
                        self.read_write::<u8>(
                            AddressingMode::AddressWithPredecrement(dest_register),
                            &mut |cpu, val| abcd(cpu, val, operand),
                        );
                    }
                }
            }
            Opcode::Scc { mode, condition } => {
                if self.check_condition(condition) {
                    self.write::<u8>(mode, 0xFF);
                    self.tick(2);
                } else {
                    self.write::<u8>(mode, 0x00);
                }
            }
            Opcode::STOP => {
                let new_status = self.read_extension();
                self.set_status(new_status);
                self.stopped = true;
            }
            Opcode::SUB {
                mode,
                size,
                operand_direction,
                register,
            } => match size {
                Size::Byte => self.sub::<u8>(mode, register, operand_direction),
                Size::Word => self.sub::<u16>(mode, register, operand_direction),
                Size::Long => self.sub::<u32>(mode, register, operand_direction),
                Size::Illegal => panic!(),
            },
            Opcode::SUBA {
                mode,
                size,
                register,
            } => {
                let operand = match size {
                    Size::Word => self.read::<i16>(mode) as i32,
                    Size::Long => self.read(mode),
                    Size::Byte | Size::Illegal => panic!(),
                };
                let result = self.addr_register(register).wrapping_add_signed(-operand);
                self.set_addr_register(register, result);
            }
            Opcode::SUBI { mode, size } => match size {
                Size::Byte => self.subi::<u8>(mode),
                Size::Word => self.subi::<u16>(mode),
                Size::Long => self.subi::<u32>(mode),
                Size::Illegal => panic!(),
            },
            Opcode::SUBQ { mode, size, data } => match size {
                Size::Byte => self.subq::<u8>(mode, data),
                Size::Word => self.subq::<u16>(mode, data),
                Size::Long => self.subq::<u32>(mode, data),
                Size::Illegal => panic!(),
            },
            Opcode::SUBX {
                operand_mode,
                size,
                src_register,
                dest_register,
            } => match size {
                Size::Byte => self.subx::<u8>(operand_mode, src_register, dest_register),
                Size::Word => self.subx::<u16>(operand_mode, src_register, dest_register),
                Size::Long => self.subx::<u32>(operand_mode, src_register, dest_register),
                Size::Illegal => panic!(),
            },
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
            Opcode::TRAP { vector } => {
                self.process_exception((vector as u32) + 32);
            }
            Opcode::TRAPV => {
                if self.flag(OVERFLOW) {
                    self.process_exception(7);
                }
            }
            Opcode::TST { mode, size } => match size {
                Size::Byte => self.tst::<i8>(mode),
                Size::Word => self.tst::<i16>(mode),
                Size::Long => self.tst::<i32>(mode),
                Size::Illegal => panic!(),
            },
            Opcode::UNLK { register } => {
                self.set_addr_register(7, self.addr_register(register));
                let val = self.pop();
                self.set_addr_register(register, val);
            }
        }
        self.tick(opcode.cycle_count());
    }

    pub fn next_operation(&mut self, _inputs: &[ControllerState<8>; 2]) {
        if self.stopped {
            self.ticks = 0.0;
        } else {
            if let Some((vdp_interrupt_vector, vdp_interrupt_level)) = {
                let mut vdp_bus = self.vdp_bus.borrow_mut();
                if vdp_bus.horizontal_interrupt {
                    vdp_bus.horizontal_interrupt = false;
                    Some((28, 4))
                } else if vdp_bus.vertical_interrupt {
                    vdp_bus.vertical_interrupt = false;
                    Some((30, 6))
                } else {
                    None
                }
            } {
                self.process_exception(vdp_interrupt_vector);
                self.set_interrupt_level(vdp_interrupt_level);
            }
            self.execute_opcode();
        }
    }

    pub fn close(&mut self) {
        self.vdp.close();
    }
}

#[cfg(feature = "test")]
#[allow(dead_code)]
pub mod testing {
    use gen::m68k::Cpu;
    use gen::m68k::opcodes::{opcode, Opcode};

    impl Cpu<'_> {
        pub fn expand_ram(&mut self, amount: usize) {
            self.internal_ram = vec![0; amount].into_boxed_slice();
            self.test_ram_only = true;
        }

        pub fn init_state(&mut self, pc: u32, sr: u16, d: [u32; 8], a: [u32; 8], ssp: u32) {
            self.pc = pc;
            self.status = sr;
            self.d = d;
            self.a = a;
            self.ssp = ssp;
        }

        pub fn verify_state(
            &self,
            pc: u32,
            sr: u16,
            d: [u32; 8],
            a: [u32; 8],
            ssp: u32,
            sr_mask: u16,
            test_id: &str,
        ) {
            assert_eq!(self.pc, pc, "{}   PC", test_id);
            assert_eq!(
                self.status & sr_mask,
                sr & sr_mask,
                "{}   SR {:016b} {:016b}",
                test_id,
                self.status,
                sr
            );
            for i in 0..8 {
                assert_eq!(self.d[i], d[i], "{}   D{}", test_id, i);
                assert_eq!(self.a[i], a[i], "{}   A{}", test_id, i);
            }
            assert_eq!(self.ssp, ssp, "{}   SSP", test_id);
        }

        pub fn poke_ram(&mut self, addr: u32, val: u8) {
            self.write_addr(addr, val);
        }

        pub fn set_ram(&mut self, contents: &[u8]) {
            let mut vec = contents.to_vec();
            vec.resize(0x1000000, 0);
            self.internal_ram = vec.into_boxed_slice();
            self.test_ram_only = true;
        }

        pub fn peek_opcode(&mut self) -> Opcode {
            opcode(self.read_addr(self.pc))
        }

        pub fn peek_ram_long(&mut self, addr: u32) -> u32 {
            self.read_addr_no_tick(addr)
        }

        pub fn verify_ram(&mut self, addr: u32, val: u8, test_id: &str) {
            assert_eq!(
                self.read_addr::<u8>(addr),
                val,
                "{}   {:06X}",
                test_id,
                addr
            );
        }

        pub fn pc_for_test(&self) -> u32 {
            self.pc
        }
    }
}

impl window::Cpu for Cpu<'_> {
    fn reset(&mut self, _soft: bool) {
        self.ssp = self.read_addr_no_tick(0x000000);
        self.pc = self.read_addr_no_tick(0x000004);
        self.set_interrupt_level(7);
        self.set_flag(SUPERVISOR_MODE, true);
        self.stopped = false;
    }

    fn do_frame(&mut self, time_secs: f64, inputs: &[ControllerState<8>; 2]) {
        self.ticks += time_secs * CPU_TICKS_PER_SECOND * self.speed_adj;

        while self.ticks > 0.0 {
            self.next_operation(inputs);
        }
    }

    fn render(
        &mut self,
        c: Context,
        texture_ctx: &mut G2dTextureContext,
        gl: &mut G2d,
        device: &mut Device,
    ) {
        self.vdp.render(c, texture_ctx, gl, device);
    }

    fn save_state(&self, _out: &mut Vec<u8>) {
        todo!()
    }

    fn load_state(&mut self, _state: &mut dyn Buf) {
        todo!()
    }

    fn increase_speed(&mut self) {
        todo!()
    }

    fn decrease_speed(&mut self) {
        todo!()
    }
}
