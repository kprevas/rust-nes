mod opcodes;
pub mod disassembler;

use std::cell::RefCell;

use self::opcodes::Opcode;
use self::opcodes::AddressingMode;
use self::opcodes::AddressingMode::*;

use cartridge::CartridgeBus;
use input::ControllerState;
use ppu::bus::*;

pub struct Cpu<'a> {
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    pc: u16,
    cycles_to_next: u16,
    oam_dma_write: Option<(u8, u8)>,
    internal_ram: Box<[u8]>,
    cartridge: &'a mut Box<CartridgeBus>,
    ppu_bus: &'a RefCell<PpuBus>,
    controller_strobe: bool,
    last_inputs: u8,
}

const CARRY: u8 = 0b1;
const ZERO: u8 = 0b10;
const INTERRUPT: u8 = 0b100;
const DECIMAL: u8 = 0b1000;
const OVERFLOW: u8 = 0b1000000;
const NEGATIVE: u8 = 0b10000000;

impl<'a> Cpu<'a> {
    pub fn boot<'b>(cartridge: &'b mut Box<CartridgeBus>, ppu_bus: &'b RefCell<PpuBus>) -> Cpu<'b> {
        let mut cpu = Cpu {
            a: 0,
            x: 0,
            y: 0,
            p: 0x34,
            sp: 0xfd,
            pc: 0,
            cycles_to_next: 0,
            oam_dma_write: None,
            internal_ram: vec![0; 0x800].into_boxed_slice(),
            cartridge,
            ppu_bus,
            controller_strobe: false,
            last_inputs: Default::default(),
        };

        cpu.pc = cpu.read_word(0xfffc);
        cpu.cycles_to_next = 0;

        cpu
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
        (u16::from(self.read_memory(address + 1)) << 8) + u16::from(self.read_memory(address))
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
        self.cycles_to_next += 1;
        match address {
            0x0000 ... 0x1FFF => self.internal_ram[(address % 0x800) as usize],
            0x2000 ... 0x3FFF => self.ppu_bus.borrow_mut().read(address),
            0x4016 => {
                let value = self.last_inputs & 1;
                self.last_inputs >>= 1;
                value
            },
            0x4017 =>
            // TODO joypad 2
                0,
            0x4000 ... 0x4019 =>
            // TODO APU and I/O registers
                0,
            _ => self.cartridge.read_memory(address),
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        self.cycles_to_next += 1;
        match address {
            0x0000 ... 0x1FFF => self.internal_ram[(address % 0x800) as usize] = value,
            0x2000 ... 0x3FFF => self.ppu_bus.borrow_mut().write(address, value),
            0x4014 => {
                let data = self.read_memory(u16::from(value) * 0x100);
                self.write_memory(0x2014, data);
                self.oam_dma_write = Some((value, 1));
            }
            0x4016 => self.controller_strobe = value & 1 > 0,
            0x4000 ... 0x4019 =>
            // TODO APU and I/O registers
                (),
            _ =>
            // TODO cartridge space
                (),
        }
    }

    fn read_memory_mode(&mut self, mode: &AddressingMode, operand: u16, page_boundary_penalty: bool) -> u8 {
        match *mode {
            Accumulator => self.a,
            Immediate => operand as u8,
            _ => {
                let target = self.apply_memory_mode(mode, operand, page_boundary_penalty);
                self.read_memory(target)
            }
        }
    }

    fn write_memory_mode(&mut self, mode: &AddressingMode, operand: u16, value: u8) {
        match *mode {
            Accumulator => self.a = value,
            Immediate => panic!("Attempted to write with immediate addressing"),
            _ => {
                let address = self.apply_memory_mode(mode, operand, false);
                self.write_memory(address, value)
            }
        }
    }

    fn apply_memory_mode(&mut self, mode: &AddressingMode, operand: u16, page_boundary_penalty: bool) -> u16 {
        match *mode {
            ZeropageX => {
                self.cycles_to_next += 1;
                u16::from((operand as u8).wrapping_add(self.x))
            }
            ZeropageY => {
                self.cycles_to_next += 1;
                u16::from((operand as u8).wrapping_add(self.y))
            }
            Zeropage => operand & 0xff,
            AbsoluteIndexedX => {
                let address = operand.wrapping_add(u16::from(self.x));
                if page_boundary_penalty && address & (!0xff) != operand & (!0xff) {
                    self.cycles_to_next += 1;
                }
                address
            }
            AbsoluteIndexedY => {
                let address = operand.wrapping_add(u16::from(self.y));
                if page_boundary_penalty && address & (!0xff) != operand & (!0xff) {
                    self.cycles_to_next += 1;
                }
                address
            }
            IndexedIndirect => {
                self.cycles_to_next += 1;
                let target = u16::from((operand as u8).wrapping_add(self.x));
                self.read_word_zeropage_wrapped(target)
            }
            IndirectIndexed => {
                let offset = u16::from(self.y);
                let target_start = self.read_word_zeropage_wrapped(operand);
                let address = target_start.wrapping_add(offset);
                if page_boundary_penalty && address & (!0xff) != target_start & (!0xff) {
                    self.cycles_to_next += 1;
                }
                address
            }
            Indirect => self.read_word_page_wrapped(operand),
            Absolute => operand,
            _ => panic!("Invalid memory mode {:?}", mode)
        }
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
        self.cycles_to_next += 1;
        let prev_pc = self.pc;
        if offset >= 0 {
            self.pc = self.pc.wrapping_add(offset as u16);
        } else {
            self.pc = self.pc.wrapping_sub((-offset) as u16);
        }
        if prev_pc >> 8 != self.pc >> 8 {
            self.cycles_to_next += 1;
        }
    }

    fn execute_opcode(&mut self, instrument: bool) {
        use self::opcodes::OPCODES;
        use self::Opcode::*;

        let opcode_pc = self.pc;
        let opcode_hex = self.read_memory(opcode_pc);
        self.pc += 1;

        let (ref opcode, ref mode) = OPCODES[usize::from(opcode_hex)];
        let operand_pc = self.pc;
        let operand = match mode.bytes() {
            0 => {
                self.cycles_to_next += 1;
                0
            }
            1 => u16::from(self.read_memory(operand_pc)),
            2 => self.read_word(operand_pc),
            _ => panic!("too many bytes")
        };
        self.pc += u16::from(mode.bytes());

        if instrument {
            debug!(target: "cpu", "{:04X}\t{:02X} {}\t{:?} {}\t\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                   self.pc - u16::from(mode.bytes()) - 1,
                   opcode_hex,
                   match mode.bytes() {
                       1 => format!("{:02X}", operand),
                       2 => format!("{:02X} {:02X}", operand & 0xff, operand >> 8),
                       _ => String::from(""),
                   },
                   opcode,
                   mode.format_operand(operand, self.pc),
                   self.a, self.x, self.y, self.p, self.sp);
        }

        match *opcode {
            ADC => {
                let prev_a = self.a;
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = u16::from(self.a) + u16::from(operand_value) + (if self.flag(CARRY) { 1 } else { 0 });
                self.a = (result & 0xff) as u8;
                self.set_zero_flag(result as u8);
                self.set_negative_flag(result as u8);
                self.set_carry_flag(result);
                self.set_overflow_flag(result, prev_a, operand_value, false);
            }

            AND => {
                let result = self.a & self.read_memory_mode(mode, operand, true);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.a = result;
            }

            ASL => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = u16::from(operand_value) << 1;
                self.write_memory_mode(mode, operand, result as u8);
                self.set_zero_flag(result as u8);
                self.set_negative_flag(result as u8);
                self.set_carry_flag(result);
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
                let old_pc = self.pc;
                let p = self.p | 0b00110000;

                self.push_word(old_pc);
                self.push(p);
                self.pc = self.read_word(0xFFFE);
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
                let operand_value = self.read_memory_mode(mode, operand, false);
                let dec_result = operand_value.wrapping_sub(1);
                self.write_memory_mode(mode, operand, dec_result);
                let result = self.a.wrapping_sub(dec_result);
                let carry = self.a >= dec_result;
                self.set_flag(CARRY, carry);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            DEC => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = operand_value.wrapping_sub(1);
                self.write_memory_mode(mode, operand, result);
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
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = operand_value.wrapping_add(1);
                self.write_memory_mode(mode, operand, result);
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
                let operand_value = self.read_memory_mode(mode, operand, false);
                let inc_result = operand_value.wrapping_add(1);
                self.write_memory_mode(mode, operand, inc_result);
                let prev_a = self.a;
                let result = u16::from(self.a).wrapping_sub(u16::from(inc_result) + (if self.flag(CARRY) { 0 } else { 1 }));
                self.a = (result & 0xff) as u8;
                self.set_zero_flag(result as u8);
                self.set_negative_flag(result as u8);
                self.set_flag(CARRY, prev_a >= inc_result);
                self.set_overflow_flag(result, prev_a, inc_result, true);
            }

            JMP => {
                self.pc = self.apply_memory_mode(mode, operand, false);
            }

            JSR => {
                let old_pc = self.pc - 1;
                self.push_word(old_pc);
                let new_pc = self.apply_memory_mode(mode, operand, false);
                self.pc = new_pc;
                self.cycles_to_next += 1;
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
                let operand_value = self.read_memory_mode(mode, operand, false);
                let result = operand_value >> 1;
                self.write_memory_mode(mode, operand, result);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.set_flag(CARRY, operand_value & 0b1 > 0);
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
                self.cycles_to_next += 1;
            }

            PLP => {
                let value = self.pop();
                self.p = (value & 0b11101111) | 0b00100000;
                self.cycles_to_next += 1;
            }

            RLA => {
                let operand_value = self.read_memory_mode(mode, operand, false);
                let new_carry = operand_value & 0x80 > 0;
                let rol_result = (operand_value << 1) + if self.flag(CARRY) { 1 } else { 0 };
                self.write_memory_mode(mode, operand, rol_result);
                let result = self.a & rol_result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.set_flag(CARRY, new_carry);
                self.a = result;
            }

            ROL => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let new_carry = operand_value & 0x80 > 0;
                let result = (operand_value << 1) + if self.flag(CARRY) { 1 } else { 0 };
                self.write_memory_mode(mode, operand, result);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.set_flag(CARRY, new_carry);
            }

            ROR => {
                let operand_value = self.read_memory_mode(mode, operand, true);
                let new_carry = operand_value & 0x1 > 0;
                let result = (operand_value >> 1) + if self.flag(CARRY) { 0x80 } else { 0 };
                self.write_memory_mode(mode, operand, result);
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.set_flag(CARRY, new_carry);
            }

            RRA => {
                let operand_value = self.read_memory_mode(mode, operand, false);
                let ror_carry = (operand_value & 0x1) as u16;
                let ror_result = (operand_value >> 1) + if self.flag(CARRY) { 0x80 } else { 0 };
                self.write_memory_mode(mode, operand, ror_result);
                let prev_a = self.a;
                let result = u16::from(self.a) + u16::from(ror_result) + ror_carry;
                self.a = (result & 0xff) as u8;
                self.set_zero_flag(result as u8);
                self.set_negative_flag(result as u8);
                self.set_carry_flag(result);
                self.set_overflow_flag(result, prev_a, ror_result, false);
            }

            RTI => {
                let p = self.pop();
                self.p = p;
                let pc = self.pop_word();
                self.pc = pc;
                self.cycles_to_next += 1;
            }

            RTS => {
                let pc = self.pop_word();
                self.pc = pc + 1;
                self.cycles_to_next += 2;
            }

            SAX => {
                let value = self.a & self.x;
                self.write_memory_mode(mode, operand, value);
            }

            SBC => {
                let prev_a = self.a;
                let operand_value = self.read_memory_mode(mode, operand, true);
                let result = u16::from(self.a).wrapping_sub(u16::from(operand_value) + (if self.flag(CARRY) { 0 } else { 1 }));
                self.a = (result & 0xff) as u8;
                self.set_zero_flag(result as u8);
                self.set_negative_flag(result as u8);
                self.set_flag(CARRY, prev_a >= operand_value);
                self.set_overflow_flag(result, prev_a, operand_value, true);
            }

            SEC => {
                self.set_flag(CARRY, true);
            }

            SED => {
                self.set_flag(DECIMAL, true);
            }

            SEI => {
                self.set_flag(INTERRUPT, true);
            }

            SLO => {
                let operand_value = self.read_memory_mode(mode, operand, false);
                let asl_result = u16::from(operand_value) << 1;
                self.write_memory_mode(mode, operand, asl_result as u8);
                let result = self.a | asl_result as u8;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
                self.set_carry_flag(asl_result);
                self.a = result;
            }

            SRE => {
                let operand_value = self.read_memory_mode(mode, operand, false);
                let lsr_result = operand_value >> 1;
                self.write_memory_mode(mode, operand, lsr_result);
                self.set_flag(CARRY, operand_value & 0b1 > 0);
                let result = self.a ^ lsr_result;
                self.a = result;
                self.set_zero_flag(result);
                self.set_negative_flag(result);
            }

            STA => {
                let value = self.a;
                let cycles = self.cycles_to_next;
                self.write_memory_mode(mode, operand, value);
                match *mode {
                    IndirectIndexed => self.cycles_to_next = cycles + 4,
                    AbsoluteIndexedX | AbsoluteIndexedY => self.cycles_to_next = cycles + 2,
                    _ => ()
                }
            }

            STX => {
                let value = self.x;
                self.write_memory_mode(mode, operand, value);
            }

            STY => {
                let value = self.y;
                self.write_memory_mode(mode, operand, value);
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

            XXX => { unimplemented!("{:02X}", opcode_hex) }
        }

        match *opcode {
            ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP => {
                match *mode {
                    Accumulator | ZeropageX | IndirectIndexed => (),
                    AbsoluteIndexedX | AbsoluteIndexedY => self.cycles_to_next += 2,
                    IndexedIndirect => self.cycles_to_next -= 2,
                    _ => self.cycles_to_next += 1
                };
            }
            _ => ()
        }
    }

    pub fn tick(&mut self, instrument: bool, inputs: ControllerState) {
        if self.controller_strobe {
            self.last_inputs = inputs.to_u8();
        }
        if self.cycles_to_next == 0 {
            if let Some((addr, i)) = self.oam_dma_write {
                let data = self.read_memory(u16::from(addr) * 0x100 + u16::from(i));
                self.write_memory(0x2014, data);
                self.oam_dma_write = if i < 255 { Some((addr, i + 1)) } else { None };
            } else if self.ppu_bus.borrow().nmi_interrupt {
                let old_pc = self.pc;
                let p = self.p | 0b00100000;

                self.push_word(old_pc);
                self.push(p);
                self.pc = self.read_word(0xFFFA);
                self.ppu_bus.borrow_mut().nmi_interrupt = false;
            } else {
                self.execute_opcode(instrument);
            }
            self.cycles_to_next -= 1;
        } else {
            self.cycles_to_next -= 1;
        }
    }

    pub fn setup_for_test(&mut self, p_start: u8, pc_start: u16) {
        self.p = p_start;
        self.pc = pc_start;
    }

    pub fn pc_for_test(&self) -> u16 {
        self.pc
    }
}