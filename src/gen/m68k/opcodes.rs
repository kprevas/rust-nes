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

    fn extension_bytes(self, size: Size) -> u8 {
        match self {
            AddressingMode::DataRegister(_)
            | AddressingMode::AddressRegister(_)
            | AddressingMode::Address(_)
            | AddressingMode::AddressWithPostincrement(_)
            | AddressingMode::AddressWithPredecrement(_)
            | AddressingMode::Illegal => 0,
            AddressingMode::AddressWithDisplacement(_)
            | AddressingMode::AddressWithIndex(_)
            | AddressingMode::ProgramCounterWithDisplacement
            | AddressingMode::ProgramCounterWithIndex
            | AddressingMode::AbsoluteShort => 2,
            AddressingMode::AbsoluteLong => 4,
            AddressingMode::Immediate => size.extension_bytes(),
        }
    }

    pub fn disassemble(&self, ext: Option<&[u8]>, size: Size) -> String {
        match self {
            AddressingMode::DataRegister(register) => format!("D{}", register),
            AddressingMode::AddressRegister(register) => format!("A{}", register),
            AddressingMode::Address(register) => format!("(A{})", register),
            AddressingMode::AddressWithPostincrement(register) => {
                format!("(A{})+", register)
            }
            AddressingMode::AddressWithPredecrement(register) => {
                format!("-(A{})", register)
            }
            AddressingMode::AddressWithDisplacement(register) => {
                format!(
                    "({}, A{})",
                    match ext {
                        None => "d16".to_string(),
                        Some(val) => format!("${:02X}{:02X}", val[0], val[1]),
                    },
                    register
                )
            }
            AddressingMode::AddressWithIndex(register) => match ext {
                None => format!("(d8, A{}, Xn)", register),
                Some(val) => {
                    let (mode, _, index) =
                        brief_extension_word(((val[0] as u16) << 8) | (val[1] as u16));
                    format!(
                        "(${:02X}, A{}, {})",
                        index,
                        register,
                        mode.disassemble(None, size)
                    )
                }
            },
            AddressingMode::ProgramCounterWithDisplacement => format!(
                "({}, PC)",
                match ext {
                    None => "d16".to_string(),
                    Some(val) => format!("${:02X}{:02X}", val[0], val[1]),
                }
            ),
            AddressingMode::ProgramCounterWithIndex => match ext {
                None => "(d8, PC, Xn)".to_string(),
                Some(val) => {
                    let (mode, _, index) =
                        brief_extension_word(((val[0] as u16) << 8) | (val[1] as u16));
                    format!("(${:02X}, PC, {})", index, mode.disassemble(None, size))
                }
            },
            AddressingMode::AbsoluteShort => format!(
                "({}).w",
                match ext {
                    None => "xxx".to_string(),
                    Some(val) => format!("${:02X}{:02X}", val[0], val[1]),
                }
            ),
            AddressingMode::AbsoluteLong => format!(
                "({}).l",
                match ext {
                    None => "xxx".to_string(),
                    Some(val) =>
                        format!("${:02X}{:02X}{:02X}{:02X}", val[0], val[1], val[2], val[3]),
                }
            ),
            AddressingMode::Immediate => match ext {
                None => "#".to_string(),
                Some(_) => size.display(ext),
            },
            AddressingMode::Illegal => "XXX".to_string(),
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

    pub fn display(&self, val: Option<&[u8]>) -> String {
        match (val, self) {
            (None, _) | (_, Size::Illegal) => "#".to_string(),
            (Some(val), Size::Byte) => format!("${:02X}", val[1]),
            (Some(val), Size::Word) => format!("${:02X}{:02X}", val[0], val[1]),
            (Some(val), Size::Long) => {
                format!("${:02X}{:02X}{:02X}{:02X}", val[0], val[1], val[2], val[3])
            }
        }
    }

    pub fn display_signed(&self, val: Option<&[u8]>) -> String {
        match (val, self) {
            (None, _) | (_, Size::Illegal) => "#".to_string(),
            (Some(val), Size::Byte) => {
                let val = val[1] as i8;
                if val < 0 {
                    format!("-${:02X}", -val)
                } else {
                    format!("${:02X}", val)
                }
            },
            (Some(val), Size::Word) => {
                let val = (((val[0] as u16) << 8) | val[1] as u16) as i16;
                if val < 0 {
                    format!("-${:04X}", -val)
                } else {
                    format!("${:04X}", val)
                }
            }
            (Some(val), Size::Long) => {
                let val = (((val[0] as u32) << 24)
                    | ((val[1] as u32) << 16)
                    | ((val[2] as u32) << 8)
                    | val[3] as u32) as i32;
                if val < 0 {
                    format!("-${:08X}", -val)
                } else {
                    format!("${:08X}", val)
                }
            },
        }
    }

    pub fn extension_bytes(&self) -> u8 {
        match self {
            Size::Byte => 2,
            Size::Word => 2,
            Size::Long => 4,
            Size::Illegal => 0,
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
        register: usize,
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
        register: usize,
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
                                AddressingMode::DataRegister(register) => SWAP { register },
                                _ if mode.is_control_addressing() => PEA { mode },
                                _ => ILLEGAL,
                            },
                            _ => match mode {
                                AddressingMode::DataRegister(register) => EXT {
                                    register,
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
                            | AddressingMode::AddressRegister(_) => {
                                let exg_mode = ExchangeMode::from_opcode(opcode_hex);
                                if let ExchangeMode::Illegal = exg_mode {
                                    ILLEGAL
                                } else {
                                    EXG {
                                        mode: exg_mode,
                                        src_register: register,
                                        dest_register: src_register,
                                    }
                                }
                            }
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

impl Opcode {
    pub fn disassemble(&self, ext: Option<&[u8]>) -> String {
        match self {
            Opcode::ABCD {
                operand_mode,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => {
                    format!("ABCD D{}, D{}", src_register, dest_register)
                }
                OperandMode::MemoryToMemory => {
                    format!("ABCD -(A{}), -(A{})", src_register, dest_register)
                }
            },
            Opcode::ADD {
                mode,
                size,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    format!(
                        "ADD{} {}, D{}",
                        size,
                        mode.disassemble(ext, *size),
                        register
                    )
                }
                OperandDirection::ToMemory => {
                    format!(
                        "ADD{} D{}, {}",
                        size,
                        register,
                        mode.disassemble(ext, *size)
                    )
                }
            },
            Opcode::ADDA {
                mode,
                size,
                register,
            } => format!(
                "ADDA{} {}, A{}",
                size,
                mode.disassemble(ext, *size),
                register
            ),
            Opcode::ADDI { mode, size } => {
                format!(
                    "ADDI{} {}, {}",
                    size,
                    size.display(ext),
                    mode.disassemble(
                        ext.map(|ext| &ext[(size.extension_bytes() as usize)..]),
                        *size,
                    ),
                )
            }
            Opcode::ADDQ { mode, size, data } => {
                format!("ADDQ{} {}, {}", size, data, mode.disassemble(ext, *size))
            }
            Opcode::ADDX {
                operand_mode,
                size,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => {
                    format!("ADDX{} D{}, D{}", size, src_register, dest_register)
                }
                OperandMode::MemoryToMemory => {
                    format!("ADDX{} -(A{}), -(A{})", size, src_register, dest_register)
                }
            },
            Opcode::AND {
                mode,
                size,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    format!(
                        "AND{} {}, D{}",
                        size,
                        mode.disassemble(ext, *size),
                        register
                    )
                }
                OperandDirection::ToMemory => {
                    format!(
                        "AND{} D{}, {}",
                        size,
                        register,
                        mode.disassemble(ext, *size)
                    )
                }
            },
            Opcode::ANDI { mode, size } => format!(
                "ANDI{} {}, {}",
                size,
                size.display(ext),
                mode.disassemble(
                    ext.map(|ext| &ext[(size.extension_bytes() as usize)..]),
                    *size,
                ),
            ),
            Opcode::ANDI_to_CCR => format!("ANDI {}, CCR", Size::Byte.display(ext)),
            Opcode::ANDI_to_SR => format!("ANDI {}, SR", Size::Word.display(ext)),
            Opcode::ASL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "ASL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("ASL{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("ASL{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::ASR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "ASR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("ASR{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("ASR{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::Bcc {
                condition,
                displacement,
            } => format!(
                "B{} {}",
                condition,
                if *displacement == 0 {
                    Size::Word.display(ext)
                } else {
                    displacement.to_string()
                }
            ),
            Opcode::BCHG { bit_num, mode } => match bit_num {
                BitNum::Immediate => format!(
                    "BCHG {}, {}",
                    Size::Word.display(ext),
                    mode.disassemble(
                        ext.map(|ext| &ext[(Size::Word.extension_bytes() as usize)..]),
                        Size::Byte,
                    )
                ),
                BitNum::DataRegister(register) => {
                    format!("BCHG D{}, {}", register, mode.disassemble(ext, Size::Byte))
                }
            },
            Opcode::BCLR { bit_num, mode } => match bit_num {
                BitNum::Immediate => format!(
                    "BCLR {}, {}",
                    Size::Word.display(ext),
                    mode.disassemble(
                        ext.map(|ext| &ext[(Size::Word.extension_bytes() as usize)..]),
                        Size::Byte,
                    )
                ),
                BitNum::DataRegister(register) => {
                    format!("BCLR D{}, {}", register, mode.disassemble(ext, Size::Byte))
                }
            },
            Opcode::BRA { displacement } => format!(
                "BRA {}",
                if *displacement == 0 {
                    Size::Word.display_signed(ext)
                } else {
                    displacement.to_string()
                }
            ),
            Opcode::BSET { bit_num, mode } => match bit_num {
                BitNum::Immediate => {
                    format!(
                        "BSET {}, {}",
                        Size::Word.display(ext),
                        mode.disassemble(
                            ext.map(|ext| &ext[(Size::Word.extension_bytes() as usize)..]),
                            Size::Byte,
                        )
                    )
                }
                BitNum::DataRegister(register) => {
                    format!("BSET D{}, {}", register, mode.disassemble(ext, Size::Byte))
                }
            },
            Opcode::BSR { displacement } => format!(
                "BSR {}",
                if *displacement == 0 {
                    Size::Word.display(ext)
                } else {
                    displacement.to_string()
                }
            ),
            Opcode::BTST { bit_num, mode } => match bit_num {
                BitNum::Immediate => format!(
                    "BTST {}, {}",
                    Size::Word.display(ext),
                    mode.disassemble(
                        ext.map(|ext| &ext[(Size::Word.extension_bytes() as usize)..]),
                        Size::Byte,
                    )
                ),
                BitNum::DataRegister(register) => {
                    format!("BTST D{}, {}", register, mode.disassemble(ext, Size::Byte))
                }
            },
            Opcode::CHK { register, mode } => {
                format!("CHK {}, D{}", mode.disassemble(ext, Size::Word), register)
            }
            Opcode::CLR { mode, size } => {
                format!("CLR{} {}", size, mode.disassemble(ext, *size))
            }
            Opcode::CMP {
                mode,
                size,
                register,
            } => format!(
                "CMP{} {}, D{}",
                size,
                mode.disassemble(ext, *size),
                register
            ),
            Opcode::CMPA {
                mode,
                size,
                register,
            } => format!(
                "CMPA{} {}, A{}",
                size,
                mode.disassemble(ext, *size),
                register
            ),
            Opcode::CMPI { mode, size } => format!(
                "CMPI{} {}, {}",
                size,
                size.display(ext),
                mode.disassemble(
                    ext.map(|ext| &ext[(size.extension_bytes() as usize)..]),
                    *size,
                ),
            ),
            Opcode::CMPM {
                size,
                src_register,
                dest_register,
            } => format!("CMPM{} (A{})+, (A{})+", size, src_register, dest_register),
            Opcode::DBcc {
                condition,
                register,
            } => format!(
                "DB{} D{}, {}",
                condition,
                register,
                Size::Word.display_signed(ext)
            ),
            Opcode::DIVS { mode, register } => {
                format!("DIVS {}, D{}", mode.disassemble(ext, Size::Word), register)
            }
            Opcode::DIVU { mode, register } => {
                format!("DIVU {}, D{}", mode.disassemble(ext, Size::Word), register)
            }
            Opcode::EOR {
                size,
                mode,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToMemory => {
                    format!(
                        "EOR{} D{}, {}",
                        size,
                        register,
                        mode.disassemble(ext, *size)
                    )
                }
                OperandDirection::ToRegister => "ILLEGAL".to_string(),
            },
            Opcode::EORI { mode, size } => format!(
                "EORI{} {}, {}",
                size,
                size.display(ext),
                mode.disassemble(
                    ext.map(|ext| &ext[(size.extension_bytes() as usize)..]),
                    *size,
                ),
            ),
            Opcode::EORI_to_CCR => format!("EORI {}, CCR", Size::Byte.display(ext)),
            Opcode::EORI_to_SR => format!("EORI {}, SR", Size::Word.display(ext)),
            Opcode::EXG {
                mode,
                src_register,
                dest_register,
            } => match mode {
                ExchangeMode::DataRegisters => {
                    format!("EXG D{}, D{}", src_register, dest_register)
                }
                ExchangeMode::AddressRegisters => {
                    format!("EXG A{}, A{}", src_register, dest_register)
                }
                ExchangeMode::DataRegisterAndAddressRegister => {
                    format!("EXG D{}, A{}", src_register, dest_register)
                }
                ExchangeMode::Illegal => "ILLEGAL".to_string(),
            },
            Opcode::EXT { register, size } => {
                format!("EXT{} D{}", size, register)
            }
            Opcode::ILLEGAL => "ILLEGAL".to_string(),
            Opcode::JMP { mode } => format!("JMP {}", mode.disassemble(ext, Size::Illegal)),
            Opcode::JSR { mode } => format!("JSR {}", mode.disassemble(ext, Size::Illegal)),
            Opcode::LEA { register, mode } => {
                format!("LEA {}, A{}", mode.disassemble(ext, Size::Long), register)
            }
            Opcode::LINK { register } => {
                format!("LINK A{}, {}", register, Size::Word.display_signed(ext))
            }
            Opcode::LSL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "LSL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("LSL{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("LSL{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::LSR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "LSR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("LSR{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("LSR{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::MOVE {
                src_mode,
                dest_mode,
                size,
            } => format!(
                "MOVE{} {}, {}",
                size,
                src_mode.disassemble(ext, *size),
                dest_mode.disassemble(
                    ext.map(|ext| &ext[(src_mode.extension_bytes(*size) as usize)..]),
                    *size,
                )
            ),
            Opcode::MOVEA {
                src_mode,
                dest_mode,
                size,
            } => format!(
                "MOVEA{} {}, {}",
                size,
                src_mode.disassemble(ext, *size),
                dest_mode.disassemble(
                    ext.map(|ext| &ext[(src_mode.extension_bytes(*size) as usize)..]),
                    *size,
                )
            ),
            Opcode::MOVE_to_CCR { mode } => {
                format!("MOVE {}, CCR", mode.disassemble(ext, Size::Byte))
            }
            Opcode::MOVE_from_SR { mode } => {
                format!("MOVE SR, {}", mode.disassemble(ext, Size::Word))
            }
            Opcode::MOVE_to_SR { mode } => {
                format!("MOVE {}, SR", mode.disassemble(ext, Size::Word))
            }
            Opcode::MOVE_USP {
                register,
                direction,
            } => match direction {
                Direction::RegisterToMemory => format!("MOVE A{}, USP", register),
                Direction::MemoryToRegister => format!("MOVE USP, A{}", register),
            },
            Opcode::MOVEM {
                mode,
                size,
                direction,
            } => {
                let mode = mode.disassemble(
                    ext.map(|ext| &ext[(Size::Word.extension_bytes() as usize)..]),
                    *size,
                );
                match direction {
                    Direction::RegisterToMemory => match ext {
                        None => format!("MOVEM{} #, {}", size, mode),
                        Some(val) => {
                            format!("MOVEM{} {:08b}{:08b}, {}", size, val[0], val[1], mode)
                        }
                    },
                    Direction::MemoryToRegister => match ext {
                        None => format!("MOVEM{} {}, #", size, mode),
                        Some(val) => {
                            format!("MOVEM{} {}, {:08b}{:08b}", size, mode, val[0], val[1], )
                        }
                    },
                }
            }
            Opcode::MOVEP {
                data_register,
                address_register,
                direction,
                size,
            } => match direction {
                Direction::RegisterToMemory => format!(
                    "MOVEP{} D{}, (d16, A{})",
                    size, data_register, address_register
                ),
                Direction::MemoryToRegister => format!(
                    "MOVEP{} (d16, A{}), D{}",
                    size, address_register, data_register
                ),
            },
            Opcode::MOVEQ { register, data } => {
                format!("MOVEQ {}, D{}", data, register)
            }
            Opcode::MULS { mode, register } => {
                format!("MULS {}, D{}", mode.disassemble(ext, Size::Word), register)
            }
            Opcode::MULU { mode, register } => {
                format!("MULU {}, D{}", mode.disassemble(ext, Size::Word), register)
            }
            Opcode::NBCD { mode } => format!("NBCD {}", mode.disassemble(ext, Size::Byte)),
            Opcode::NEG { mode, size } => format!("NEG{} {}", size, mode.disassemble(ext, *size)),
            Opcode::NEGX { mode, size } => {
                format!("NEGX{} {}", size, mode.disassemble(ext, *size))
            }
            Opcode::NOP => "NOP".to_string(),
            Opcode::NOT { mode, size } => format!("NOT{} {}", size, mode.disassemble(ext, *size)),
            Opcode::OR {
                size,
                mode,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    format!("OR{} {}, D{}", size, mode.disassemble(ext, *size), register)
                }
                OperandDirection::ToMemory => {
                    format!("OR{} D{}, {}", size, register, mode.disassemble(ext, *size))
                }
            },
            Opcode::ORI { mode, size } => format!(
                "ORI{} {}, {}",
                size,
                size.display(ext),
                mode.disassemble(
                    ext.map(|ext| &ext[(size.extension_bytes() as usize)..]),
                    *size,
                )
            ),
            Opcode::ORI_to_CCR => format!("ORI {}, CCR", Size::Byte.display(ext)),
            Opcode::ORI_to_SR => format!("ORI {}, SR", Size::Word.display(ext)),
            Opcode::PEA { mode } => format!("PEA {}", mode.disassemble(ext, Size::Long)),
            Opcode::RESET => "RESET".to_string(),
            Opcode::ROL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "ROL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("ROL{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("ROL{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::ROR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "ROR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("ROR{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("ROR{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::ROXL {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "ROXL{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("ROXL{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("ROXL{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::ROXR {
                mode,
                size,
                register,
                shift_count,
                shift_register,
            } => match mode {
                AddressingMode::Illegal => match shift_count {
                    None => format!(
                        "ROXR{} D{}, D{}",
                        size,
                        shift_register.unwrap(),
                        register.unwrap()
                    ),
                    Some(shift_count) => {
                        format!("ROXR{} {}, D{}", size, shift_count, register.unwrap())
                    }
                },
                _ => format!("ROXR{} {}", size, mode.disassemble(ext, *size)),
            },
            Opcode::RTE => "RTE".to_string(),
            Opcode::RTR => "RTR".to_string(),
            Opcode::RTS => "RTS".to_string(),
            Opcode::SBCD {
                operand_mode,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => {
                    format!("SBCD D{}, D{}", src_register, dest_register)
                }
                OperandMode::MemoryToMemory => {
                    format!("SBCD -(A{}), -(A{})", src_register, dest_register)
                }
            },
            Opcode::Scc { mode, condition } => {
                format!("S{} {}", condition, mode.disassemble(ext, Size::Byte))
            }
            Opcode::STOP => format!("STOP {}", Size::Word.display(ext)),
            Opcode::SUB {
                mode,
                size,
                operand_direction,
                register,
            } => match operand_direction {
                OperandDirection::ToRegister => {
                    format!(
                        "SUB{} {}, D{}",
                        size,
                        mode.disassemble(ext, *size),
                        register
                    )
                }
                OperandDirection::ToMemory => {
                    format!(
                        "SUB{} D{}, {}",
                        size,
                        register,
                        mode.disassemble(ext, *size)
                    )
                }
            },
            Opcode::SUBA {
                mode,
                size,
                register,
            } => format!(
                "SUBA{} {}, A{}",
                size,
                mode.disassemble(ext, *size),
                register
            ),
            Opcode::SUBI { mode, size } => format!(
                "SUBI{} {}, {}",
                size,
                size.display(ext),
                mode.disassemble(
                    ext.map(|ext| &ext[(size.extension_bytes() as usize)..]),
                    *size,
                )
            ),
            Opcode::SUBQ { mode, size, data } => {
                format!("SUBQ{} {}, {}", size, data, mode.disassemble(ext, *size))
            }
            Opcode::SUBX {
                operand_mode,
                size,
                src_register,
                dest_register,
            } => match operand_mode {
                OperandMode::RegisterToRegister => {
                    format!("SUBX{} D{}, D{}", size, src_register, dest_register)
                }
                OperandMode::MemoryToMemory => {
                    format!("SUBX{} -(A{}), -(A{})", size, src_register, dest_register)
                }
            },
            Opcode::SWAP { register } => format!("SWAP D{}", register),
            Opcode::TAS { mode } => format!("TAS {}", mode.disassemble(ext, Size::Byte)),
            Opcode::TRAP { vector } => format!("TRAP {}", vector),
            Opcode::TRAPV => "TRAPV".to_string(),
            Opcode::TST { mode, size } => format!("TST{} {}", size, mode.disassemble(ext, *size)),
            Opcode::UNLK { register } => format!("UNLK A{}", register),
        }
    }

    pub fn extension_bytes(&self) -> u8 {
        match self {
            Opcode::Bcc { displacement, .. }
            | Opcode::BRA { displacement, .. }
            | Opcode::BSR { displacement, .. }
            if *displacement == 0 => Size::Word.extension_bytes(),
            Opcode::Bcc { .. }
            | Opcode::BRA { .. }
            | Opcode::BSR { .. }
            => 0,

            Opcode::MOVE {
                src_mode,
                dest_mode,
                size,
                ..
            }
            | Opcode::MOVEA {
                src_mode,
                dest_mode,
                size,
                ..
            } => src_mode.extension_bytes(*size) + dest_mode.extension_bytes(*size),

            Opcode::BCHG { mode, .. }
            | Opcode::BCLR { mode, .. }
            | Opcode::BSET { mode, .. }
            | Opcode::BTST { mode, .. }
            | Opcode::MOVEM { mode, .. } => {
                mode.extension_bytes(Size::Word) + Size::Word.extension_bytes()
            }

            Opcode::ANDI_to_CCR
            | Opcode::EORI_to_CCR
            | Opcode::MOVE_to_CCR { .. }
            | Opcode::ORI_to_CCR => Size::Byte.extension_bytes(),

            Opcode::ANDI_to_SR
            | Opcode::DBcc { .. }
            | Opcode::EORI_to_SR
            | Opcode::LINK { .. }
            | Opcode::MOVE_from_SR { .. }
            | Opcode::MOVE_to_SR { .. }
            | Opcode::ORI_to_SR
            | Opcode::STOP => Size::Word.extension_bytes(),

            Opcode::ADDI { mode, size, .. }
            | Opcode::ANDI { mode, size, .. }
            | Opcode::CMPI { mode, size, .. }
            | Opcode::EORI { mode, size, .. }
            | Opcode::ORI { mode, size, .. }
            | Opcode::SUBI { mode, size, .. } => {
                mode.extension_bytes(*size) + size.extension_bytes()
            }

            Opcode::ADD { mode, size, .. }
            | Opcode::ADDA { mode, size, .. }
            | Opcode::ADDQ { mode, size, .. }
            | Opcode::AND { mode, size, .. }
            | Opcode::ASL { mode, size, .. }
            | Opcode::ASR { mode, size, .. }
            | Opcode::CLR { mode, size, .. }
            | Opcode::CMP { mode, size, .. }
            | Opcode::CMPA { mode, size, .. }
            | Opcode::EOR { mode, size, .. }
            | Opcode::LSL { mode, size, .. }
            | Opcode::LSR { mode, size, .. }
            | Opcode::NEG { mode, size, .. }
            | Opcode::NEGX { mode, size, .. }
            | Opcode::NOT { mode, size, .. }
            | Opcode::OR { mode, size, .. }
            | Opcode::ROL { mode, size, .. }
            | Opcode::ROR { mode, size, .. }
            | Opcode::ROXL { mode, size, .. }
            | Opcode::ROXR { mode, size, .. }
            | Opcode::SUB { mode, size, .. }
            | Opcode::SUBA { mode, size, .. }
            | Opcode::SUBQ { mode, size, .. }
            | Opcode::TST { mode, size, .. } => mode.extension_bytes(*size),

            Opcode::NBCD { mode, .. } | Opcode::Scc { mode, .. } | Opcode::TAS { mode, .. } => {
                mode.extension_bytes(Size::Byte)
            }

            Opcode::CHK { mode, .. }
            | Opcode::DIVS { mode, .. }
            | Opcode::DIVU { mode, .. }
            | Opcode::JMP { mode, .. }
            | Opcode::JSR { mode, .. }
            | Opcode::MULS { mode, .. }
            | Opcode::MULU { mode, .. } => mode.extension_bytes(Size::Word),

            Opcode::LEA { mode, .. } | Opcode::PEA { mode, .. } => mode.extension_bytes(Size::Long),

            Opcode::ABCD { .. }
            | Opcode::ADDX { .. }
            | Opcode::CMPM { .. }
            | Opcode::EXG { .. }
            | Opcode::EXT { .. }
            | Opcode::ILLEGAL
            | Opcode::MOVE_USP { .. }
            | Opcode::MOVEP { .. }
            | Opcode::MOVEQ { .. }
            | Opcode::NOP
            | Opcode::RESET
            | Opcode::RTE
            | Opcode::RTR
            | Opcode::RTS
            | Opcode::SBCD { .. }
            | Opcode::SUBX { .. }
            | Opcode::SWAP { .. }
            | Opcode::TRAP { .. }
            | Opcode::TRAPV
            | Opcode::UNLK { .. } => 0,
        }
    }

    pub fn cycle_count(&self) -> u8 {
        match self {
            Opcode::ADDI { mode, size, .. }
            | Opcode::ANDI { mode, size, .. }
            | Opcode::EORI { mode, size, .. }
            | Opcode::ORI { mode, size, .. }
            | Opcode::SUBI { mode, size, .. } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) => 8,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 16,
                    AddressingMode::AddressWithPredecrement(_) => 18,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 20,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 22,
                    AddressingMode::AbsoluteLong => 24,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) => 16,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 28,
                    AddressingMode::AddressWithPredecrement(_) => 30,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 32,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 34,
                    AddressingMode::AbsoluteLong => 36,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::ANDI_to_CCR
            | Opcode::EORI_to_CCR
            | Opcode::ORI_to_CCR
            | Opcode::ANDI_to_SR
            | Opcode::EORI_to_SR
            | Opcode::ORI_to_SR => 20,
            Opcode::CMPI { mode, size } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) => 8,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) => 14,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 20,
                    AddressingMode::AddressWithPredecrement(_) => 22,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 24,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 26,
                    AddressingMode::AbsoluteLong => 28,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::BCHG { mode, bit_num } | Opcode::BSET { mode, bit_num } => match bit_num {
                BitNum::Immediate => match mode {
                    AddressingMode::DataRegister(_) => 12, // TODO: 10 if value < 16
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 16,
                    AddressingMode::AddressWithPredecrement(_) => 18,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 20,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 22,
                    AddressingMode::AbsoluteLong => 24,
                    _ => panic!(),
                },
                BitNum::DataRegister(_) => match mode {
                    AddressingMode::DataRegister(_) => 8, // TODO: 6 if value < 16
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
            },
            Opcode::BCLR { mode, bit_num } => match bit_num {
                BitNum::Immediate => match mode {
                    AddressingMode::DataRegister(_) => 14, // TODO: 12 if value < 16
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 16,
                    AddressingMode::AddressWithPredecrement(_) => 18,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 20,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 22,
                    AddressingMode::AbsoluteLong => 24,
                    _ => panic!(),
                },
                BitNum::DataRegister(_) => match mode {
                    AddressingMode::DataRegister(_) => 10, // TODO: 8 if value < 16
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
            },
            Opcode::BTST { mode, bit_num } => match bit_num {
                BitNum::Immediate => match mode {
                    AddressingMode::DataRegister(_) => 10,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
                BitNum::DataRegister(_) => match mode {
                    AddressingMode::DataRegister(_) => 6,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 8,
                    AddressingMode::AddressWithPredecrement(_) => 10,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 12,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 14,
                    AddressingMode::AbsoluteLong => 16,
                    AddressingMode::Immediate => 10,
                    _ => panic!(),
                },
            },
            Opcode::MOVEP { size, .. } => match size {
                Size::Word => 16,
                Size::Long => 24,
                _ => panic!(),
            },
            Opcode::MOVE {
                size,
                src_mode,
                dest_mode,
            } => match dest_mode {
                AddressingMode::DataRegister(_) => match size {
                    Size::Byte | Size::Word => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 4,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 8,
                        AddressingMode::AddressWithPredecrement(_) => 10,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 12,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 14,
                        AddressingMode::AbsoluteLong => 16,
                        AddressingMode::Immediate => 8,
                        _ => panic!(),
                    },
                    Size::Long => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 4,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 12,
                        AddressingMode::AddressWithPredecrement(_) => 14,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 16,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 18,
                        AddressingMode::AbsoluteLong => 20,
                        AddressingMode::Immediate => 12,
                        _ => panic!(),
                    },
                    Size::Illegal => panic!(),
                },
                AddressingMode::Address(_)
                | AddressingMode::AddressWithPostincrement(_)
                | AddressingMode::AddressWithPredecrement(_) => match size {
                    Size::Byte | Size::Word => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 8,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 12,
                        AddressingMode::AddressWithPredecrement(_) => 14,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 16,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 18,
                        AddressingMode::AbsoluteLong => 20,
                        AddressingMode::Immediate => 12,
                        _ => panic!(),
                    },
                    Size::Long => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 12,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 20,
                        AddressingMode::AddressWithPredecrement(_) => 22,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 24,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 26,
                        AddressingMode::AbsoluteLong => 28,
                        AddressingMode::Immediate => 20,
                        _ => panic!(),
                    },
                    Size::Illegal => panic!(),
                },
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => match size {
                    Size::Byte | Size::Word => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 12,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 16,
                        AddressingMode::AddressWithPredecrement(_) => 18,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 20,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 22,
                        AddressingMode::AbsoluteLong => 24,
                        AddressingMode::Immediate => 16,
                        _ => panic!(),
                    },
                    Size::Long => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 16,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 24,
                        AddressingMode::AddressWithPredecrement(_) => 26,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 28,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 30,
                        AddressingMode::AbsoluteLong => 32,
                        AddressingMode::Immediate => 24,
                        _ => panic!(),
                    },
                    Size::Illegal => panic!(),
                },
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => {
                    match size {
                        Size::Byte | Size::Word => match src_mode {
                            AddressingMode::DataRegister(_)
                            | AddressingMode::AddressRegister(_) => 14,
                            AddressingMode::Address(_)
                            | AddressingMode::AddressWithPostincrement(_) => 18,
                            AddressingMode::AddressWithPredecrement(_) => 20,
                            AddressingMode::AddressWithDisplacement(_)
                            | AddressingMode::ProgramCounterWithDisplacement
                            | AddressingMode::AbsoluteShort => 22,
                            AddressingMode::AddressWithIndex(_)
                            | AddressingMode::ProgramCounterWithIndex => 24,
                            AddressingMode::AbsoluteLong => 26,
                            AddressingMode::Immediate => 18,
                            _ => panic!(),
                        },
                        Size::Long => match src_mode {
                            AddressingMode::DataRegister(_)
                            | AddressingMode::AddressRegister(_) => 18,
                            AddressingMode::Address(_)
                            | AddressingMode::AddressWithPostincrement(_) => 26,
                            AddressingMode::AddressWithPredecrement(_) => 28,
                            AddressingMode::AddressWithDisplacement(_)
                            | AddressingMode::AbsoluteShort => 30,
                            AddressingMode::AddressWithIndex(_)
                            | AddressingMode::ProgramCounterWithIndex => 32,
                            AddressingMode::AbsoluteLong => 34,
                            AddressingMode::Immediate => 26,
                            _ => panic!(),
                        },
                        Size::Illegal => panic!(),
                    }
                }
                AddressingMode::AbsoluteLong => match size {
                    Size::Byte | Size::Word => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 16,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 20,
                        AddressingMode::AddressWithPredecrement(_) => 22,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 24,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 26,
                        AddressingMode::AbsoluteLong => 28,
                        AddressingMode::Immediate => 20,
                        _ => panic!(),
                    },
                    Size::Long => match src_mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 20,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 28,
                        AddressingMode::AddressWithPredecrement(_) => 30,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 32,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 34,
                        AddressingMode::AbsoluteLong => 36,
                        AddressingMode::Immediate => 28,
                        _ => panic!(),
                    },
                    Size::Illegal => panic!(),
                },
                _ => panic!(),
            },
            Opcode::MOVEA { size, src_mode, .. } => match size {
                Size::Byte | Size::Word => match src_mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 4,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 8,
                    AddressingMode::AddressWithPredecrement(_) => 10,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 12,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 14,
                    AddressingMode::AbsoluteLong => 16,
                    AddressingMode::Immediate => 8,
                    _ => panic!(),
                },
                Size::Long => match src_mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 4,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    AddressingMode::Immediate => 12,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::CLR { size, mode }
            | Opcode::NEGX { size, mode }
            | Opcode::NEG { size, mode }
            | Opcode::NOT { size, mode } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) => 4,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) => 6,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 20,
                    AddressingMode::AddressWithPredecrement(_) => 22,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 24,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 26,
                    AddressingMode::AbsoluteLong => 28,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::MOVE_from_SR { mode } => match mode {
                AddressingMode::DataRegister(_) => 6,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                AddressingMode::AddressWithPredecrement(_) => 14,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 16,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 18,
                AddressingMode::AbsoluteLong => 20,
                _ => panic!(),
            },
            Opcode::MOVE_to_CCR { mode } | Opcode::MOVE_to_SR { mode } => match mode {
                AddressingMode::DataRegister(_) => 12,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 16,
                AddressingMode::AddressWithPredecrement(_) => 18,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 20,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 22,
                AddressingMode::AbsoluteLong => 24,
                AddressingMode::Immediate => 16,
                _ => panic!(),
            },
            Opcode::NBCD { mode } => match mode {
                AddressingMode::DataRegister(_) => 6,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                AddressingMode::AddressWithPredecrement(_) => 14,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 16,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 18,
                AddressingMode::AbsoluteLong => 20,
                _ => panic!(),
            },
            Opcode::PEA { mode } => match mode {
                AddressingMode::Address(_) => 12,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 16,
                AddressingMode::AddressWithIndex(_)
                | AddressingMode::ProgramCounterWithIndex
                | AddressingMode::AbsoluteLong => 20,
                _ => panic!(),
            },
            Opcode::SWAP { .. }
            | Opcode::EXT { .. }
            | Opcode::MOVE_USP { .. }
            | Opcode::NOP
            | Opcode::STOP
            | Opcode::TRAP { .. }
            | Opcode::TRAPV
            | Opcode::ILLEGAL
            | Opcode::MOVEQ { .. } => 4,
            Opcode::MOVEM {
                mode, direction, ..
            } => match direction {
                Direction::RegisterToMemory => match mode {
                    AddressingMode::Address(_) | AddressingMode::AddressWithPredecrement(_) => 8,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 12,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 14,
                    AddressingMode::AbsoluteLong => 16,
                    _ => panic!(),
                },
                Direction::MemoryToRegister => match mode {
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
            },
            Opcode::TST { mode, size } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) => 4,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 8,
                    AddressingMode::AddressWithPredecrement(_) => 10,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 12,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 14,
                    AddressingMode::AbsoluteLong => 16,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) => 4,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::TAS { mode } => match mode {
                AddressingMode::DataRegister(_) => 4,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 8,
                AddressingMode::AddressWithPredecrement(_) => 10,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 12,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 14,
                AddressingMode::AbsoluteLong => 16,
                _ => panic!(),
            },
            Opcode::CHK { mode, .. } => match mode {
                AddressingMode::DataRegister(_) => 10,
                AddressingMode::Address(_)
                | AddressingMode::AddressWithPostincrement(_)
                | AddressingMode::Immediate => 14,
                AddressingMode::AddressWithPredecrement(_) => 16,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 18,
                AddressingMode::AddressWithIndex(_)
                | AddressingMode::ProgramCounterWithIndex
                | AddressingMode::AbsoluteLong => 20,
                _ => panic!(),
            },
            Opcode::LEA { mode, .. } => match mode {
                AddressingMode::Address(_) => 4,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 8,
                AddressingMode::AddressWithIndex(_)
                | AddressingMode::ProgramCounterWithIndex
                | AddressingMode::AbsoluteLong => 12,
                _ => panic!(),
            },
            Opcode::JSR { mode, .. } => match mode {
                AddressingMode::Address(_) => 16,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 18,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 22,
                AddressingMode::AbsoluteLong => 20,
                _ => panic!(),
            },
            Opcode::JMP { mode, .. } => match mode {
                AddressingMode::Address(_) => 8,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 10,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 14,
                AddressingMode::AbsoluteLong => 12,
                _ => panic!(),
            },
            Opcode::RESET => 132,
            Opcode::RTE | Opcode::RTR => 20,
            Opcode::RTS | Opcode::LINK { .. } => 16,
            Opcode::UNLK { .. } => 12,
            Opcode::ADDQ { mode, size, .. } | Opcode::SUBQ { mode, size, .. } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) => 4,
                    AddressingMode::AddressRegister(_) => 8,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) => 8,
                    AddressingMode::AddressRegister(_) => 8,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 20,
                    AddressingMode::AddressWithPredecrement(_) => 22,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 24,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 26,
                    AddressingMode::AbsoluteLong => 28,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::DBcc { .. } => 10,
            Opcode::Scc { mode, .. } => match mode {
                AddressingMode::DataRegister(_) => 4,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                AddressingMode::AddressWithPredecrement(_) => 14,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 16,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 18,
                AddressingMode::AbsoluteLong => 20,
                _ => panic!(),
            },
            Opcode::BRA { .. } | Opcode::Bcc { .. } => 8,
            Opcode::BSR { .. } => 16,
            Opcode::ABCD { operand_mode, .. } | Opcode::SBCD { operand_mode, .. } => {
                match operand_mode {
                    OperandMode::RegisterToRegister => 6,
                    OperandMode::MemoryToMemory => 18,
                }
            }
            Opcode::AND {
                mode,
                size,
                operand_direction,
                ..
            }
            | Opcode::OR {
                mode,
                size,
                operand_direction,
                ..
            }
            | Opcode::ADD {
                mode,
                size,
                operand_direction,
                ..
            }
            | Opcode::SUB {
                mode,
                size,
                operand_direction,
                ..
            } => match operand_direction {
                OperandDirection::ToRegister => match size {
                    Size::Byte | Size::Word => match mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 4,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 8,
                        AddressingMode::AddressWithPredecrement(_) => 10,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 12,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 14,
                        AddressingMode::AbsoluteLong => 16,
                        AddressingMode::Immediate => 8,
                        _ => panic!(),
                    },
                    Size::Long => match mode {
                        AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 8,
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 14,
                        AddressingMode::AddressWithPredecrement(_) => 16,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 18,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 20,
                        AddressingMode::AbsoluteLong => 22,
                        AddressingMode::Immediate => 16,
                        _ => panic!(),
                    },
                    Size::Illegal => panic!(),
                },
                OperandDirection::ToMemory => match size {
                    Size::Byte | Size::Word => match mode {
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 12,
                        AddressingMode::AddressWithPredecrement(_) => 14,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 16,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 18,
                        AddressingMode::AbsoluteLong => 20,
                        _ => panic!(),
                    },
                    Size::Long => match mode {
                        AddressingMode::Address(_)
                        | AddressingMode::AddressWithPostincrement(_) => 20,
                        AddressingMode::AddressWithPredecrement(_) => 22,
                        AddressingMode::AddressWithDisplacement(_)
                        | AddressingMode::ProgramCounterWithDisplacement
                        | AddressingMode::AbsoluteShort => 24,
                        AddressingMode::AddressWithIndex(_)
                        | AddressingMode::ProgramCounterWithIndex => 26,
                        AddressingMode::AbsoluteLong => 28,
                        _ => panic!(),
                    },
                    Size::Illegal => panic!(),
                },
            },
            Opcode::DIVU { mode, .. } => match mode {
                // TODO exact timings
                AddressingMode::DataRegister(_) => 136,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 140,
                AddressingMode::AddressWithPredecrement(_) => 142,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 144,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => {
                    146
                }
                AddressingMode::AbsoluteLong => 148,
                AddressingMode::Immediate => 144,
                _ => panic!(),
            },
            Opcode::DIVS { mode, .. } => match mode {
                // TODO exact timings
                AddressingMode::DataRegister(_) => 156,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 160,
                AddressingMode::AddressWithPredecrement(_) => 162,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 164,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => {
                    166
                }
                AddressingMode::AbsoluteLong => 168,
                AddressingMode::Immediate => 164,
                _ => panic!(),
            },
            Opcode::EXG { .. } => 6,
            Opcode::MULU { mode, .. } | Opcode::MULS { mode, .. } => match mode {
                // TODO exact timings
                AddressingMode::DataRegister(_) => 70,
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 74,
                AddressingMode::AddressWithPredecrement(_) => 76,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 78,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 80,
                AddressingMode::AbsoluteLong => 82,
                AddressingMode::Immediate => 78,
                _ => panic!(),
            },
            Opcode::ADDA { mode, size, .. } | Opcode::SUBA { mode, size, .. } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 8,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    AddressingMode::Immediate => 12,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 8,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 14,
                    AddressingMode::AddressWithPredecrement(_) => 16,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 18,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 20,
                    AddressingMode::AbsoluteLong => 22,
                    AddressingMode::Immediate => 16,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::ADDX {
                operand_mode, size, ..
            }
            | Opcode::SUBX {
                operand_mode, size, ..
            } => match operand_mode {
                OperandMode::RegisterToRegister => match size {
                    Size::Byte | Size::Word => 4,
                    Size::Long => 8,
                    Size::Illegal => panic!(),
                },
                OperandMode::MemoryToMemory => match size {
                    Size::Byte | Size::Word => 18,
                    Size::Long => 30,
                    Size::Illegal => panic!(),
                },
            },
            Opcode::CMP { mode, size, .. } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 4,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 8,
                    AddressingMode::AddressWithPredecrement(_) => 10,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 12,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 14,
                    AddressingMode::AbsoluteLong => 16,
                    AddressingMode::Immediate => 8,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 6,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 14,
                    AddressingMode::AddressWithPredecrement(_) => 16,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 18,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 20,
                    AddressingMode::AbsoluteLong => 22,
                    AddressingMode::Immediate => 14,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::CMPA { mode, size, .. } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 6,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 10,
                    AddressingMode::AddressWithPredecrement(_) => 12,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 14,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 16,
                    AddressingMode::AbsoluteLong => 18,
                    AddressingMode::Immediate => 10,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) | AddressingMode::AddressRegister(_) => 6,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 14,
                    AddressingMode::AddressWithPredecrement(_) => 16,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 18,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 20,
                    AddressingMode::AbsoluteLong => 22,
                    AddressingMode::Immediate => 14,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::CMPM { size, .. } => match size {
                Size::Byte | Size::Word => 12,
                Size::Long => 20,
                Size::Illegal => panic!(),
            },
            Opcode::EOR { size, mode, .. } => match size {
                Size::Byte | Size::Word => match mode {
                    AddressingMode::DataRegister(_) => 4,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                    AddressingMode::AddressWithPredecrement(_) => 14,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 16,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 18,
                    AddressingMode::AbsoluteLong => 20,
                    _ => panic!(),
                },
                Size::Long => match mode {
                    AddressingMode::DataRegister(_) => 8,
                    AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 20,
                    AddressingMode::AddressWithPredecrement(_) => 22,
                    AddressingMode::AddressWithDisplacement(_)
                    | AddressingMode::ProgramCounterWithDisplacement
                    | AddressingMode::AbsoluteShort => 24,
                    AddressingMode::AddressWithIndex(_)
                    | AddressingMode::ProgramCounterWithIndex => 26,
                    AddressingMode::AbsoluteLong => 28,
                    _ => panic!(),
                },
                Size::Illegal => panic!(),
            },
            Opcode::ASL { mode, size, .. }
            | Opcode::ASR { mode, size, .. }
            | Opcode::LSL { mode, size, .. }
            | Opcode::LSR { mode, size, .. }
            | Opcode::ROL { mode, size, .. }
            | Opcode::ROR { mode, size, .. }
            | Opcode::ROXL { mode, size, .. }
            | Opcode::ROXR { mode, size, .. } => match mode {
                AddressingMode::Address(_) | AddressingMode::AddressWithPostincrement(_) => 12,
                AddressingMode::AddressWithPredecrement(_) => 14,
                AddressingMode::AddressWithDisplacement(_)
                | AddressingMode::ProgramCounterWithDisplacement
                | AddressingMode::AbsoluteShort => 16,
                AddressingMode::AddressWithIndex(_) | AddressingMode::ProgramCounterWithIndex => 18,
                AddressingMode::AbsoluteLong => 20,
                _ => match size {
                    Size::Byte | Size::Word => 6,
                    Size::Long => 8,
                    Size::Illegal => panic!(),
                },
            },
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.disassemble(None))
    }
}
