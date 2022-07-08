#[derive(Debug)]
pub enum AddressingMode {
    DataRegister(u8),
    AddressRegister(u8),
    Address(u8),
    AddressWithPostincrement(u8),
    AddressWithPredecrement(u8),
    AddressWithDisplacement(u8),
    AddressWithIndex(u8),
    ProgramCounterWithDisplacement,
    ProgramCounterWithIndex,
    AbsoluteShort,
    AbsoluteLong,
    Immediate,
    Illegal,
}

impl AddressingMode {
    pub fn from_opcode(opcode: u16) -> AddressingMode {
        let address_register = (opcode & 0b111) as u8;
        let mode = (opcode >> 3) & 0b111;
        Self::from(mode, address_register)
    }

    pub fn from_opcode_dest(opcode: u16) -> AddressingMode {
        let address_register = ((opcode >> 9) & 0b111) as u8;
        let mode = (opcode >> 6) & 0b111;
        Self::from(mode, address_register)
    }

    fn from(mode: u16, address_register: u8) -> AddressingMode {
        match mode {
            0 => AddressingMode::DataRegister(address_register),
            1 => AddressingMode::AddressRegister(address_register),
            2 => AddressingMode::Address(address_register),
            3 => AddressingMode::AddressWithPostincrement(address_register),
            4 => AddressingMode::AddressWithPredecrement(address_register),
            5 => AddressingMode::AddressWithDisplacement(address_register),
            6 => AddressingMode::AddressWithIndex(address_register),
            7 => match address_register {
                0 => AddressingMode::AbsoluteShort,
                1 => AddressingMode::AbsoluteLong,
                2 => AddressingMode::ProgramCounterWithDisplacement,
                3 => AddressingMode::ProgramCounterWithIndex,
                4 => AddressingMode::Immediate,
                _ => AddressingMode::Illegal,
            }
            _ => AddressingMode::Illegal,
        }
    }
}

#[derive(Debug)]
pub enum OperandMode {
    RegisterToRegister,
    MemoryToMemory,
}

impl OperandMode {
    pub fn from_opcode(opcode: u16) -> OperandMode {
        if (opcode >> 3) & 0b1 == 0 {
            OperandMode::RegisterToRegister
        } else {
            OperandMode::MemoryToMemory
        }
    }
}

#[derive(Debug)]
pub enum ExchangeMode {
    DataRegisters,
    AddressRegisters,
    DataRegisterAndAddressRegister,
    Illegal,
}

impl ExchangeMode {
    pub fn from_opcode(opcode: u16) -> ExchangeMode {
        match (opcode >> 3) & 0b11111 {
            0b01000 => ExchangeMode::DataRegisters,
            0b01001 => ExchangeMode::AddressRegisters,
            0b10001 => ExchangeMode::DataRegisterAndAddressRegister,
            _ => ExchangeMode::Illegal,
        }
    }
}

#[derive(Debug)]
pub enum Condition {
    True,
    False,
    Higher,
    LowerOrSame,
    CarryClear,
    CarrySet,
    NotEqual,
    Equal,
    OverflowClear,
    OverflowSet,
    Plus,
    Minus,
    GreaterOrEqual,
    LessThan,
    GreaterThan,
    LessOrEqual,
    Illegal,
}

impl Condition {
    pub fn from_opcode(opcode: u16) -> Condition {
        match (opcode >> 8) & 0b1111 {
            0b0000 => Condition::True,
            0b0001 => Condition::False,
            0b0010 => Condition::Higher,
            0b0011 => Condition::LowerOrSame,
            0b0100 => Condition::CarryClear,
            0b0101 => Condition::CarrySet,
            0b0110 => Condition::NotEqual,
            0b0111 => Condition::Equal,
            0b1000 => Condition::OverflowClear,
            0b1001 => Condition::OverflowSet,
            0b1010 => Condition::Plus,
            0b1011 => Condition::Minus,
            0b1100 => Condition::GreaterOrEqual,
            0b1101 => Condition::LessThan,
            0b1110 => Condition::GreaterThan,
            0b1111 => Condition::LessOrEqual,
            _ => Condition::Illegal,
        }
    }
}

#[derive(Debug)]
pub enum Size {
    Byte,
    Word,
    Long,
    Illegal,
}

impl Size {
    pub fn from_opcode(opcode: u16) -> Size {
        match (opcode >> 6) & 0b11 {
            0 => Size::Byte,
            1 => Size::Word,
            2 => Size::Long,
            _ => Size::Illegal,
        }
    }

    pub fn from_move_opcode(opcode: u16) -> Size {
        match (opcode >> 12) & 0b11 {
            1 => Size::Byte,
            2 => Size::Long,
            3 => Size::Word,
            _ => Size::Illegal,
        }
    }

    pub fn from_opcode_bit(opcode: u16, bit: u16) -> Size {
        if (opcode >> bit) & 0b1 == 0 {
            Size::Word
        } else {
            Size::Long
        }
    }
}

#[derive(Debug)]
pub enum Direction {
    RegisterToMemory,
    MemoryToRegister,
}

#[derive(Debug)]
pub enum OperandDirection {
    ToRegister,
    ToMemory,
}

impl OperandDirection {
    pub fn from_opcode(opcode: u16) -> OperandDirection {
        if (opcode >> 8) & 0b1 == 0 {
            OperandDirection::ToRegister
        } else {
            OperandDirection::ToMemory
        }
    }
}

#[derive(Debug)]
pub enum BitNum {
    Immediate,
    DataRegister(u8),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Opcode {
    ABCD { operand_mode: OperandMode, src_register: u8, dest_register: u8 },
    // Add Decimal with Extend
    ADD { mode: AddressingMode, size: Size, operand_direction: OperandDirection, register: u8 },
    // Add
    ADDA { mode: AddressingMode, size: Size, register: u8 },
    // Add Address
    ADDI { mode: AddressingMode, size: Size },
    // Add Immediate
    ADDQ { mode: AddressingMode, size: Size, data: u8 },
    // Add Quick
    ADDX { operand_mode: OperandMode, size: Size, src_register: u8, dest_register: u8 },
    // Add with Extend
    AND { mode: AddressingMode, size: Size, operand_direction: OperandDirection, register: u8 },
    // Logical AND
    ANDI { mode: AddressingMode, size: Size },
    // Logical AND Immediate
    ASL { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Arithmetic Shift Left
    ASR { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Arithmetic Shift Right
    Bcc { condition: Condition, displacement: u8 },
    // Branch Conditionally
    BCHG { bit_num: BitNum, mode: AddressingMode },
    // Test Bit and Change
    BCLR { bit_num: BitNum, mode: AddressingMode },
    // Test Bit and Clear
    BRA { displacement: u8 },
    // Branch
    BSET { bit_num: BitNum, mode: AddressingMode },
    // Test Bit and Set
    BSR { displacement: u8 },
    // Branch to Subroutine
    BTST { bit_num: BitNum, mode: AddressingMode },
    // Test Bit
    CHK { register: u8, mode: AddressingMode },
    // Check Register Against Bound
    CLR { mode: AddressingMode, size: Size },
    // Clear
    CMP { mode: AddressingMode, size: Size, register: u8 },
    // Compare
    CMPA { mode: AddressingMode, size: Size, register: u8 },
    // Compare Address
    CMPI { mode: AddressingMode, size: Size },
    // Compare Immediate
    CMPM { size: Size, src_register: u8, dest_register: u8 },
    // Compare Memory to Memory
    DBcc { mode: AddressingMode, condition: Condition },
    // Test Condition, Decrement, and Branch
    DIVS { mode: AddressingMode, register: u8 },
    // Signed Divide
    DIVU { mode: AddressingMode, register: u8 },
    // Unsigned Divide
    EOR { size: Size, mode: AddressingMode, operand_direction: OperandDirection, register: u8 },
    // Logical Exclusive-OR
    EORI { mode: AddressingMode, size: Size },
    // Logical Exclusive-OR Immediate
    EXG { mode: ExchangeMode, src_register: u8, dest_register: u8 },
    // Exchange Registers
    EXT { mode: AddressingMode, size: Size },
    // Sign Extend
    ILLEGAL,
    // Take Illegal Instruction Trap
    JMP { mode: AddressingMode },
    // Jump
    JSR { mode: AddressingMode },
    // Jump to Subroutine
    LEA { register: u8, mode: AddressingMode },
    // Load Effective Address
    LINK { register: u8 },
    // Link and Allocate
    LSL { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Logical Shift Left
    LSR { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Logical Shift Right
    MOVE { src_mode: AddressingMode, dest_mode: AddressingMode, size: Size },
    // Move
    MOVEA { src_mode: AddressingMode, dest_mode: AddressingMode, size: Size },
    // Move Address
    MOVE_to_CCR { mode: AddressingMode },
    // Move to Condition Code Register
    MOVE_from_SR { mode: AddressingMode },
    // Move from Status Register
    MOVE_to_SR { mode: AddressingMode },
    // Move to Status Register
    MOVE_USP { register: u8, direction: Direction },
    // Move User Stack Pointer
    MOVEM { mode: AddressingMode, size: Size, direction: Direction },
    // Move Multiple Registers
    MOVEP { register: u8, direction: Direction, mode: AddressingMode, size: Size },
    // Move Peripheral
    MOVEQ { register: u8, data: u8 },
    // Move Quick
    MULS { mode: AddressingMode, register: u8 },
    // Signed Multiply
    MULU { mode: AddressingMode, register: u8 },
    // Unsigned Multiply
    NBCD { mode: AddressingMode },
    // Negate Decimal with Extend
    NEG { mode: AddressingMode, size: Size },
    // Negate
    NEGX { mode: AddressingMode, size: Size },
    // Negate with Extend
    NOP,
    // No Operation
    NOT { mode: AddressingMode, size: Size },
    // Logical Complement
    OR { size: Size, mode: AddressingMode, operand_direction: OperandDirection, register: u8 },
    // Logical Inclusive-OR
    ORI { mode: AddressingMode, size: Size },
    // Logical Inclusive-OR Immediate
    PEA { mode: AddressingMode },
    // Push Effective Address
    RESET,
    // Reset External Devices
    ROL { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Rotate Left
    ROR { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Rotate Right
    ROXL { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Rotate with Extend Left
    ROXR { mode: AddressingMode, size: Size, register: Option<u8>, shift_count: Option<u8>, shift_register: Option<u8> },
    // Rotate with Extend Right
    RTE,
    // Return from Exception
    RTR,
    // Return and Restore
    RTS,
    // Return from Subroutine
    SBCD { operand_mode: OperandMode, src_register: u8, dest_register: u8 },
    // Subtract Decimal with Extend
    Scc { mode: AddressingMode, condition: Condition },
    // Set Conditionally
    STOP,
    // Stop
    SUB { mode: AddressingMode, size: Size, operand_direction: OperandDirection, register: u8 },
    // Subtract
    SUBA { mode: AddressingMode, size: Size, register: u8 },
    // Subtract Address
    SUBI { mode: AddressingMode, size: Size },
    // Subtract Immediate
    SUBQ { mode: AddressingMode, size: Size, data: u8 },
    // Subtract Quick
    SUBX { operand_mode: OperandMode, size: Size, src_register: u8, dest_register: u8 },
    // Subtract with Extend
    SWAP { mode: AddressingMode },
    // Swap Register Words
    TAS { mode: AddressingMode },
    // Test Operand and Set
    TRAP { vector: u8 },
    // Trap
    TRAPV,
    // Trap on Overflow
    TST { mode: AddressingMode, size: Size },
    // Test Operand
    UNLK { register: u8 }, // Unlink
}

pub fn opcode(opcode_hex: u16) -> Opcode {
    use self::Opcode::*;
    let mode = AddressingMode::from_opcode(opcode_hex);
    let dest_mode = AddressingMode::from_opcode_dest(opcode_hex);
    let operand_mode = OperandMode::from_opcode(opcode_hex);
    let operand_direction = OperandDirection::from_opcode(opcode_hex);
    let size = Size::from_opcode(opcode_hex);
    let condition = Condition::from_opcode(opcode_hex);
    let register = ((opcode_hex >> 9) & 0b111) as u8;
    let src_register = (opcode_hex & 0b111) as u8;
    match opcode_hex >> 12 {
        0b0000 =>
            if (opcode_hex >> 8) & 0b1 > 0 {
                match &mode {
                    AddressingMode::AddressRegister(_) => MOVEP {
                        register,
                        direction: if (opcode_hex >> 7) & 0b1 > 0 {
                            Direction::RegisterToMemory
                        } else {
                            Direction::MemoryToRegister
                        },
                        size: Size::from_opcode_bit(opcode_hex, 6),
                        mode,
                    },
                    _ => match (opcode_hex >> 6) & 0b11 {
                        0 => BTST { bit_num: BitNum::DataRegister(register), mode },
                        1 => BCHG { bit_num: BitNum::DataRegister(register), mode },
                        2 => BCLR { bit_num: BitNum::DataRegister(register), mode },
                        3 => BSET { bit_num: BitNum::DataRegister(register), mode },
                        _ => ILLEGAL,
                    }
                }
            } else {
                match (opcode_hex >> 9) & 0b111 {
                    0 => ORI { mode, size },
                    1 => ANDI { mode, size },
                    2 => SUBI { mode, size },
                    3 => ADDI { mode, size },
                    4 => match (opcode_hex >> 6) & 0b11 {
                        0 => BTST { bit_num: BitNum::Immediate, mode },
                        1 => BCHG { bit_num: BitNum::Immediate, mode },
                        2 => BCLR { bit_num: BitNum::Immediate, mode },
                        3 => BSET { bit_num: BitNum::Immediate, mode },
                        _ => ILLEGAL,
                    },
                    5 => EORI { mode, size },
                    6 => CMPI { mode, size },
                    _ => ILLEGAL,
                }
            },
        0b0001 | 0b0010 | 0b0011 => {
            let size = Size::from_move_opcode(opcode_hex);
            match mode {
                AddressingMode::AddressRegister(_) => MOVEA { src_mode: mode, dest_mode, size },
                _ => MOVE { src_mode: mode, dest_mode, size },
            }
        }
        0b0100 =>
            if (opcode_hex >> 11) & 0b1 == 0 {
                match size {
                    Size::Illegal => match (opcode_hex >> 8) & 0b1111 {
                        0b0000 => MOVE_from_SR { mode },
                        0b0100 => MOVE_to_CCR { mode },
                        0b0110 => MOVE_to_SR { mode },
                        _ => ILLEGAL,
                    }
                    _ => match (opcode_hex >> 8) & 0b1111 {
                        0b0000 => NEGX { mode, size },
                        0b0010 => CLR { mode, size },
                        0b0100 => NEG { mode, size },
                        0b0110 => NOT { mode, size },
                        _ => ILLEGAL,
                    }
                }
            } else {
                if (opcode_hex >> 8) & 0b1 == 1 {
                    if (opcode_hex >> 6) & 0b1 == 0 {
                        CHK { register, mode }
                    } else {
                        LEA { register, mode }
                    }
                } else {
                    match (opcode_hex >> 8) & 0b111 {
                        0b000 =>
                            match (opcode_hex >> 6) & 0b11 {
                                0b00 => NBCD { mode },
                                0b01 => match mode {
                                    AddressingMode::DataRegister(_) => SWAP { mode },
                                    _ => PEA { mode },
                                }
                                _ => match mode {
                                    AddressingMode::DataRegister(_) => EXT {
                                        mode,
                                        size: Size::from_opcode_bit(opcode_hex, 6),
                                    },
                                    _ => MOVEM {
                                        mode,
                                        size: Size::from_opcode_bit(opcode_hex, 6),
                                        direction: Direction::RegisterToMemory,
                                    },
                                },
                            },
                        0b010 => match size {
                            Size::Illegal => match mode {
                                AddressingMode::Immediate => ILLEGAL,
                                _ => TAS { mode },
                            },
                            _ => TST { mode, size },
                        },
                        0b100 => MOVEM {
                            mode,
                            size: Size::from_opcode_bit(opcode_hex, 6),
                            direction: Direction::MemoryToRegister,
                        },
                        0b110 => if (opcode_hex >> 7) & 1 == 0 {
                            match (opcode_hex >> 4) & 0b111 {
                                0b100 => TRAP { vector: (opcode_hex & 0b1111) as u8 },
                                0b101 => if (opcode_hex >> 3) & 0b1 == 0 {
                                    LINK { register: (opcode_hex & 0b111) as u8 }
                                } else {
                                    UNLK { register: (opcode_hex & 0b111) as u8 }
                                },
                                0b110 => MOVE_USP {
                                    register: (opcode_hex & 0b111) as u8,
                                    direction: if (opcode_hex >> 4) & 0b1 == 0 {
                                        Direction::RegisterToMemory
                                    } else {
                                        Direction::MemoryToRegister
                                    },
                                },
                                0b111 => match opcode_hex & 0b111 {
                                    0b000 => RESET,
                                    0b001 => NOP,
                                    0b010 => STOP,
                                    0b011 => RTE,
                                    0b101 => RTS,
                                    0b110 => TRAPV,
                                    0b111 => RTR,
                                    _ => ILLEGAL,
                                },
                                _ => ILLEGAL,
                            }
                        } else {
                            if (opcode_hex >> 6) & 0b1 == 0 {
                                JSR { mode }
                            } else {
                                JMP { mode }
                            }
                        },
                        _ => ILLEGAL,
                    }
                }
            }
        0b0101 => match size {
            Size::Illegal => match mode {
                AddressingMode::DataRegister(_) => DBcc { mode, condition },
                _ => Scc { mode, condition },
            }
            _ => if (opcode_hex >> 8) & 0b1 == 0 {
                ADDQ { mode, size, data: register }
            } else {
                SUBQ { mode, size, data: register }
            }
        }
        0b0110 => match condition {
            Condition::True => BRA { displacement: (opcode_hex & 0b11111111) as u8 },
            Condition::False => BSR { displacement: (opcode_hex & 0b11111111) as u8 },
            _ => Bcc { condition, displacement: (opcode_hex & 0b11111111) as u8 },
        }
        0b0111 => MOVEQ { register, data: (opcode_hex & 0b11111111) as u8 },
        0b1000 => if (opcode_hex >> 1) & 0b10000 == 1 {
            SBCD { operand_mode, src_register, dest_register: register }
        } else {
            match size {
                Size::Illegal => if (opcode_hex >> 8) & 0b1 == 0 {
                    DIVU { mode, register }
                } else {
                    DIVS { mode, register }
                }
                _ => OR { size, mode, operand_direction, register }
            }
        }
        0b1001 => match size {
            Size::Illegal => SUBA {
                mode,
                register,
                size: Size::from_opcode_bit(opcode_hex, 8),
            },
            _ => match operand_direction {
                OperandDirection::ToMemory => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => SUBX {
                        size,
                        operand_mode,
                        src_register,
                        dest_register: register,
                    },
                    _ => SUB { size, mode, register, operand_direction },
                }
                _ => SUB { size, mode, register, operand_direction },
            }
        }
        0b1011 => match size {
            Size::Illegal => CMPA {
                mode,
                size: Size::from_opcode_bit(opcode_hex, 8),
                register,
            },
            _ => if (opcode_hex >> 8) & 0b1 == 0 {
                CMP { mode, size, register }
            } else {
                match mode {
                    AddressingMode::AddressRegister(_) => CMPM {
                        size,
                        src_register,
                        dest_register: register,
                    },
                    _ => EOR { mode, size, operand_direction, register },
                }
            }
        }
        0b1100 => if (opcode_hex >> 1) & 0b10000 == 1 {
            ABCD { operand_mode, src_register, dest_register: register }
        } else {
            match size {
                Size::Illegal => if (opcode_hex >> 8) & 0b1 == 0 {
                    MULU { mode, register }
                } else {
                    MULS { mode, register }
                }
                _ => match operand_direction {
                    OperandDirection::ToMemory => match mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => EXG {
                            mode: ExchangeMode::from_opcode(opcode_hex),
                            src_register,
                            dest_register: register,
                        },
                        _ => AND { size, mode, register, operand_direction },
                    }
                    _ => AND { size, mode, register, operand_direction },
                }
            }
        }
        0b1101 => match size {
            Size::Illegal => ADDA {
                mode,
                register,
                size: Size::from_opcode_bit(opcode_hex, 8),
            },
            _ => match operand_direction {
                OperandDirection::ToMemory => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => ADDX {
                        size,
                        operand_mode,
                        src_register,
                        dest_register: register,
                    },
                    _ => ADD { size, mode, register, operand_direction },
                }
                _ => ADD { size, mode, register, operand_direction },
            }
        }
        0b1110 => {
            let direction = (opcode_hex >> 8) & 0b1;
            match size {
                Size::Illegal => match (opcode_hex >> 9) & 0b111 {
                    0b000 => if direction == 0 {
                        ASL { mode, size, register: None, shift_register: None, shift_count: None }
                    } else {
                        ASR { mode, size, register: None, shift_register: None, shift_count: None }
                    },
                    0b001 => if direction == 0 {
                        LSL { mode, size, register: None, shift_register: None, shift_count: None }
                    } else {
                        LSR { mode, size, register: None, shift_register: None, shift_count: None }
                    },
                    0b010 => if direction == 0 {
                        ROXL { mode, size, register: None, shift_register: None, shift_count: None }
                    } else {
                        ROXR { mode, size, register: None, shift_register: None, shift_count: None }
                    },
                    0b011 => if direction == 0 {
                        ROL { mode, size, register: None, shift_register: None, shift_count: None }
                    } else {
                        ROR { mode, size, register: None, shift_register: None, shift_count: None }
                    },
                    _ => ILLEGAL,
                },
                _ => {
                    let shift_count = if (opcode_hex >> 5) & 0b1 == 0 {
                        Some(((opcode_hex >> 9) & 0b111) as u8)
                    } else {
                        None
                    };
                    let shift_register = if (opcode_hex >> 5) & 0b1 == 1 {
                        Some(((opcode_hex >> 9) & 0b111) as u8)
                    } else {
                        None
                    };
                    match (opcode_hex >> 1) & 0b11 {
                        0b00 => if direction == 0 {
                            ASL {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        } else {
                            ASR {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        },
                        0b01 => if direction == 0 {
                            LSL {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        } else {
                            LSR {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        },
                        0b10 => if direction == 0 {
                            ROXL {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        } else {
                            ROXR {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        },
                        0b11 => if direction == 0 {
                            ROL {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        } else {
                            ROR {
                                mode: AddressingMode::Illegal,
                                size,
                                register: Some(src_register),
                                shift_register,
                                shift_count,
                            }
                        },
                        _ => ILLEGAL,
                    }
                }
            }
        }
        _ => ILLEGAL,
    }
}