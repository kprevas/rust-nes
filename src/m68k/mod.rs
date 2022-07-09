use std::convert::TryFrom;
use std::marker::PhantomData;

use dasp::Sample;
use input::ControllerState;
use m68k::opcodes::{AddressingMode, brief_extension_word, opcode, Opcode, OperandDirection, Size};

mod opcodes;

const CPU_TICKS_PER_SECOND: f64 = 7_670_454.0;

trait DataSize: TryFrom<u32> {
    fn max_value() -> Self;
    fn address_size() -> u32;
    fn word_aligned_address_size() -> u32;
    fn from(value: u32) -> Self {
        Self::try_from(value).unwrap_or(Self::max_value())
    }
    fn apply_to_register(self, register_val: u32) -> u32;
}

impl DataSize for u8 {
    fn max_value() -> Self {
        Self::MAX
    }
    fn address_size() -> u32 {
        1
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn apply_to_register(self, register_val: u32) -> u32 {
        (register_val & !0xFF) + (self as u32)
    }
}

impl DataSize for u16 {
    fn max_value() -> Self {
        Self::MAX
    }
    fn address_size() -> u32 {
        2
    }
    fn word_aligned_address_size() -> u32 {
        2
    }

    fn apply_to_register(self, register_val: u32) -> u32 {
        (register_val & !0xFFFF) + (self as u32)
    }
}

impl DataSize for u32 {
    fn max_value() -> Self {
        Self::MAX
    }
    fn address_size() -> u32 {
        4
    }
    fn word_aligned_address_size() -> u32 {
        4
    }

    fn apply_to_register(self, register_val: u32) -> u32 {
        self
    }
}

pub struct Cpu<'a> {
    a: [u32; 8],
    d: [u32; 8],
    status: u16,
    pc: u32,
    ticks: f64,
    instrumented: bool,
    cycle_count: u64,

    pub speed_adj: f64,

    phantom: PhantomData<&'a u8>,
}

const INTERRUPT: u16 = 0b0000011100000000;
const INTERRUPT_SHIFT: u16 = 8;

impl<'a> Cpu<'a> {
    pub fn boot<'b>(instrumented: bool) -> Cpu<'b> {
        let mut cpu = Cpu {
            a: [0, 0, 0, 0, 0, 0, 0, 0],
            d: [0, 0, 0, 0, 0, 0, 0, 0],
            status: 0,
            pc: 0,
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

    fn set_interrupt_level(&mut self, level: u16) {
        assert!(level <= (INTERRUPT >> INTERRUPT_SHIFT));
        self.status = (self.status & (!INTERRUPT)) | (level << INTERRUPT_SHIFT)
    }

    fn read_addr<Size: DataSize>(&mut self, _addr: u32) -> Size {
        Size::from(0)
    }

    fn read_addr_word_aligned<Size: DataSize>(&mut self, _addr: u32) -> Size {
        Size::from(0)
    }

    fn read_addr_no_tick<Size: DataSize>(&mut self, _addr: u32) -> Size {
        Size::from(0)
    }

    fn read<Size: DataSize>(&mut self, mode: AddressingMode) -> Size {
        match mode {
            AddressingMode::DataRegister(register) => Size::from(self.d[register]),
            AddressingMode::AddressRegister(register) => Size::from(self.a[register]),
            AddressingMode::Address(register) => self.read_addr(self.a[register]),
            AddressingMode::AddressWithPostincrement(register) => {
                let val = self.read_addr(self.a[register]);
                self.a[register] += if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                };
                val
            }
            AddressingMode::AddressWithPredecrement(register) => {
                self.a[register] -= if register == 7 {
                    Size::word_aligned_address_size()
                } else {
                    Size::address_size()
                };
                self.read_addr(self.a[register])
            }
            AddressingMode::AddressWithDisplacement(register) => {
                let displacement: i16 = self.read_addr::<u16>(self.pc).to_signed_sample();
                self.pc += 2;
                let addr = self.a[register].wrapping_add_signed(displacement as i32);
                self.read_addr(addr)
            }
            AddressingMode::AddressWithIndex(register) => {
                let extension = self.read_addr::<u16>(self.pc);
                self.pc += 2;
                let (ext_mode, size, index) = brief_extension_word(extension);
                let ext_register_value = match size {
                    opcodes::Size::Word => self.read::<u16>(ext_mode) as u32,
                    opcodes::Size::Long => self.read::<u32>(ext_mode),
                    _ => panic!(),
                };
                let addr = self.a[register]
                    .wrapping_add(ext_register_value)
                    .wrapping_add_signed(index as i32);
                self.read_addr(addr)
            }
            AddressingMode::ProgramCounterWithDisplacement => {
                let displacement: i16 = self.read_addr::<u16>(self.pc).to_signed_sample();
                let addr = self.pc.wrapping_add_signed(displacement as i32);
                self.pc += 2;
                self.read_addr(addr)
            }
            AddressingMode::ProgramCounterWithIndex => {
                let extension = self.read_addr::<u16>(self.pc);
                let (ext_mode, size, index) = brief_extension_word(extension);
                let ext_register_value = match size {
                    opcodes::Size::Word => self.read::<u16>(ext_mode) as u32,
                    opcodes::Size::Long => self.read::<u32>(ext_mode),
                    _ => panic!(),
                };
                let addr = self.pc
                    .wrapping_add(ext_register_value)
                    .wrapping_add_signed(index as i32);
                self.pc += 2;
                self.read_addr(addr)
            }
            AddressingMode::AbsoluteShort => {
                let extension = self.read_addr::<u16>(self.pc);
                self.pc += 2;
                let short_addr = extension.to_signed_sample();
                let addr = if short_addr < 0 {
                    u32::MAX - (short_addr + 1) as u32
                } else {
                    short_addr as u32
                };
                self.read_addr(addr)
            }
            AddressingMode::AbsoluteLong => {
                let addr = self.read_addr::<u32>(self.pc);
                self.pc += 4;
                self.read_addr(addr)
            }
            AddressingMode::Immediate => {
                let val = self.read_addr_word_aligned::<Size>(self.pc);
                self.pc += Size::word_aligned_address_size();
                val
            }
            AddressingMode::Illegal => panic!(),
        }
    }

    fn write<Size: DataSize>(&mut self, mode: AddressingMode, val: Size) {}

    fn execute_opcode(&mut self) {
        let opcode_pc = self.pc;
        let opcode_hex = self.read_addr(opcode_pc);
        self.pc += 2;

        let opcode = opcode(opcode_hex);

        match opcode {
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
        self.a[7] = self.read_addr_no_tick(0x000000);
        self.pc = self.read_addr_no_tick(0x000004);
        self.set_interrupt_level(7);
    }

    pub fn close(&mut self) {}
}
