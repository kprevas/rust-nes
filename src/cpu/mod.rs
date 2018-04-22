use apu::*;
use apu::bus::*;
use bincode::{deserialize_from, serialize};
use bytes::*;
use cartridge::CartridgeBus;
use input::ControllerState;
use piston_window::{Context, G2d, Glyphs};
use ppu::*;
use ppu::bus::*;
use self::opcodes::AddressingMode;
use self::opcodes::AddressingMode::*;
use self::opcodes::Opcode;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::{Cursor, Result};
use std::io::prelude::*;
use std::ops::Range;

mod opcodes;
pub mod disassembler;

const CPU_TICKS_PER_SECOND: f64 = 1_789_773.0;

pub struct Cpu<'a> {
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    pc: u16,
    oam_dma_write: Option<(u8, u8)>,
    internal_ram: Box<[u8]>,
    cartridge: &'a mut Box<CartridgeBus>,
    ppu: Ppu<'a>,
    ppu_bus: &'a RefCell<PpuBus>,
    apu: Apu<'a>,
    apu_bus: &'a RefCell<ApuBus>,
    controller_strobe: bool,
    last_inputs: [u8;2],
    ticks: f64,
    open_bus: u8,
    instrumented: bool,
    delayed_irq_flag: Option<bool>,
    irq: bool,
    prev_irq: bool,
    dmc_delay: u8,
    cycle_count: u64,

    memory_watches: Box<HashSet<u16>>,
    pc_watches: Box<HashSet<u16>>,
    pc_breaks: Box<HashSet<u16>>,
    pc_ignores: Box<Vec<Range<u16>>>,
}

const CARRY: u8 = 0b1;
const ZERO: u8 = 0b10;
const INTERRUPT: u8 = 0b100;
const DECIMAL: u8 = 0b1000;
const OVERFLOW: u8 = 0b1000000;
const NEGATIVE: u8 = 0b10000000;

impl<'a> Cpu<'a> {
    pub fn boot<'b>(cartridge: &'b mut Box<CartridgeBus>,
                    ppu: Ppu<'b>, ppu_bus: &'b RefCell<PpuBus>,
                    apu: Apu<'b>, apu_bus: &'b RefCell<ApuBus>,
                    instrumented: bool) -> Cpu<'b> {
        let mut cpu = Cpu {
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            sp: 0,
            pc: 0,
            oam_dma_write: None,
            internal_ram: vec![0; 0x800].into_boxed_slice(),
            cartridge,
            ppu,
            ppu_bus,
            apu,
            apu_bus,
            controller_strobe: false,
            last_inputs: [0, 0],
            ticks: 0.0,
            open_bus: 0,
            instrumented,
            pc_watches: Box::new(HashSet::new()),
            memory_watches: Box::new(HashSet::new()),
            pc_breaks: Box::new(HashSet::new()),
            pc_ignores: Box::new(Vec::new()),
            delayed_irq_flag: None,
            irq: false,
            prev_irq: false,
            dmc_delay: 0,
            cycle_count: 0,
        };

        cpu.reset(false);

        cpu
    }

    fn tick(&mut self, write_addr: Option<u16>) {
        if self.dmc_delay > 0 {
            self.dmc_delay -= 1;
        }
        self.ticks -= 1.0;
        for _ in 0..3 {
            self.ppu.tick();
        }
        self.apu.tick(self.cartridge);
        self.ppu_bus.borrow_mut().tick();
        let mut apu_bus = self.apu_bus.borrow_mut();
        if apu_bus.dmc_delay {
            apu_bus.dmc_delay = false;
            self.dmc_delay = match self.oam_dma_write {
                Some((_, 255)) => 3,
                Some((_, 254)) => 1,
                Some(_) => 2,
                None => match write_addr {
                    Some(0x4014) => 2,
                    Some(_) => 3,
                    None => 4,
                },
            }
        }
        if self.oam_dma_write.is_none() && self.dmc_delay == 0 {
            let irq_interrupt = apu_bus.irq_interrupt() && !match self.delayed_irq_flag {
                Some(val) => val,
                None => self.flag(INTERRUPT)
            };
            let ppu_bus = self.ppu_bus.borrow();
            let nmi_interrupt = ppu_bus.nmi_interrupt && ppu_bus.nmi_interrupt_age > 1;
            self.prev_irq = self.irq || nmi_interrupt;
            self.irq = irq_interrupt;
        }
        self.cycle_count = self.cycle_count.wrapping_add(1);
    }

    fn flag(&self, flag: u8) -> bool {
        self.p & flag > 0
    }

    fn set_flag(&mut self, flag: u8, val: bool) {
        if val {
            self.p |= flag
        } else {
            self.p &= !flag;
        }
    }

    fn read_word(&mut self, address: u16) -> u16 {
        let lo_byte = u16::from(self.read_memory(address));
        let hi_byte = u16::from(self.read_memory(address + 1));
        (hi_byte << 8) + lo_byte
    }

    fn read_word_no_tick(&mut self, address: u16) -> u16 {
        (u16::from(self.read_memory_no_tick(address + 1)) << 8) + u16::from(self.read_memory_no_tick(address))
    }

    fn read_word_zeropage_wrapped(&mut self, address: u16) -> u16 {
        (u16::from(self.read_memory(u16::from((address as u8).wrapping_add(1)))) << 8) + u16::from(self.read_memory(address))
    }

    fn read_word_page_wrapped(&mut self, address: u16) -> u16 {
        if address & 0xff == 0xff {
            (u16::from(self.read_memory(address & (!0xff))) << 8) + u16::from(self.read_memory(address))
        } else {
            self.read_word(address)
        }
    }

    pub fn read_memory(&mut self, address: u16) -> u8 {
        self.tick(None);
        while self.dmc_delay > 0 {
            self.tick(None);
            if (address != 0x4016 && address != 0x4017 && self.cycle_count & 1 > 0) || self.dmc_delay == 1 {
                self.read_memory_no_tick(address);
            }
        }
        let value = self.read_memory_no_tick(address);
        self.open_bus = value;
        value
    }

    pub fn read_memory_no_tick(&mut self, address: u16) -> u8 {
        let value = match address {
            0x0000 ... 0x1FFF => self.internal_ram[(address % 0x800) as usize],
            0x2000 ... 0x3FFF => self.ppu_bus.borrow_mut().read(address),
            0x4000 ... 0x4014 => self.open_bus,
            0x4015 => self.apu_bus.borrow_mut().read_status(),
            0x4016 => {
                let value = self.last_inputs[0] & 1;
                self.last_inputs[0] >>= 1;
                value | (self.open_bus & 0xF0)
            }
            0x4017 => {
                let value = self.last_inputs[1] & 1;
                self.last_inputs[1] >>= 1;
                value | (self.open_bus & 0xF0)
            }
            0x4018 ... 0x401F => self.open_bus,
            _ => self.cartridge.read_memory(address, self.open_bus),
        };
        if self.instrumented && self.memory_watches.contains(&address) {
            warn!(target: "cpu", "read memory {:04X} {:02X} {} {}", address, value,
                  self.ppu.instrumentation_short(), self.apu.instrumentation_short());
        }
        value
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        self.tick(Some(address));
        while self.dmc_delay > 0 {
            self.tick(Some(address));
        }
        self.write_memory_no_tick(address, value);
        while self.dmc_delay > 0 {
            self.tick(Some(address));
        }
    }

    fn write_memory_no_tick(&mut self, address: u16, value: u8) {
        if self.instrumented && self.memory_watches.contains(&address) {
            warn!(target: "cpu", "write memory {:04X} {:02X} {} {}", address, value,
                  self.ppu.instrumentation_short(), self.apu.instrumentation_short());
        }
        match address {
            0x0000 ... 0x1FFF => self.internal_ram[(address % 0x800) as usize] = value,
            0x2000 ... 0x3FFF => self.ppu_bus.borrow_mut().write(address, value),
            0x4014 => {
                self.oam_dma_write = Some((value, 0));
                if self.cycle_count & 1 == 0 {
                    self.tick(Some(address));
                }
                self.tick(Some(address));
            }
            0x4000 ... 0x4013 | 0x4015 | 0x4017 => self.apu_bus.borrow_mut().write(address, value),
            0x4016 => self.controller_strobe = value & 1 > 0,
            0x4018 ... 0x401F => (),
            _ => self.cartridge.write_memory(address, value, self.cycle_count),
        }
    }

    fn read_memory_mode(&mut self, mode: &AddressingMode, operand: u16, page_boundary_penalty: bool) -> u8 {
        self.read_memory_mode_with_dummy_read(mode, operand, page_boundary_penalty, false)
    }

    fn read_memory_mode_with_dummy_read(&mut self, mode: &AddressingMode, operand: u16, page_boundary_penalty: bool, dummy_read: bool) -> u8 {
        match *mode {
            Accumulator => self.a,
            Immediate => operand as u8,
            _ => {
                let target = self.apply_memory_mode(mode, operand, page_boundary_penalty, dummy_read);
                self.read_memory(target)
            }
        }
    }

    fn write_memory_mode(&mut self, mode: &AddressingMode, operand: u16, value: u8) {
        self.write_memory_mode_with_dummy_read(mode, operand, value, false)
    }

    fn write_memory_mode_with_dummy_read(&mut self, mode: &AddressingMode, operand: u16, value: u8, dummy_read: bool) {
        match *mode {
            Accumulator => self.a = value,
            Immediate => panic!("Attempted to write with immediate addressing"),
            _ => {
                let address = self.apply_memory_mode(mode, operand, false, dummy_read);
                self.write_memory(address, value)
            }
        }
    }

    fn apply_memory_mode(&mut self, mode: &AddressingMode, operand: u16, page_boundary_penalty: bool, dummy_read: bool) -> u16 {
        match *mode {
            ZeropageX => {
                self.read_memory(operand);
                u16::from((operand as u8).wrapping_add(self.x))
            }
            ZeropageY => {
                self.read_memory(operand);
                u16::from((operand as u8).wrapping_add(self.y))
            }
            Zeropage => operand & 0xff,
            AbsoluteIndexedX => {
                let address = operand.wrapping_add(u16::from(self.x));
                self.dummy_read(operand, page_boundary_penalty, dummy_read, address);
                address
            }
            AbsoluteIndexedY => {
                let address = operand.wrapping_add(u16::from(self.y));
                self.dummy_read(operand, page_boundary_penalty, dummy_read, address);
                address
            }
            IndexedIndirect => {
                self.read_memory(operand);
                let target = u16::from((operand as u8).wrapping_add(self.x));
                self.read_word_zeropage_wrapped(target)
            }
            IndirectIndexed => {
                let offset = u16::from(self.y);
                let target_start = self.read_word_zeropage_wrapped(operand);
                let address = target_start.wrapping_add(offset);
                self.dummy_read(target_start, page_boundary_penalty, dummy_read, address);
                address
            }
            Indirect => self.read_word_page_wrapped(operand),
            Absolute => operand,
            _ => panic!("Invalid memory mode {:?}", mode)
        }
    }

    fn dummy_read(&mut self, operand: u16, page_boundary_penalty: bool, dummy_read: bool, address: u16) {
        if (page_boundary_penalty || dummy_read) && address & (!0xff) != operand & (!0xff) {
            self.read_memory(if address >= 0x100 { address - 0x100 } else { address + 0xFF00 });
        } else if dummy_read {
            self.read_memory(address);
        }
    }

    fn read_modify_write(&mut self, mode: &AddressingMode, operand: u16, modify: &mut FnMut(u8, &mut Cpu) -> u8) -> u8 {
        let (addr, operand_value) = match *mode {
            Accumulator => (0, self.a),
            _ => {
                let addr = self.apply_memory_mode(mode, operand, false, true);
                (addr, self.read_memory(addr))
            }
        };
        let result = modify(operand_value, self);
        match *mode {
            Accumulator => self.a = result,
            _ => {
                self.write_memory(addr, operand_value);
                self.write_memory(addr, result as u8);
            }
        }
        result
    }

    fn set_zero_flag(&mut self, result: u8) {
        self.set_flag(ZERO, result == 0);
    }

    fn set_negative_flag(&mut self, result: u8) {
        self.set_flag(NEGATIVE, result & 0x80 > 0);
    }

    fn set_carry_flag(&mut self, result: u16) {
        self.set_flag(CARRY, result > 0xff);
    }

    fn set_overflow_flag(&mut self, result: u16, a: u8, operand: u8, sbc: bool) {
        let a_sign = a & 0x80 > 0;
        let op_sign = operand & 0x80 > 0;
        let result_sign = result & 0x80 > 0;
        self.set_flag(OVERFLOW, ((a_sign ^ op_sign) ^ (!sbc)) & (a_sign ^ result_sign));
    }

    fn push(&mut self, value: u8) {
        let stack_pointer = 0x100 + u16::from(self.sp);
        self.write_memory(stack_pointer, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_word(&mut self, value: u16) {
        self.push(((value & 0xff00) >> 8) as u8);
        self.push((value & 0xff) as u8);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let stack_pointer = 0x100 + u16::from(self.sp);
        let value = self.read_memory(stack_pointer);
        value
    }

    fn pop_word(&mut self) -> u16 {
        u16::from(self.pop()) + (u16::from(self.pop()) << 8)
    }

    fn branch(&mut self, offset: i8) {
        let prev_pc = self.pc;
        if offset >= 0 {
            self.pc = self.pc.wrapping_add(offset as u16);
        } else {
            self.pc = self.pc.wrapping_sub((-(offset as i16)) as u16);
        }
        if prev_pc >> 8 != self.pc >> 8 {
            self.tick(None);
        } else if self.irq && !self.prev_irq {
            self.irq = false;
        }
        self.tick(None);
    }

    fn adc(&mut self, operand_value: u8) {
        let prev_a = self.a;
        let result = u16::from(self.a) + u16::from(operand_value) + (if self.flag(CARRY) { 1 } else { 0 });
        self.a = (result & 0xff) as u8;
        self.set_zero_flag(result as u8);
        self.set_negative_flag(result as u8);
        self.set_carry_flag(result);
        self.set_overflow_flag(result, prev_a, operand_value, false);
    }

    fn execute_opcode(&mut self) {
        use self::opcodes::OPCODES;
        use self::Opcode::*;

        let opcode_pc = self.pc;
        let opcode_hex = self.read_memory(opcode_pc);
        self.pc += 1;

        let (ref opcode, ref mode) = OPCODES[usize::from(opcode_hex)];
        let operand_pc = self.pc;
        let operand = match mode.bytes() {
            0 => {
                self.read_memory(operand_pc);
                0
            }
            1 => u16::from(self.read_memory(operand_pc)),
            2 => self.read_word(operand_pc),
            _ => panic!("too many bytes")
        };
        self.pc += u16::from(mode.bytes());

        if self.instrumented {
            let pc = self.pc - u16::from(mode.bytes()) - 1;
            if self.pc_breaks.contains(&pc) {
                panic!("{:04X}\t{:02X} {}\t{:?} {}\t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} ppu:{} apu:{} cyc:{}",
                       pc,
                       opcode_hex,
                       match mode.bytes() {
                           1 => format!("{:02X}", operand),
                           2 => format!("{:02X} {:02X}", operand & 0xff, operand >> 8),
                           _ => String::from(""),
                       },
                       opcode,
                       mode.format_operand(operand, self.pc),
                       self.a, self.x, self.y, self.p, self.sp,
                       self.ppu.instrumentation_short(),
                       self.apu.instrumentation_short(),
                       self.cycle_count);
            } else if self.pc_watches.contains(&pc) {
                warn!(target: "cpu", "{:04X}\t{:02X} {}\t{:?} {}\t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} ppu:{} apu:{} cyc:{}",
                      pc,
                      opcode_hex,
                      match mode.bytes() {
                          1 => format!("{:02X}", operand),
                          2 => format!("{:02X} {:02X}", operand & 0xff, operand >> 8),
                          _ => String::from(""),
                      },
                      opcode,
                      mode.format_operand(operand, self.pc),
                      self.a, self.x, self.y, self.p, self.sp,
                      self.ppu.instrumentation_short(),
                      self.apu.instrumentation_short(),
                      self.cycle_count);
            } else {
                let mut ignore = false;
                for ignore_range in self.pc_ignores.iter() {
                    if ignore_range.start <= pc && ignore_range.end >= pc {
                        ignore = true;
                        break;
                    }
                }
                if !ignore {
                    debug!(target: "cpu", "{:04X}\t{:02X} {}\t{:?} {}\t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} ppu:{} apu:{} cyc:{}",
                           pc,
                           opcode_hex,
                           match mode.bytes() {
                               1 => format!("{:02X}", operand),
                               2 => format!("{:02X} {:02X}", operand & 0xff, operand >> 8),
                               _ => String::from(""),
                           },
                           opcode,
                           mode.format_operand(operand, self.pc),
                           self.a, self.x, self.y, self.p, self.sp,
                           self.ppu.instrumentation_short(),
                           self.apu.instrumentation_short(),
                           self.cycle_count);
                }
            }
        }

        match *opcode {
            ADC => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                self.adc(operand_value);
            }

            AHX => {
                let addr = self.apply_memory_mode(mode, operand, false, true);
                let addr_high = (addr >> 8) as u8;
                let result = self.a & self.x & addr_high;
                self.write_memory(addr, result);
            }

            ALR => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let and_result = self.a & operand_value;
                let result = and_result >> 1;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.set_flag(CARRY, and_result & 0b1 > 0);
                self.a = result;
            }

            ANC => {
                let result = self.a & self.read_memory_mode(mode, operand, true);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.set_flag(CARRY, result & (1 << 7) > 0);
                self.a = result;
            }

            AND => {
                let result = self.a & self.read_memory_mode(mode, operand, true);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.a = result;
            }

            ARR => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let and_result = self.a & operand_value;
                let result = (and_result >> 1) + if self.flag(CARRY) { 0x80 } else { 0 };
                self.set_flag(CARRY, result & (1 << 6) > 0);
                self.set_flag(OVERFLOW, (result & (1 << 6) > 0) != (result & (1 << 5) > 0));
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.a = result;
            }

            ASL => {
                let result = self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    let result = u16::from(operand_value) << 1;
                    cpu.set_carry_flag(result);
                    result as u8
                });
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            AXS => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let left_operand = self.a & self.x;
                let result = left_operand.wrapping_sub(operand_value);
                let carry = left_operand >= operand_value;
                self.set_flag(CARRY, carry);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.x = result;
            }

            BCC => {
                if !self.flag(CARRY) {
                    self.branch(operand as i8);
                }
            }

            BCS => {
                if self.flag(CARRY) {
                    self.branch(operand as i8);
                }
            }

            BEQ => {
                if self.flag(ZERO) {
                    self.branch(operand as i8);
                }
            }

            BIT => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = self.a & operand_value;
                self.set_zero_flag(result);
                self.set_negative_flag(operand_value);
                self.set_flag(OVERFLOW, operand_value & 0x40 > 0);
            }

            BMI => {
                if self.flag(NEGATIVE) {
                    self.branch(operand as i8);
                }
            }

            BNE => {
                if !self.flag(ZERO) {
                    self.branch(operand as i8);
                }
            }

            BPL => {
                if !self.flag(NEGATIVE) {
                    self.branch(operand as i8);
                }
            }

            BRK => {
                let old_pc = self.pc + 1;
                let p = self.p | 0b00110000;

                self.push_word(old_pc);
                let vector = if self.ppu_bus.borrow().nmi_interrupt { 0xFFFA } else { 0xFFFE };
                self.push(p);
                self.pc = self.read_word(vector);
                self.set_flag(INTERRUPT, true);

                self.prev_irq = false;
            }

            BVC => {
                if !self.flag(OVERFLOW) {
                    self.branch(operand as i8);
                }
            }

            BVS => {
                if self.flag(OVERFLOW) {
                    self.branch(operand as i8);
                }
            }

            CLC => {
                self.set_flag(CARRY, false);
            }

            CLD => {
                self.set_flag(DECIMAL, false);
            }

            CLI => {
                self.delayed_irq_flag = Some(self.flag(INTERRUPT));
                self.set_flag(INTERRUPT, false);
            }

            CLV => {
                self.set_flag(OVERFLOW, false);
            }

            CMP => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = self.a.wrapping_sub(operand_value);
                let carry = self.a >= operand_value;
                self.set_flag(CARRY, carry);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            CPX => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = self.x.wrapping_sub(operand_value);
                let carry = self.x >= operand_value;
                self.set_flag(CARRY, carry);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            CPY => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = self.y.wrapping_sub(operand_value);
                let carry = self.y >= operand_value;
                self.set_flag(CARRY, carry);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            DCP => {
                self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    let dec_result = operand_value.wrapping_sub(1);
                    let result = cpu.a.wrapping_sub(dec_result);
                    let carry = cpu.a >= dec_result;
                    cpu.set_flag(CARRY, carry);
                    cpu.set_zero_flag(result);
                    cpu.set_negative_flag(result);
                    dec_result
                });
            }

            DEC => {
                let result = self.read_modify_write(mode, operand, &mut |operand_value, _cpu| {
                    operand_value.wrapping_sub(1)
                });
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            DEX => {
                let result = self.x.wrapping_sub(1);
                self.x = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            DEY => {
                let result = self.y.wrapping_sub(1);
                self.y = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            EOR => {
                let result = self.a ^ self.read_memory_mode(mode, operand, true);
                self.a = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            INC => {
                let result = self.read_modify_write(mode, operand, &mut |operand_value, _cpu| {
                    operand_value.wrapping_add(1)
                });
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            INX => {
                let result = self.x.wrapping_add(1);
                self.x = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            INY => {
                let result = self.y.wrapping_add(1);
                self.y = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            ISC => {
                let result = self.read_modify_write(mode, operand, &mut |operand_value, _cpu| {
                    operand_value.wrapping_add(1)
                });
                self.adc(!result);
            }

            JMP => {
                self.pc = self.apply_memory_mode(mode, operand, false, false);
            }

            JSR => {
                let old_pc = self.pc - 1;
                self.push_word(old_pc);
                let new_pc = self.apply_memory_mode(mode, operand, false, false);
                self.pc = new_pc;
                self.tick(None);
            }

            LAS => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = operand_value & self.sp;
                self.a = result;
                self.x = result;
                self.sp = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            LAX => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                self.a = operand_value;
                self.x = operand_value;
                self.set_zero_flag(operand_value);
                self.set_negative_flag(operand_value);
            }

            LDA => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                self.a = operand_value;
                self.set_zero_flag(operand_value);
                self.set_negative_flag(operand_value);
            }

            LDX => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                self.x = operand_value;
                self.set_zero_flag(operand_value);
                self.set_negative_flag(operand_value);
            }

            LDY => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                self.y = operand_value;
                self.set_zero_flag(operand_value);
                self.set_negative_flag(operand_value);
            }

            LSR => {
                let result = self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    cpu.set_flag(CARRY, operand_value & 0b1 > 0);
                    operand_value >> 1
                });
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            NOP => {
                match *mode {
                    Implied => (),
                    _ => { self.read_memory_mode(mode, operand, true); }
                };
            }

            ORA => {
                let result = self.a | self.read_memory_mode(mode, operand, true);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.a = result;
            }

            PHA => {
                let value = self.a;
                self.push(value);
            }

            PHP => {
                let value = self.p | 0b00110000;
                self.push(value);
            }

            PLA => {
                let value = self.pop();
                self.a = value;
                self.set_zero_flag(value);
                self.set_negative_flag(value);
                self.tick(None);
            }

            PLP => {
                self.delayed_irq_flag = Some(self.flag(INTERRUPT));
                let value = self.pop();
                self.p = (value & 0b11101111) | 0b00100000;
                self.tick(None);
            }

            RLA => {
                let rol_result = self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    let new_carry = operand_value & 0x80 > 0;
                    let rol_result = (operand_value << 1) + if cpu.flag(CARRY) { 1 } else { 0 };
                    cpu.set_flag(CARRY, new_carry);
                    rol_result
                });
                let result = self.a & rol_result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.a = result;
            }

            ROL => {
                let result = self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    let new_carry = operand_value & 0x80 > 0;
                    let result = (operand_value << 1) + if cpu.flag(CARRY) { 1 } else { 0 };
                    cpu.set_flag(CARRY, new_carry);
                    result
                });
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            ROR => {
                let result = self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    let new_carry = operand_value & 0x1 > 0;
                    let result = (operand_value >> 1) + if cpu.flag(CARRY) { 0x80 } else { 0 };
                    cpu.set_flag(CARRY, new_carry);
                    result
                });
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            RRA => {
                self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    let ror_carry = (operand_value & 0x1) as u16;
                    let ror_result = (operand_value >> 1) + if cpu.flag(CARRY) { 0x80 } else { 0 };
                    let prev_a = cpu.a;
                    let result = u16::from(cpu.a) + u16::from(ror_result) + ror_carry;
                    cpu.a = (result & 0xff) as u8;
                    cpu.set_zero_flag(result as u8);
                    cpu.set_negative_flag(result as u8);
                    cpu.set_carry_flag(result);
                    cpu.set_overflow_flag(result, prev_a, ror_result, false);
                    ror_result
                });
            }

            RTI => {
                let p = self.pop();
                self.p = p;
                let pc = self.pop_word();
                self.pc = pc;
                self.tick(None);
            }

            RTS => {
                self.tick(None);
                let pc = self.pop_word();
                self.pc = pc + 1;
                self.tick(None);
            }

            SAX => {
                let value = self.a & self.x;
                self.write_memory_mode(mode, operand, value);
            }

            SBC => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                self.adc(!operand_value);
            }

            SEC => {
                self.set_flag(CARRY, true);
            }

            SED => {
                self.set_flag(DECIMAL, true);
            }

            SEI => {
                self.delayed_irq_flag = Some(self.flag(INTERRUPT));
                self.set_flag(INTERRUPT, true);
            }

            SHX => {
                let addr = self.apply_memory_mode(mode, operand, true, true);
                let addr_high = addr >> 8;
                let addr_low = addr & 0xFF;
                let value = self.x & ((addr_high + 1) as u8);
                self.write_memory((u16::from(value) << 8) | addr_low, value);
            }

            SHY => {
                let addr = self.apply_memory_mode(mode, operand, true, true);
                let addr_high = addr >> 8;
                let addr_low = addr & 0xFF;
                let value = self.y & ((addr_high + 1) as u8);
                self.write_memory((u16::from(value) << 8) | addr_low, value);
            }

            SLO => {
                let asl_result = self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    let result = u16::from(operand_value) << 1;
                    cpu.set_carry_flag(result);
                    result as u8
                });
                let result = self.a | asl_result as u8;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.a = result;
            }

            SRE => {
                let lsr_result = self.read_modify_write(mode, operand, &mut |operand_value, cpu| {
                    cpu.set_flag(CARRY, operand_value & 0b1 > 0);
                    operand_value >> 1
                });
                let result = self.a ^ lsr_result;
                self.a = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            STA => {
                let value = self.a;
                self.write_memory_mode_with_dummy_read(mode, operand, value, true);
            }

            STX => {
                let value = self.x;
                self.write_memory_mode(mode, operand, value);
            }

            STY => {
                let value = self.y;
                self.write_memory_mode(mode, operand, value);
            }

            TAS => {
                let addr = self.apply_memory_mode(mode, operand, true, true);
                let addr_high = (addr >> 8) as u8;
                self.sp = self.a & self.x;
                let result = self.sp & addr_high;
                self.write_memory(addr, result);
            }

            TAX => {
                let value = self.a;
                self.x = value;
                self.set_zero_flag(value);
                self.set_negative_flag(value);
            }

            TAY => {
                let value = self.a;
                self.y = value;
                self.set_zero_flag(value);
                self.set_negative_flag(value);
            }

            TSX => {
                let value = self.sp;
                self.x = value;
                self.set_zero_flag(value);
                self.set_negative_flag(value);
            }

            TXA => {
                let value = self.x;
                self.a = value;
                self.set_zero_flag(value);
                self.set_negative_flag(value);
            }

            TXS => {
                let value = self.x;
                self.sp = value;
            }

            TYA => {
                let value = self.y;
                self.a = value;
                self.set_zero_flag(value);
                self.set_negative_flag(value);
            }

            XAA => {
                let value = self.a;
                self.x = value & self.read_memory_mode(mode, operand, true);
                self.set_zero_flag(value);
                self.set_negative_flag(value);
            }

            XXX => { unimplemented!("{:02X}", opcode_hex) }
        }
    }

    fn irq(&mut self) {
        let old_pc = self.pc;
        let p = self.p | 0b00100000;
        self.read_memory(old_pc);
        self.read_memory(old_pc);
        self.push_word(old_pc);
        self.push(p);
        let vector = if self.ppu_bus.borrow().nmi_interrupt { 0xFFFA } else { 0xFFFE };
        self.pc = self.read_word(vector);
        if vector == 0xFFFA {
            self.ppu_bus.borrow_mut().nmi_interrupt = false;
        }
        self.set_flag(INTERRUPT, true);
    }

    pub fn next_operation(&mut self, inputs: &[ControllerState;2]) {
        if self.controller_strobe {
            self.last_inputs = [inputs[0].to_u8(), inputs[1].to_u8()];
        }
        if let Some((addr, i)) = self.oam_dma_write {
            let data = self.read_memory(u16::from(addr) * 0x100 + u16::from(i));
            self.write_memory(0x2004, data);
            self.oam_dma_write = if i < 255 { Some((addr, i + 1)) } else { None };
        } else {
            self.delayed_irq_flag = None;
            self.execute_opcode();
            if self.prev_irq {
                self.irq();
            }
        }
    }

    pub fn do_frame(&mut self, time_secs: f64, inputs: &[ControllerState;2]) {
        self.ticks += time_secs * CPU_TICKS_PER_SECOND;

        while self.ticks > 0.0 {
            self.next_operation(inputs);
        }
    }

    pub fn render(&mut self, c: Context, gl: &mut G2d, glyphs: &mut Glyphs) {
        self.ppu.render(c, gl, glyphs);
    }

    pub fn reset(&mut self, soft: bool) {
        if soft {
            self.sp -= 3;
            self.p |= 0x4;
        } else {
            self.sp = 0xfd;
            self.p = 0x34;
        };
        self.pc = self.read_word_no_tick(0xFFFC);

        self.apu_bus.borrow_mut().reset(soft);
        self.write_memory_no_tick(0x2000, 0);
        self.write_memory_no_tick(0x2001, 0);

        for _ in 0..28 {
            self.ppu.tick();
        };
        for _ in 0..9 {
            self.apu.tick(self.cartridge);
        }
    }

    pub fn close(&mut self) {
        self.apu.close();
        self.ppu.close();
    }

    pub fn save_to_battery(&self, out: &mut Write) -> Result<usize> {
        let result = self.cartridge.save_to_battery(out);
        if let Ok(bytes) = result {
            info!(target: "cartridge", "{} bytes written", bytes);
        } else {
            info!(target: "cartridge", "no save data written");
        }
        result
    }

    pub fn save_state(&self, out: &mut Vec<u8>) {
        out.put_u8(self.a);
        out.put_u8(self.x);
        out.put_u8(self.y);
        out.put_u8(self.p);
        out.put_u8(self.sp);
        out.put_u16::<BigEndian>(self.pc);
        out.put_slice(&serialize(&self.oam_dma_write).unwrap());
        out.put_slice(&self.internal_ram);
        out.put_u8(if self.controller_strobe { 1 } else { 0 });
        out.put_u8(self.last_inputs[0]);
        out.put_u8(self.last_inputs[1]);
        out.put_f64::<BigEndian>(self.ticks);
        out.put_u8(self.open_bus);
        out.put_slice(&serialize(&self.delayed_irq_flag).unwrap());
        out.put_u8(if self.irq { 1 } else { 0 });
        out.put_u8(if self.prev_irq { 1 } else { 0 });
        out.put_u8(self.dmc_delay);
        out.put_u64::<BigEndian>(self.cycle_count);
        self.cartridge.save_state(out);
        self.ppu.save_state(out);
        self.ppu_bus.borrow().save_state(out);
        self.apu.save_state(out);
        self.apu_bus.borrow().save_state(out);
    }

    pub fn load_state(&mut self, state: &mut Cursor<Vec<u8>>) {
        self.a = state.get_u8();
        self.x = state.get_u8();
        self.y = state.get_u8();
        self.p = state.get_u8();
        self.sp = state.get_u8();
        self.pc = state.get_u16::<BigEndian>();
        self.oam_dma_write = deserialize_from(state.reader()).unwrap();
        state.copy_to_slice(&mut self.internal_ram);
        self.controller_strobe = state.get_u8() == 1;
        self.last_inputs[0] = state.get_u8();
        self.last_inputs[1] = state.get_u8();
        self.ticks = state.get_f64::<BigEndian>();
        self.open_bus = state.get_u8();
        self.delayed_irq_flag = deserialize_from(state.reader()).unwrap();
        self.irq = state.get_u8() == 1;
        self.prev_irq = state.get_u8() == 1;
        self.dmc_delay = state.get_u8();
        self.cycle_count = state.get_u64::<BigEndian>();
        self.cartridge.load_state(state);
        self.ppu.load_state(state);
        self.ppu_bus.borrow_mut().load_state(state);
        self.apu.load_state(state);
        self.apu_bus.borrow_mut().load_state(state);
    }

    pub fn setup_for_test(&mut self, p_start: u8, pc_start: u16) {
        self.p = p_start;
        self.pc = pc_start;
    }

    pub fn pc_for_test(&self) -> u16 {
        self.pc
    }

    pub fn a_for_test(&self) -> u8 {
        self.a
    }

    pub fn set_pc_watch(&mut self, addr: u16) {
        self.pc_watches.insert(addr);
    }

    pub fn set_pc_break(&mut self, addr: u16) {
        self.pc_breaks.insert(addr);
    }

    pub fn add_pc_ignore_range(&mut self, range: Range<u16>) {
        self.pc_ignores.push(range);
    }

    pub fn set_memory_watch(&mut self, addr: u16) {
        self.memory_watches.insert(addr);
    }
}