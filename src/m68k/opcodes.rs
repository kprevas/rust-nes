use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    DataRegister(usize),
    AddressRegister(usize),
    Address(usize),
    AddressWithPostincrement(usize),
    AddressWithPredecrement(usize),
    AddressWithDisplacement(usize),
    AddressWithIndex(usize),
    ProgramCounterWithDisplacement,
    ProgramCounterWithIndex,
    AbsoluteShort,
    AbsoluteLong,
    Immediate,
    Illegal,
}

impl AddressingMode {
    pub fn from_opcode(opcode: u16) -> AddressingMode {
        let address_register = (opcode & 0b111) as usize;
        let mode = (opcode >> 3) & 0b111;
        Self::from(mode, address_register)
    }

    pub fn from_opcode_dest(opcode: u16) -> AddressingMode {
        let address_register = ((opcode >> 9) & 0b111) as usize;
        let mode = (opcode >> 6) & 0b111;
        Self::from(mode, address_register)
    }

    fn from(mode: u16, address_register: usize) -> AddressingMode {
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
            },
            _ => AddressingMode::Illegal,
        }
    }

    pub fn is_data_alterable(self) -> bool {
        match self {
            AddressingMode::AddressRegister(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::Immediate
            | AddressingMode::Illegal => false,
            _ => true,
        }
    }

    pub fn is_memory_alterable(self) -> bool {
        match self {
            AddressingMode::DataRegister(_)
            | AddressingMode::AddressRegister(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::Immediate
            | AddressingMode::Illegal => false,
            _ => true,
        }
    }

    pub fn is_control_addressing(self) -> bool {
        match self {
            AddressingMode::DataRegister(_)
            | AddressingMode::AddressRegister(_)
            | AddressingMode::AddressWithPostincrement(_)
            | AddressingMode::AddressWithPredecrement(_)
            | AddressingMode::Immediate
            | AddressingMode::Illegal => false,
            _ => true,
        }
    }
}

impl Display for AddressingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::DataRegister(register) => f.write_fmt(format_args!("D{}", register)),
            AddressingMode::AddressRegister(register) => f.write_fmt(format_args!("A{}", register)),
            AddressingMode::Address(register) => f.write_fmt(format_args!("(A{})", register)),
            AddressingMode::AddressWithPostincrement(register) => {
                f.write_fmt(format_args!("(A{})+", register))
            }
            AddressingMode::AddressWithPredecrement(register) => {
                f.write_fmt(format_args!("-(A{})", register))
            }
            AddressingMode::AddressWithDisplacement(register) => {
                f.write_fmt(format_args!("(d16, A{})", register))
            }
            AddressingMode::AddressWithIndex(register) => {
                f.write_fmt(format_args!("(d8, A{}, Xn)", register))
            }
            AddressingMode::ProgramCounterWithDisplacement => f.write_str("(d16, PC)"),
            AddressingMode::ProgramCounterWithIndex => f.write_str("(d8, PC, Xn)"),
            AddressingMode::AbsoluteShort => f.write_str("(xxx).w"),
            AddressingMode::AbsoluteLong => f.write_str("(xxx).l"),
            AddressingMode::Immediate => f.write_str("#"),
            AddressingMode::Illegal => f.write_str("XXX"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::True => f.write_str("T"),
            Condition::False => f.write_str("F"),
            Condition::Higher => f.write_str("HI"),
            Condition::LowerOrSame => f.write_str("LS"),
            Condition::CarryClear => f.write_str("CC"),
            Condition::CarrySet => f.write_str("CS"),
            Condition::NotEqual => f.write_str("NE"),
            Condition::Equal => f.write_str("EQ"),
            Condition::OverflowClear => f.write_str("VC"),
            Condition::OverflowSet => f.write_str("VS"),
            Condition::Plus => f.write_str("PL"),
            Condition::Minus => f.write_str("MI"),
            Condition::GreaterOrEqual => f.write_str("GE"),
            Condition::LessThan => f.write_str("LT"),
            Condition::GreaterThan => f.write_str("GT"),
            Condition::LessOrEqual => f.write_str("LE"),
            Condition::Illegal => f.write_str("XX"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
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

impl Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Byte => f.write_str(".b"),
            Size::Word => f.write_str(".w"),
            Size::Long => f.write_str(".l"),
            Size::Illegal => f.write_str(".X"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    RegisterToMemory,
    MemoryToRegister,
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub enum BitNum {
    Immediate,
    DataRegister(usize),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum Opcode {
    ABCD {
        operand_mode: OperandMode,
        src_register: usize,
        dest_register: usize,
    },
    // Add Decimal with Extend
    ADD {
        mode: AddressingMode,
        size: Size,
        operand_direction: OperandDirection,
        register: usize,
    },
    // Add
    ADDA {
        mode: AddressingMode,
        size: Size,
        register: usize,
    },
    // Add Address
    ADDI {
        mode: AddressingMode,
        size: Size,
    },
    // Add Immediate
    ADDQ {
        mode: AddressingMode,
        size: Size,
        data: u8,
    },
    // Add Quick
    ADDX {
        operand_mode: OperandMode,
        size: Size,
        src_register: usize,
        dest_register: usize,
    },
    // Add with Extend
    AND {
        mode: AddressingMode,
        size: Size,
        operand_direction: OperandDirection,
        register: usize,
    },
    // Logical AND
    ANDI {
        mode: AddressingMode,
        size: Size,
    },
    // Logical AND Immediate
    ANDI_to_CCR,
    // Logical AND to Condition Code Register
    ANDI_to_SR,
    // Logical AND to Status Register
    ASL {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Arithmetic Shift Left
    ASR {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Arithmetic Shift Right
    Bcc {
        condition: Condition,
        displacement: i8,
    },
    // Branch Conditionally
    BCHG {
        bit_num: BitNum,
        mode: AddressingMode,
    },
    // Test Bit and Change
    BCLR {
        bit_num: BitNum,
        mode: AddressingMode,
    },
    // Test Bit and Clear
    BRA {
        displacement: i8,
    },
    // Branch
    BSET {
        bit_num: BitNum,
        mode: AddressingMode,
    },
    // Test Bit and Set
    BSR {
        displacement: i8,
    },
    // Branch to Subroutine
    BTST {
        bit_num: BitNum,
        mode: AddressingMode,
    },
    // Test Bit
    CHK {
        register: usize,
        mode: AddressingMode,
    },
    // Check Register Against Bound
    CLR {
        mode: AddressingMode,
        size: Size,
    },
    // Clear
    CMP {
        mode: AddressingMode,
        size: Size,
        register: usize,
    },
    // Compare
    CMPA {
        mode: AddressingMode,
        size: Size,
        register: usize,
    },
    // Compare Address
    CMPI {
        mode: AddressingMode,
        size: Size,
    },
    // Compare Immediate
    CMPM {
        size: Size,
        src_register: usize,
        dest_register: usize,
    },
    // Compare Memory to Memory
    DBcc {
        condition: Condition,
        register: usize,
    },
    // Test Condition, Decrement, and Branch
    DIVS {
        mode: AddressingMode,
        register: usize,
    },
    // Signed Divide
    DIVU {
        mode: AddressingMode,
        register: usize,
    },
    // Unsigned Divide
    EOR {
        size: Size,
        mode: AddressingMode,
        operand_direction: OperandDirection,
        register: usize,
    },
    // Logical Exclusive-OR
    EORI {
        mode: AddressingMode,
        size: Size,
    },
    // Logical Exclusive-OR Immediate
    EORI_to_CCR,
    // Logical Exclusive-OR to Condition Code Register
    EORI_to_SR,
    // Logical Exclusive-OR to Status Register
    EXG {
        mode: ExchangeMode,
        src_register: usize,
        dest_register: usize,
    },
    // Exchange Registers
    EXT {
        mode: AddressingMode,
        size: Size,
    },
    // Sign Extend
    ILLEGAL,
    // Take Illegal Instruction Trap
    JMP {
        mode: AddressingMode,
    },
    // Jump
    JSR {
        mode: AddressingMode,
    },
    // Jump to Subroutine
    LEA {
        register: usize,
        mode: AddressingMode,
    },
    // Load Effective Address
    LINK {
        register: usize,
    },
    // Link and Allocate
    LSL {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Logical Shift Left
    LSR {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Logical Shift Right
    MOVE {
        src_mode: AddressingMode,
        dest_mode: AddressingMode,
        size: Size,
    },
    // Move
    MOVEA {
        src_mode: AddressingMode,
        dest_mode: AddressingMode,
        size: Size,
    },
    // Move Address
    MOVE_to_CCR {
        mode: AddressingMode,
    },
    // Move to Condition Code Register
    MOVE_from_SR {
        mode: AddressingMode,
    },
    // Move from Status Register
    MOVE_to_SR {
        mode: AddressingMode,
    },
    // Move to Status Register
    MOVE_USP {
        register: usize,
        direction: Direction,
    },
    // Move User Stack Pointer
    MOVEM {
        mode: AddressingMode,
        size: Size,
        direction: Direction,
    },
    // Move Multiple Registers
    MOVEP {
        data_register: usize,
        address_register: usize,
        direction: Direction,
        size: Size,
    },
    // Move Peripheral
    MOVEQ {
        register: usize,
        data: i8,
    },
    // Move Quick
    MULS {
        mode: AddressingMode,
        register: usize,
    },
    // Signed Multiply
    MULU {
        mode: AddressingMode,
        register: usize,
    },
    // Unsigned Multiply
    NBCD {
        mode: AddressingMode,
    },
    // Negate Decimal with Extend
    NEG {
        mode: AddressingMode,
        size: Size,
    },
    // Negate
    NEGX {
        mode: AddressingMode,
        size: Size,
    },
    // Negate with Extend
    NOP,
    // No Operation
    NOT {
        mode: AddressingMode,
        size: Size,
    },
    // Logical Complement
    OR {
        size: Size,
        mode: AddressingMode,
        operand_direction: OperandDirection,
        register: usize,
    },
    // Logical Inclusive-OR
    ORI {
        mode: AddressingMode,
        size: Size,
    },
    // Logical Inclusive-OR Immediate
    ORI_to_CCR,
    // Logical Inclusive-OR to Condition Code Register
    ORI_to_SR,
    // Logical Inclusive-OR to Status Register
    PEA {
        mode: AddressingMode,
    },
    // Push Effective Address
    RESET,
    // Reset External Devices
    ROL {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Rotate Left
    ROR {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Rotate Right
    ROXL {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Rotate with Extend Left
    ROXR {
        mode: AddressingMode,
        size: Size,
        register: Option<usize>,
        shift_count: Option<u8>,
        shift_register: Option<usize>,
    },
    // Rotate with Extend Right
    RTE,
    // Return from Exception
    RTR,
    // Return and Restore
    RTS,
    // Return from Subroutine
    SBCD {
        operand_mode: OperandMode,
        src_register: usize,
        dest_register: usize,
    },
    // Subtract Decimal with Extend
    Scc {
        mode: AddressingMode,
        condition: Condition,
    },
    // Set Conditionally
    STOP,
    // Stop
    SUB {
        mode: AddressingMode,
        size: Size,
        operand_direction: OperandDirection,
        register: usize,
    },
    // Subtract
    SUBA {
        mode: AddressingMode,
        size: Size,
        register: usize,
    },
    // Subtract Address
    SUBI {
        mode: AddressingMode,
        size: Size,
    },
    // Subtract Immediate
    SUBQ {
        mode: AddressingMode,
        size: Size,
        data: u8,
    },
    // Subtract Quick
    SUBX {
        operand_mode: OperandMode,
        size: Size,
        src_register: usize,
        dest_register: usize,
    },
    // Subtract with Extend
    SWAP {
        mode: AddressingMode,
    },
    // Swap Register Words
    TAS {
        mode: AddressingMode,
    },
    // Test Operand and Set
    TRAP {
        vector: u8,
    },
    // Trap
    TRAPV,
    // Trap on Overflow
    TST {
        mode: AddressingMode,
        size: Size,
    },
    // Test Operand
    UNLK {
        register: usize,
    }, // Unlink
}

pub fn opcode(opcode_hex: u16) -> Opcode {
    use self::Opcode::*;
    let mode = AddressingMode::from_opcode(opcode_hex);
    let dest_mode = AddressingMode::from_opcode_dest(opcode_hex);
    let operand_mode = OperandMode::from_opcode(opcode_hex);
    let operand_direction = OperandDirection::from_opcode(opcode_hex);
    let size = Size::from_opcode(opcode_hex);
    let condition = Condition::from_opcode(opcode_hex);
    let register = ((opcode_hex >> 9) & 0b111) as usize;
    let src_register = (opcode_hex & 0b111) as usize;
    match opcode_hex >> 12 {
        0b0000 => {
            if (opcode_hex >> 8) & 0b1 == 1 {
                match &mode {
                    AddressingMode::AddressRegister(address_register) => MOVEP {
                        data_register: register,
                        address_register: *address_register,
                        direction: if (opcode_hex >> 7) & 0b1 > 0 {
                            Direction::RegisterToMemory
                        } else {
                            Direction::MemoryToRegister
                        },
                        size: Size::from_opcode_bit(opcode_hex, 6),
                    },
                    _ => match (opcode_hex >> 6) & 0b11 {
                        0 => match mode {
                            AddressingMode::AddressRegister(_) | AddressingMode::Illegal => ILLEGAL,
                            _ => BTST {
                                bit_num: BitNum::DataRegister(register),
                                mode,
                            },
                        },
                        1 if mode.is_data_alterable() => BCHG {
                            bit_num: BitNum::DataRegister(register),
                            mode,
                        },
                        2 if mode.is_data_alterable() => BCLR {
                            bit_num: BitNum::DataRegister(register),
                            mode,
                        },
                        3 if mode.is_data_alterable() => BSET {
                            bit_num: BitNum::DataRegister(register),
                            mode,
                        },
                        _ => ILLEGAL,
                    },
                }
            } else {
                match (opcode_hex >> 9) & 0b111 {
                    0b000 => match (size, mode) {
                        (Size::Illegal, _)
                        | (_, AddressingMode::Illegal)
                        | (_, AddressingMode::AddressRegister(_))
                        | (_, AddressingMode::ProgramCounterWithIndex)
                        | (_, AddressingMode::ProgramCounterWithDisplacement)
                        | (Size::Long, AddressingMode::Immediate) => ILLEGAL,
                        (Size::Byte, AddressingMode::Immediate) => ORI_to_CCR,
                        (Size::Word, AddressingMode::Immediate) => ORI_to_SR,
                        _ => ORI { mode, size },
                    },
                    0b001 => match (size, mode) {
                        (Size::Illegal, _)
                        | (_, AddressingMode::Illegal)
                        | (_, AddressingMode::AddressRegister(_))
                        | (_, AddressingMode::ProgramCounterWithIndex)
                        | (_, AddressingMode::ProgramCounterWithDisplacement)
                        | (Size::Long, AddressingMode::Immediate) => ILLEGAL,
                        (Size::Byte, AddressingMode::Immediate) => ANDI_to_CCR,
                        (Size::Word, AddressingMode::Immediate) => ANDI_to_SR,
                        _ => ANDI { mode, size },
                    },
                    0b010 if mode.is_data_alterable() => match size {
                        Size::Illegal => ILLEGAL,
                        _ => SUBI { mode, size },
                    },
                    0b011 if mode.is_data_alterable() => match size {
                        Size::Illegal => ILLEGAL,
                        _ => ADDI { mode, size },
                    },
                    0b100 => match (opcode_hex >> 6) & 0b11 {
                        0 => match mode {
                            AddressingMode::AddressRegister(_)
                            | AddressingMode::Immediate
                            | AddressingMode::Illegal => ILLEGAL,
                            _ => BTST {
                                bit_num: BitNum::Immediate,
                                mode,
                            },
                        },
                        1 if mode.is_data_alterable() => BCHG {
                            bit_num: BitNum::Immediate,
                            mode,
                        },
                        2 if mode.is_data_alterable() => BCLR {
                            bit_num: BitNum::Immediate,
                            mode,
                        },
                        3 if mode.is_data_alterable() => BSET {
                            bit_num: BitNum::Immediate,
                            mode,
                        },
                        _ => ILLEGAL,
                    },
                    0b101 => match (size, mode) {
                        (Size::Illegal, _)
                        | (_, AddressingMode::Illegal)
                        | (_, AddressingMode::AddressRegister(_))
                        | (_, AddressingMode::ProgramCounterWithIndex)
                        | (_, AddressingMode::ProgramCounterWithDisplacement)
                        | (Size::Long, AddressingMode::Immediate) => ILLEGAL,
                        (Size::Byte, AddressingMode::Immediate) => EORI_to_CCR,
                        (Size::Word, AddressingMode::Immediate) => EORI_to_SR,
                        _ => EORI { mode, size },
                    },
                    0b110 if mode.is_data_alterable() => match size {
                        Size::Illegal => ILLEGAL,
                        _ => CMPI { mode, size },
                    },
                    _ => ILLEGAL,
                }
            }
        }
        0b0001 | 0b0010 | 0b0011 => {
            let size = Size::from_move_opcode(opcode_hex);
            match (mode, dest_mode, size) {
                (AddressingMode::Illegal, _, _)
                | (_, AddressingMode::Illegal, _)
                | (AddressingMode::AddressRegister(_), _, Size::Byte)
                | (_, AddressingMode::AddressRegister(_), Size::Byte) => ILLEGAL,
                (_, AddressingMode::AddressRegister(_), _) => MOVEA {
                    src_mode: mode,
                    dest_mode,
                    size,
                },
                _ if dest_mode.is_data_alterable() => MOVE {
                    src_mode: mode,
                    dest_mode,
                    size,
                },
                _ => ILLEGAL,
            }
        }
        0b0100 => {
            if (opcode_hex >> 7) & 0b11 == 0b11 {
                match (opcode_hex >> 6) & 0b1 {
                    0b0 => match mode {
                        AddressingMode::AddressRegister(_) | AddressingMode::Illegal => ILLEGAL,
                        _ => CHK { register, mode },
                    },
                    0b1 if mode.is_control_addressing() => LEA { register, mode },
                    _ => ILLEGAL,
                }
            } else if (opcode_hex >> 11) & 0b1 == 0 {
                match size {
                    Size::Illegal => match (opcode_hex >> 8) & 0b1111 {
                        0b0000 => {
                            if mode.is_data_alterable() {
                                MOVE_from_SR { mode }
                            } else {
                                ILLEGAL
                            }
                        }
                        0b0100 => match mode {
                            AddressingMode::AddressRegister(_) | AddressingMode::Illegal => ILLEGAL,
                            _ => MOVE_to_CCR { mode },
                        },
                        0b0110 => match mode {
                            AddressingMode::AddressRegister(_) | AddressingMode::Illegal => ILLEGAL,
                            _ => MOVE_to_SR { mode },
                        },
                        _ => ILLEGAL,
                    },
                    _ => match (opcode_hex >> 8) & 0b1111 {
                        _ if !mode.is_data_alterable() => ILLEGAL,
                        0b0000 => NEGX { mode, size },
                        0b0010 => CLR { mode, size },
                        0b0100 => NEG { mode, size },
                        0b0110 => NOT { mode, size },
                        _ => ILLEGAL,
                    },
                }
            } else {
                if (opcode_hex >> 8) & 0b1 == 1 {
                    ILLEGAL
                } else {
                    match (opcode_hex >> 8) & 0b111 {
                        0b000 => match (opcode_hex >> 6) & 0b11 {
                            0b00 => {
                                if mode.is_data_alterable() {
                                    NBCD { mode }
                                } else {
                                    ILLEGAL
                                }
                            }
                            0b01 => match mode {
                                AddressingMode::DataRegister(_) => SWAP { mode },
                                _ if mode.is_control_addressing() => PEA { mode },
                                _ => ILLEGAL,
                            },
                            _ => match mode {
                                AddressingMode::DataRegister(_) => EXT {
                                    mode,
                                    size: Size::from_opcode_bit(opcode_hex, 6),
                                },
                                AddressingMode::Address(_)
                                | AddressingMode::AddressWithPredecrement(_)
                                | AddressingMode::AddressWithIndex(_)
                                | AddressingMode::AddressWithDisplacement(_)
                                | AddressingMode::AbsoluteLong
                                | AddressingMode::AbsoluteShort => MOVEM {
                                    mode,
                                    size: Size::from_opcode_bit(opcode_hex, 6),
                                    direction: Direction::RegisterToMemory,
                                },
                                _ => ILLEGAL,
                            },
                        },
                        0b010 => {
                            if mode.is_data_alterable() {
                                match size {
                                    Size::Illegal => TAS { mode },
                                    _ => TST { mode, size },
                                }
                            } else {
                                ILLEGAL
                            }
                        }
                        0b100 => {
                            if (opcode_hex >> 7) & 0b1 == 1 {
                                match mode {
                                    AddressingMode::Address(_)
                                    | AddressingMode::AddressWithPostincrement(_)
                                    | AddressingMode::AddressWithIndex(_)
                                    | AddressingMode::AddressWithDisplacement(_)
                                    | AddressingMode::AbsoluteLong
                                    | AddressingMode::AbsoluteShort
                                    | AddressingMode::ProgramCounterWithIndex
                                    | AddressingMode::ProgramCounterWithDisplacement => MOVEM {
                                        mode,
                                        size: Size::from_opcode_bit(opcode_hex, 6),
                                        direction: Direction::MemoryToRegister,
                                    },
                                    _ => ILLEGAL,
                                }
                            } else {
                                ILLEGAL
                            }
                        }
                        0b110 => {
                            if (opcode_hex >> 7) & 1 == 0 {
                                match (opcode_hex >> 4) & 0b111 {
                                    0b100 => TRAP {
                                        vector: (opcode_hex & 0b1111) as u8,
                                    },
                                    0b101 => {
                                        if (opcode_hex >> 3) & 0b1 == 0 {
                                            LINK {
                                                register: (opcode_hex & 0b111) as usize,
                                            }
                                        } else {
                                            UNLK {
                                                register: (opcode_hex & 0b111) as usize,
                                            }
                                        }
                                    }
                                    0b110 => MOVE_USP {
                                        register: (opcode_hex & 0b111) as usize,
                                        direction: if (opcode_hex >> 3) & 0b1 == 0 {
                                            Direction::RegisterToMemory
                                        } else {
                                            Direction::MemoryToRegister
                                        },
                                    },
                                    0b111 => match opcode_hex & 0b1111 {
                                        0b0000 => RESET,
                                        0b0001 => NOP,
                                        0b0010 => STOP,
                                        0b0011 => RTE,
                                        0b0101 => RTS,
                                        0b0110 => TRAPV,
                                        0b0111 => RTR,
                                        _ => ILLEGAL,
                                    },
                                    _ => ILLEGAL,
                                }
                            } else {
                                if mode.is_control_addressing() {
                                    if (opcode_hex >> 6) & 0b1 == 0 {
                                        JSR { mode }
                                    } else {
                                        JMP { mode }
                                    }
                                } else {
                                    ILLEGAL
                                }
                            }
                        }
                        _ => ILLEGAL,
                    }
                }
            }
        }
        0b0101 => match size {
            Size::Illegal => match mode {
                AddressingMode::AddressRegister(register) => DBcc {
                    condition,
                    register,
                },
                _ if mode.is_data_alterable() => Scc { mode, condition },
                _ => ILLEGAL,
            },
            _ => match (mode, size) {
                (AddressingMode::Immediate, _)
                | (AddressingMode::ProgramCounterWithDisplacement, _)
                | (AddressingMode::ProgramCounterWithIndex, _)
                | (AddressingMode::Illegal, _)
                | (AddressingMode::AddressRegister(_), Size::Byte) => ILLEGAL,
                _ => {
                    let data = if register == 0 { 8 } else { register as u8 };
                    if (opcode_hex >> 8) & 0b1 == 0 {
                        ADDQ { mode, size, data }
                    } else {
                        SUBQ { mode, size, data }
                    }
                }
            },
        },
        0b0110 => match condition {
            Condition::True => BRA {
                displacement: (opcode_hex & 0b11111111) as i8,
            },
            Condition::False => BSR {
                displacement: (opcode_hex & 0b11111111) as i8,
            },
            _ => Bcc {
                condition,
                displacement: (opcode_hex & 0b11111111) as i8,
            },
        },
        0b0111 => {
            if (opcode_hex >> 8) & 0b1 == 0 {
                MOVEQ {
                    register,
                    data: (opcode_hex & 0b11111111) as i8,
                }
            } else {
                ILLEGAL
            }
        }
        0b1000 => {
            if (opcode_hex >> 4) & 0b11111 == 0b10000 {
                SBCD {
                    operand_mode,
                    src_register,
                    dest_register: register,
                }
            } else {
                match (mode, size) {
                    (AddressingMode::Illegal, _) | (AddressingMode::AddressRegister(_), _) => {
                        ILLEGAL
                    }
                    (_, Size::Illegal) => {
                        if (opcode_hex >> 8) & 0b1 == 0 {
                            DIVU { mode, register }
                        } else {
                            DIVS { mode, register }
                        }
                    }
                    _ => match operand_direction {
                        OperandDirection::ToMemory if !mode.is_memory_alterable() => ILLEGAL,
                        _ => OR {
                            size,
                            mode,
                            operand_direction,
                            register,
                        },
                    },
                }
            }
        }
        0b1001 => match size {
            Size::Illegal => match mode {
                AddressingMode::Illegal => ILLEGAL,
                _ => SUBA {
                    mode,
                    register,
                    size: Size::from_opcode_bit(opcode_hex, 8),
                },
            },
            _ => match operand_direction {
                OperandDirection::ToMemory => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => SUBX {
                        size,
                        operand_mode,
                        src_register,
                        dest_register: register,
                    },
                    _ if mode.is_memory_alterable() => SUB {
                        size,
                        mode,
                        register,
                        operand_direction,
                    },
                    _ => ILLEGAL,
                },
                _ => match (mode, size) {
                    (AddressingMode::Illegal, _)
                    | (AddressingMode::AddressRegister(_), Size::Byte) => ILLEGAL,
                    _ => SUB {
                        size,
                        mode,
                        register,
                        operand_direction,
                    },
                },
            },
        },
        0b1011 => match size {
            Size::Illegal => match mode {
                AddressingMode::Illegal => ILLEGAL,
                _ => CMPA {
                    mode,
                    size: Size::from_opcode_bit(opcode_hex, 8),
                    register,
                },
            },
            _ => {
                if (opcode_hex >> 8) & 0b1 == 0 {
                    match (mode, size) {
                        (AddressingMode::Illegal, _)
                        | (AddressingMode::AddressRegister(_), Size::Byte) => ILLEGAL,
                        _ => CMP {
                            mode,
                            size,
                            register,
                        },
                    }
                } else {
                    match mode {
                        AddressingMode::AddressRegister(_) => CMPM {
                            size,
                            src_register,
                            dest_register: register,
                        },
                        _ if mode.is_data_alterable() => EOR {
                            mode,
                            size,
                            operand_direction,
                            register,
                        },
                        _ => ILLEGAL,
                    }
                }
            }
        },
        0b1100 => {
            if (opcode_hex >> 4) & 0b11111 == 0b10000 {
                ABCD {
                    operand_mode,
                    src_register,
                    dest_register: register,
                }
            } else {
                match size {
                    Size::Illegal => match mode {
                        AddressingMode::AddressRegister(_) | AddressingMode::Illegal => ILLEGAL,
                        _ => {
                            if (opcode_hex >> 8) & 0b1 == 0 {
                                MULU { mode, register }
                            } else {
                                MULS { mode, register }
                            }
                        }
                    },
                    _ => match operand_direction {
                        OperandDirection::ToMemory => match mode {
                            AddressingMode::DataRegister(_)
                            | AddressingMode::AddressRegister(_) => EXG {
                                mode: ExchangeMode::from_opcode(opcode_hex),
                                src_register: register,
                                dest_register: src_register,
                            },
                            _ if mode.is_memory_alterable() => AND {
                                size,
                                mode,
                                register,
                                operand_direction,
                            },
                            _ => ILLEGAL,
                        },
                        OperandDirection::ToRegister => match mode {
                            AddressingMode::AddressRegister(_) | AddressingMode::Illegal => ILLEGAL,
                            _ => AND {
                                size,
                                mode,
                                register,
                                operand_direction,
                            },
                        },
                    },
                }
            }
        }
        0b1101 => match size {
            Size::Illegal => match mode {
                AddressingMode::Illegal => ILLEGAL,
                _ => ADDA {
                    mode,
                    register,
                    size: Size::from_opcode_bit(opcode_hex, 8),
                },
            },
            _ => match operand_direction {
                OperandDirection::ToMemory => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => ADDX {
                        size,
                        operand_mode,
                        src_register,
                        dest_register: register,
                    },
                    _ => {
                        if mode.is_memory_alterable() {
                            ADD {
                                size,
                                mode,
                                register,
                                operand_direction,
                            }
                        } else {
                            ILLEGAL
                        }
                    }
                },
                OperandDirection::ToRegister => match (mode, size) {
                    (AddressingMode::Illegal, _)
                    | (AddressingMode::AddressRegister(_), Size::Byte) => ILLEGAL,
                    _ => ADD {
                        size,
                        mode,
                        register,
                        operand_direction,
                    },
                },
            },
        },
        0b1110 => {
            let direction = (opcode_hex >> 8) & 0b1;
            match size {
                Size::Illegal => {
                    if mode.is_memory_alterable() {
                        match (opcode_hex >> 9) & 0b111 {
                            0b000 => {
                                if direction == 0 {
                                    ASR {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                } else {
                                    ASL {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                }
                            }
                            0b001 => {
                                if direction == 0 {
                                    LSR {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                } else {
                                    LSL {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                }
                            }
                            0b010 => {
                                if direction == 0 {
                                    ROXR {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                } else {
                                    ROXL {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                }
                            }
                            0b011 => {
                                if direction == 0 {
                                    ROR {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                } else {
                                    ROL {
                                        mode,
                                        size: Size::Word,
                                        register: None,
                                        shift_register: None,
                                        shift_count: None,
                                    }
                                }
                            }
                            _ => ILLEGAL,
                        }
                    } else {
                        ILLEGAL
                    }
                }
                _ => {
                    let shift_count = if (opcode_hex >> 5) & 0b1 == 0 {
                        let count = ((opcode_hex >> 9) & 0b111) as u8;
                        Some(if count == 0 { 8 } else { count })
                    } else {
                        None
                    };
                    let shift_register = if (opcode_hex >> 5) & 0b1 == 1 {
                        Some(((opcode_hex >> 9) & 0b111) as usize)
                    } else {
                        None
                    };
                    match (opcode_hex >> 3) & 0b11 {
                        0b00 => {
                            if direction == 0 {
                                ASR {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            } else {
                                ASL {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            }
                        }
                        0b01 => {
                            if direction == 0 {
                                LSR {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            } else {
                                LSL {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            }
                        }
                        0b10 => {
                            if direction == 0 {
                                ROXR {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            } else {
                                ROXL {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            }
                        }
                        0b11 => {
                            if direction == 0 {
                                ROR {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            } else {
                                ROL {
                                    mode: AddressingMode::Illegal,
                                    size,
                                    register: Some(src_register),
                                    shift_register,
                                    shift_count,
                                }
                            }
                        }
                        _ => ILLEGAL,
                    }
                }
            }
        }
        _ => ILLEGAL,
    }
}

pub fn brief_extension_word(extension: u16) -> (AddressingMode, Size, i8) {
    let displacement = (extension & 0b11111111) as i8;
    let register = ((extension >> 12) & 0b111) as usize;
    let mode = if (extension >> 15) == 0b1 {
        AddressingMode::AddressRegister(register)
    } else {
        AddressingMode::DataRegister(register)
    };
    let size = if (extension >> 11) & 0b1 == 0 {
        Size::Word
    } else {
        Size::Long
    };
    (mode, size, displacement)
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::ABCD {
                operand_mode,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => {
                    f.write_fmt(format_args!("ABCD D{}, D{}", src_register, dest_register))
                }
                OperandMode::MemoryToMemory => f.write_fmt(format_args!(
                    "ABCD -(A{}), -(A{})",
                    src_register, dest_register
                )),
            },
            Opcode::ADD {
                mode,
                size,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    f.write_fmt(format_args!("ADD{} {}, D{}", size, mode, register))
                }
                OperandDirection::ToMemory => {
                    f.write_fmt(format_args!("ADD{} D{}, {}", size, register, mode))
                }
            },
            Opcode::ADDA {
                mode,
                size,
                register,
            } => f.write_fmt(format_args!("ADDA{} {}, A{}", size, mode, register)),
            Opcode::ADDI { mode, size } => f.write_fmt(format_args!("ADDI{} #, {}", size, mode)),
            Opcode::ADDQ { mode, size, data } => {
                f.write_fmt(format_args!("ADDQ{} {}, {}", size, data, mode))
            }
            Opcode::ADDX {
                operand_mode,
                size,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => f.write_fmt(format_args!(
                    "ADDX{} D{}, D{}",
                    size, src_register, dest_register
                )),
                OperandMode::MemoryToMemory => f.write_fmt(format_args!(
                    "ADDX{} -(A{}), -(A{})",
                    size, src_register, dest_register
                )),
            },
            Opcode::AND {
                mode,
                size,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    f.write_fmt(format_args!("AND{} {}, D{}", size, mode, register))
                }
                OperandDirection::ToMemory => {
                    f.write_fmt(format_args!("AND{} D{}, {}", size, register, mode))
                }
            },
            Opcode::ANDI { mode, size } => f.write_fmt(format_args!("ANDI{} #, {}", size, mode)),
            Opcode::ANDI_to_CCR => f.write_str("ANDI #, CCR"),
            Opcode::ANDI_to_SR => f.write_str("ANDI #, SR"),
            Opcode::ASL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "ASL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "ASL{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("ASL{} {}", size, mode)),
            },
            Opcode::ASR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "ASR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "ASR{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("ASR{} {}", size, mode)),
            },
            Opcode::Bcc {
                condition,
                displacement,
            } => f.write_fmt(format_args!(
                "B{} {}",
                condition,
                if *displacement == 0 {
                    "#".to_string()
                } else {
                    displacement.to_string()
                }
            )),
            Opcode::BCHG { bit_num, mode } => match bit_num {
                BitNum::Immediate => f.write_fmt(format_args!("BCHG #, {}", mode)),
                BitNum::DataRegister(register) => {
                    f.write_fmt(format_args!("BCHG D{}, {}", register, mode))
                }
            },
            Opcode::BCLR { bit_num, mode } => match bit_num {
                BitNum::Immediate => f.write_fmt(format_args!("BCLR #, {}", mode)),
                BitNum::DataRegister(register) => {
                    f.write_fmt(format_args!("BCLR D{}, {}", register, mode))
                }
            },
            Opcode::BRA { displacement } => f.write_fmt(format_args!(
                "BRA {}",
                if *displacement == 0 {
                    "#".to_string()
                } else {
                    displacement.to_string()
                }
            )),
            Opcode::BSET { bit_num, mode } => match bit_num {
                BitNum::Immediate => f.write_fmt(format_args!("BSET #, {}", mode)),
                BitNum::DataRegister(register) => {
                    f.write_fmt(format_args!("BSET D{}, {}", register, mode))
                }
            },
            Opcode::BSR { displacement } => f.write_fmt(format_args!(
                "BSR {}",
                if *displacement == 0 {
                    "#".to_string()
                } else {
                    displacement.to_string()
                }
            )),
            Opcode::BTST { bit_num, mode } => match bit_num {
                BitNum::Immediate => f.write_fmt(format_args!("BTST #, {}", mode)),
                BitNum::DataRegister(register) => {
                    f.write_fmt(format_args!("BTST D{}, {}", register, mode))
                }
            },
            Opcode::CHK { register, mode } => {
                f.write_fmt(format_args!("CHK {}, D{}", mode, register))
            }
            Opcode::CLR { mode, size } => f.write_fmt(format_args!("CLR{} {}", size, mode)),
            Opcode::CMP {
                mode,
                size,
                register,
            } => f.write_fmt(format_args!("CMP{} {}, D{}", size, mode, register)),
            Opcode::CMPA {
                mode,
                size,
                register,
            } => f.write_fmt(format_args!("CMPA{} {}, A{}", size, mode, register)),
            Opcode::CMPI { mode, size } => f.write_fmt(format_args!("CMPI{} #, {}", size, mode)),
            Opcode::CMPM {
                size,
                src_register,
                dest_register,
            } => f.write_fmt(format_args!(
                "CMPM{} (A{})+, (A{})+",
                size, src_register, dest_register
            )),
            Opcode::DBcc {
                condition,
                register,
            } => f.write_fmt(format_args!("DB{} D{}, #", condition, register)),
            Opcode::DIVS { mode, register } => {
                f.write_fmt(format_args!("DIVS {}, D{}", mode, register))
            }
            Opcode::DIVU { mode, register } => {
                f.write_fmt(format_args!("DIVU {}, D{}", mode, register))
            }
            Opcode::EOR {
                size,
                mode,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToMemory => {
                    f.write_fmt(format_args!("EOR{} D{}, {}", size, register, mode))
                }
                OperandDirection::ToRegister => f.write_str("ILLEGAL"),
            },
            Opcode::EORI { mode, size } => f.write_fmt(format_args!("EORI{} #, {}", size, mode)),
            Opcode::EORI_to_CCR => f.write_str("EORI #, CCR"),
            Opcode::EORI_to_SR => f.write_str("EORI #, SR"),
            Opcode::EXG {
                mode,
                src_register,
                dest_register,
            } => match mode {
                ExchangeMode::DataRegisters => {
                    f.write_fmt(format_args!("EXG D{}, D{}", src_register, dest_register))
                }
                ExchangeMode::AddressRegisters => {
                    f.write_fmt(format_args!("EXG A{}, A{}", src_register, dest_register))
                }
                ExchangeMode::DataRegisterAndAddressRegister => {
                    f.write_fmt(format_args!("EXG D{}, A{}", src_register, dest_register))
                }
                ExchangeMode::Illegal => f.write_str("ILLEGAL"),
            },
            Opcode::EXT { mode, size } => f.write_fmt(format_args!("EXT{} {}", size, mode)),
            Opcode::ILLEGAL => f.write_str("ILLEGAL"),
            Opcode::JMP { mode } => f.write_fmt(format_args!("JMP {}", mode)),
            Opcode::JSR { mode } => f.write_fmt(format_args!("JSR {}", mode)),
            Opcode::LEA { register, mode } => {
                f.write_fmt(format_args!("LEA {}, A{}", mode, register))
            }
            Opcode::LINK { register } => f.write_fmt(format_args!("LINK A{}, #", register)),
            Opcode::LSL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "LSL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "LSL{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("LSL{} {}", size, mode)),
            },
            Opcode::LSR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "LSR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "LSR{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("LSR{} {}", size, mode)),
            },
            Opcode::MOVE {
                src_mode,
                dest_mode,
                size,
            } => f.write_fmt(format_args!("MOVE{} {}, {}", size, src_mode, dest_mode)),
            Opcode::MOVEA {
                src_mode,
                dest_mode,
                size,
            } => f.write_fmt(format_args!("MOVEA{} {}, {}", size, src_mode, dest_mode)),
            Opcode::MOVE_to_CCR { mode } => f.write_fmt(format_args!("MOVE {}, CCR", mode)),
            Opcode::MOVE_from_SR { mode } => f.write_fmt(format_args!("MOVE SR, {}", mode)),
            Opcode::MOVE_to_SR { mode } => f.write_fmt(format_args!("MOVE {}, SR", mode)),
            Opcode::MOVE_USP {
                register,
                direction,
            } => match direction {
                Direction::RegisterToMemory => f.write_fmt(format_args!("MOVE A{}, USP", register)),
                Direction::MemoryToRegister => f.write_fmt(format_args!("MOVE USP, A{}", register)),
            },
            Opcode::MOVEM {
                mode,
                size,
                direction,
            } => match direction {
                Direction::RegisterToMemory => {
                    f.write_fmt(format_args!("MOVEM{} #, {}", size, mode))
                }
                Direction::MemoryToRegister => {
                    f.write_fmt(format_args!("MOVEM{} {}, #", size, mode))
                }
            },
            Opcode::MOVEP {
                data_register,
                address_register,
                direction,
                size,
            } => match direction {
                Direction::RegisterToMemory => f.write_fmt(format_args!(
                    "MOVEP{} D{}, (d16, A{})",
                    size, data_register, address_register
                )),
                Direction::MemoryToRegister => f.write_fmt(format_args!(
                    "MOVEP{} (d16, A{}), D{}",
                    size, address_register, data_register
                )),
            },
            Opcode::MOVEQ { register, data } => {
                f.write_fmt(format_args!("MOVEQ {}, D{}", data, register))
            }
            Opcode::MULS { mode, register } => {
                f.write_fmt(format_args!("MULS {}, D{}", mode, register))
            }
            Opcode::MULU { mode, register } => {
                f.write_fmt(format_args!("MULU {}, D{}", mode, register))
            }
            Opcode::NBCD { mode } => f.write_fmt(format_args!("NBCD {}", mode)),
            Opcode::NEG { mode, size } => f.write_fmt(format_args!("NEG{} {}", size, mode)),
            Opcode::NEGX { mode, size } => f.write_fmt(format_args!("NEGX{} {}", size, mode)),
            Opcode::NOP => f.write_str("NOP"),
            Opcode::NOT { mode, size } => f.write_fmt(format_args!("NOT{} {}", size, mode)),
            Opcode::OR {
                size,
                mode,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    f.write_fmt(format_args!("OR{} {}, D{}", size, mode, register))
                }
                OperandDirection::ToMemory => {
                    f.write_fmt(format_args!("OR{} D{}, {}", size, register, mode))
                }
            },
            Opcode::ORI { mode, size } => f.write_fmt(format_args!("ORI{} #, {}", size, mode)),
            Opcode::ORI_to_CCR => f.write_str("ORI #, CCR"),
            Opcode::ORI_to_SR => f.write_str("ORI #, SR"),
            Opcode::PEA { mode } => f.write_fmt(format_args!("PEA {}", mode)),
            Opcode::RESET => f.write_str("RESET"),
            Opcode::ROL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "ROL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "ROL{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("ROL{} {}", size, mode)),
            },
            Opcode::ROR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "ROR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "ROR{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("ROR{} {}", size, mode)),
            },
            Opcode::ROXL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "ROXL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "ROXL{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("ROXL{} {}", size, mode)),
            },
            Opcode::ROXR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => f.write_fmt(format_args!(
                        "ROXR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    )),
                    Some(shift_count) => f.write_fmt(format_args!(
                        "ROXR{} {}, D{}",
                        size,
                        shift_count,
                        register.unwrap()
                    )),
                },
                _ => f.write_fmt(format_args!("ROXR{} {}", size, mode)),
            },
            Opcode::RTE => f.write_str("RTE"),
            Opcode::RTR => f.write_str("RTR"),
            Opcode::RTS => f.write_str("RTS"),
            Opcode::SBCD {
                operand_mode,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => {
                    f.write_fmt(format_args!("SBCD D{}, D{}", src_register, dest_register))
                }
                OperandMode::MemoryToMemory => f.write_fmt(format_args!(
                    "SBCD -(A{}), -(A{})",
                    src_register, dest_register
                )),
            },
            Opcode::Scc { mode, condition } => f.write_fmt(format_args!("S{} {}", condition, mode)),
            Opcode::STOP => f.write_str("STOP #"),
            Opcode::SUB {
                mode,
                size,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    f.write_fmt(format_args!("SUB{} {}, D{}", size, mode, register))
                }
                OperandDirection::ToMemory => {
                    f.write_fmt(format_args!("SUB{} D{}, {}", size, register, mode))
                }
            },
            Opcode::SUBA {
                mode,
                size,
                register,
            } => f.write_fmt(format_args!("SUBA{} {}, A{}", size, mode, register)),
            Opcode::SUBI { mode, size } => f.write_fmt(format_args!("SUBI{} #, {}", size, mode)),
            Opcode::SUBQ { mode, size, data } => {
                f.write_fmt(format_args!("SUBQ{} {}, {}", size, data, mode))
            }
            Opcode::SUBX {
                operand_mode,
                size,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => f.write_fmt(format_args!(
                    "SUBX{} D{}, D{}",
                    size, src_register, dest_register
                )),
                OperandMode::MemoryToMemory => f.write_fmt(format_args!(
                    "SUBX{} -(A{}), -(A{})",
                    size, src_register, dest_register
                )),
            },
            Opcode::SWAP { mode } => f.write_fmt(format_args!("SWAP {}", mode)),
            Opcode::TAS { mode } => f.write_fmt(format_args!("TAS {}", mode)),
            Opcode::TRAP { vector } => f.write_fmt(format_args!("TRAP {}", vector)),
            Opcode::TRAPV => f.write_str("TRAPV"),
            Opcode::TST { mode, size } => f.write_fmt(format_args!("TST{} {}", size, mode)),
            Opcode::UNLK { register } => f.write_fmt(format_args!("UNLK A{}", register)),
        }
    }
}
