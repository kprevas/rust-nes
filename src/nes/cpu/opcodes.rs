#[derive(Debug)]
pub enum AddressingMode {
    ZeropageX,
    ZeropageY,
    Zeropage,
    AbsoluteIndexedX,
    AbsoluteIndexedY,
    IndexedIndirect,
    IndirectIndexed,
    Accumulator,
    Immediate,
    Absolute,
    Relative,
    Indirect,
    Implied,
}

impl AddressingMode {
    pub fn bytes(&self) -> u8 {
        use self::AddressingMode::*;
        match *self {
            Accumulator | Implied => 0,
            Absolute | AbsoluteIndexedX | AbsoluteIndexedY | Indirect => 2,
            _ => 1,
        }
    }

    pub fn format_operand(&self, operand: u16, pc: u16) -> String {
        use self::AddressingMode::*;
        match *self {
            Immediate => format!("#${:02X} ", operand),
            Zeropage => format!("${:02X} ", operand),
            ZeropageX => format!("${:02X},X ", operand),
            ZeropageY => format!("${:02X},Y ", operand),
            IndexedIndirect => format!("(${:02X},X) ", operand),
            IndirectIndexed => format!("(${:02X}),Y ", operand),
            Absolute => format!("${:04X} ", operand),
            Relative => format!("${:04X} ", {
                let offset = operand as i16;
                if offset >= 0 {
                    pc.wrapping_add(offset as u16)
                } else {
                    pc.wrapping_sub((-offset) as u16)
                }
            }),
            AbsoluteIndexedX => format!("${:04X},X ", operand),
            AbsoluteIndexedY => format!("${:04X},Y ", operand),
            Indirect => format!("(${:04X})", operand),
            _ => String::from(""),
        }
    }
}

#[derive(Debug)]
pub enum Opcode {
    ADC,
    // add with carry
    AHX,
    // store A & X & H
    ALR,
    // and then logical shift right
    ARR,
    // and then rotate right
    ANC,
    // and (with accumulator)
    AND,
    // and (with accumulator)
    ASL,
    // arithmetic shift left
    AXS,
    // A&X minus immediate into X
    BCC,
    // branch on carry clear
    BCS,
    // branch on carry set
    BEQ,
    // branch on equal (zero set)
    BIT,
    // bit test
    BMI,
    // branch on minus (negative set)
    BNE,
    // branch on not equal (zero clear)
    BPL,
    // branch on plus (negative clear)
    BRK,
    // interrupt
    BVC,
    // branch on overflow clear
    BVS,
    // branch on overflow set
    CLC,
    // clear carry
    CLD,
    // clear decimal
    CLI,
    // clear interrupt disable
    CLV,
    // clear overflow
    CMP,
    // compare (with accumulator)
    CPX,
    // compare with X
    CPY,
    // compare with Y
    DCP,
    // decrement then compare
    DEC,
    // decrement
    DEX,
    // decrement X
    DEY,
    // decrement Y
    EOR,
    // exclusive or (with accumulator)
    INC,
    // increment
    INX,
    // increment X
    INY,
    // increment Y
    ISC,
    // increment then subtract
    JMP,
    // jump
    JSR,
    // jump subroutine
    LAS,
    // store memory & stack pointer into A, X, and stack pointer
    LAX,
    // load accumulator and X
    LDA,
    // load accumulator
    LDX,
    // load X
    LDY,
    // load Y
    LSR,
    // logical shift right
    NOP,
    // no operation
    ORA,
    // or with accumulator
    PHA,
    // push accumulator
    PHP,
    // push processor status (SR)
    PLA,
    // pull accumulator
    PLP,
    // pull processor status (SR)
    RLA,
    // rotate left then and
    ROL,
    // rotate left
    ROR,
    // rotate right
    RRA,
    // rotate right then add
    RTI,
    // return from interrupt
    RTS,
    // return from subroutine
    SBC,
    // subtract with carry
    SEC,
    // set carry
    SED,
    // set decimal
    SEI,
    // set interrupt disable
    SAX,
    // store accumulator & X
    SHX,
    // store X & H
    SHY,
    // store Y & H
    SLO,
    // shift left then or
    SRE,
    // shift right then xor
    STA,
    // store accumulator
    STX,
    // store X
    STY,
    // store Y
    TAS,
    // store A & X in stack pointer and A & X & H in memory
    TAX,
    // transfer accumulator to X
    TAY,
    // transfer accumulator to Y
    TSX,
    // transfer stack pointer to X
    TXA,
    // transfer X to accumulator
    TXS,
    // transfer X to stack pointer
    TYA,
    // transfer Y to accumulator
    XAA,
    // transfer X to accumulator then and
    XXX, // unofficial, unimplemented
}

pub const OPCODES: [(Opcode, AddressingMode); 256] = [
    // 0x
    (Opcode::BRK, AddressingMode::Implied),
    (Opcode::ORA, AddressingMode::IndexedIndirect),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::SLO, AddressingMode::IndexedIndirect),
    (Opcode::NOP, AddressingMode::Zeropage),
    (Opcode::ORA, AddressingMode::Zeropage),
    (Opcode::ASL, AddressingMode::Zeropage),
    (Opcode::SLO, AddressingMode::Zeropage),
    (Opcode::PHP, AddressingMode::Implied),
    (Opcode::ORA, AddressingMode::Immediate),
    (Opcode::ASL, AddressingMode::Accumulator),
    (Opcode::ANC, AddressingMode::Immediate),
    (Opcode::NOP, AddressingMode::Absolute),
    (Opcode::ORA, AddressingMode::Absolute),
    (Opcode::ASL, AddressingMode::Absolute),
    (Opcode::SLO, AddressingMode::Absolute),
    // 1x
    (Opcode::BPL, AddressingMode::Relative),
    (Opcode::ORA, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::SLO, AddressingMode::IndirectIndexed),
    (Opcode::NOP, AddressingMode::ZeropageX),
    (Opcode::ORA, AddressingMode::ZeropageX),
    (Opcode::ASL, AddressingMode::ZeropageX),
    (Opcode::SLO, AddressingMode::ZeropageX),
    (Opcode::CLC, AddressingMode::Implied),
    (Opcode::ORA, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::Implied),
    (Opcode::SLO, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::AbsoluteIndexedX),
    (Opcode::ORA, AddressingMode::AbsoluteIndexedX),
    (Opcode::ASL, AddressingMode::AbsoluteIndexedX),
    (Opcode::SLO, AddressingMode::AbsoluteIndexedX),
    // 2x
    (Opcode::JSR, AddressingMode::Absolute),
    (Opcode::AND, AddressingMode::IndexedIndirect),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::RLA, AddressingMode::IndexedIndirect),
    (Opcode::BIT, AddressingMode::Zeropage),
    (Opcode::AND, AddressingMode::Zeropage),
    (Opcode::ROL, AddressingMode::Zeropage),
    (Opcode::RLA, AddressingMode::Zeropage),
    (Opcode::PLP, AddressingMode::Implied),
    (Opcode::AND, AddressingMode::Immediate),
    (Opcode::ROL, AddressingMode::Accumulator),
    (Opcode::ANC, AddressingMode::Immediate),
    (Opcode::BIT, AddressingMode::Absolute),
    (Opcode::AND, AddressingMode::Absolute),
    (Opcode::ROL, AddressingMode::Absolute),
    (Opcode::RLA, AddressingMode::Absolute),
    // 3x
    (Opcode::BMI, AddressingMode::Relative),
    (Opcode::AND, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::RLA, AddressingMode::IndirectIndexed),
    (Opcode::NOP, AddressingMode::ZeropageX),
    (Opcode::AND, AddressingMode::ZeropageX),
    (Opcode::ROL, AddressingMode::ZeropageX),
    (Opcode::RLA, AddressingMode::ZeropageX),
    (Opcode::SEC, AddressingMode::Implied),
    (Opcode::AND, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::Implied),
    (Opcode::RLA, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::AbsoluteIndexedX),
    (Opcode::AND, AddressingMode::AbsoluteIndexedX),
    (Opcode::ROL, AddressingMode::AbsoluteIndexedX),
    (Opcode::RLA, AddressingMode::AbsoluteIndexedX),
    // 4x
    (Opcode::RTI, AddressingMode::Implied),
    (Opcode::EOR, AddressingMode::IndexedIndirect),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::SRE, AddressingMode::IndexedIndirect),
    (Opcode::NOP, AddressingMode::Zeropage),
    (Opcode::EOR, AddressingMode::Zeropage),
    (Opcode::LSR, AddressingMode::Zeropage),
    (Opcode::SRE, AddressingMode::Zeropage),
    (Opcode::PHA, AddressingMode::Implied),
    (Opcode::EOR, AddressingMode::Immediate),
    (Opcode::LSR, AddressingMode::Accumulator),
    (Opcode::ALR, AddressingMode::Immediate),
    (Opcode::JMP, AddressingMode::Absolute),
    (Opcode::EOR, AddressingMode::Absolute),
    (Opcode::LSR, AddressingMode::Absolute),
    (Opcode::SRE, AddressingMode::Absolute),
    // 5x
    (Opcode::BVC, AddressingMode::Relative),
    (Opcode::EOR, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::SRE, AddressingMode::IndirectIndexed),
    (Opcode::NOP, AddressingMode::ZeropageX),
    (Opcode::EOR, AddressingMode::ZeropageX),
    (Opcode::LSR, AddressingMode::ZeropageX),
    (Opcode::SRE, AddressingMode::ZeropageX),
    (Opcode::CLI, AddressingMode::Implied),
    (Opcode::EOR, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::Implied),
    (Opcode::SRE, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::AbsoluteIndexedX),
    (Opcode::EOR, AddressingMode::AbsoluteIndexedX),
    (Opcode::LSR, AddressingMode::AbsoluteIndexedX),
    (Opcode::SRE, AddressingMode::AbsoluteIndexedX),
    // 6x
    (Opcode::RTS, AddressingMode::Implied),
    (Opcode::ADC, AddressingMode::IndexedIndirect),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::RRA, AddressingMode::IndexedIndirect),
    (Opcode::NOP, AddressingMode::Zeropage),
    (Opcode::ADC, AddressingMode::Zeropage),
    (Opcode::ROR, AddressingMode::Zeropage),
    (Opcode::RRA, AddressingMode::Zeropage),
    (Opcode::PLA, AddressingMode::Implied),
    (Opcode::ADC, AddressingMode::Immediate),
    (Opcode::ROR, AddressingMode::Accumulator),
    (Opcode::ARR, AddressingMode::Immediate),
    (Opcode::JMP, AddressingMode::Indirect),
    (Opcode::ADC, AddressingMode::Absolute),
    (Opcode::ROR, AddressingMode::Absolute),
    (Opcode::RRA, AddressingMode::Absolute),
    // 7x
    (Opcode::BVS, AddressingMode::Relative),
    (Opcode::ADC, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::RRA, AddressingMode::IndirectIndexed),
    (Opcode::NOP, AddressingMode::ZeropageX),
    (Opcode::ADC, AddressingMode::ZeropageX),
    (Opcode::ROR, AddressingMode::ZeropageX),
    (Opcode::RRA, AddressingMode::ZeropageX),
    (Opcode::SEI, AddressingMode::Implied),
    (Opcode::ADC, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::Implied),
    (Opcode::RRA, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::AbsoluteIndexedX),
    (Opcode::ADC, AddressingMode::AbsoluteIndexedX),
    (Opcode::ROR, AddressingMode::AbsoluteIndexedX),
    (Opcode::RRA, AddressingMode::AbsoluteIndexedX),
    // 8x
    (Opcode::NOP, AddressingMode::Immediate),
    (Opcode::STA, AddressingMode::IndexedIndirect),
    (Opcode::NOP, AddressingMode::Immediate),
    (Opcode::SAX, AddressingMode::IndexedIndirect),
    (Opcode::STY, AddressingMode::Zeropage),
    (Opcode::STA, AddressingMode::Zeropage),
    (Opcode::STX, AddressingMode::Zeropage),
    (Opcode::SAX, AddressingMode::Zeropage),
    (Opcode::DEY, AddressingMode::Implied),
    (Opcode::NOP, AddressingMode::Immediate),
    (Opcode::TXA, AddressingMode::Implied),
    (Opcode::XAA, AddressingMode::Immediate),
    (Opcode::STY, AddressingMode::Absolute),
    (Opcode::STA, AddressingMode::Absolute),
    (Opcode::STX, AddressingMode::Absolute),
    (Opcode::SAX, AddressingMode::Absolute),
    // 9x
    (Opcode::BCC, AddressingMode::Relative),
    (Opcode::STA, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::AHX, AddressingMode::IndirectIndexed),
    (Opcode::STY, AddressingMode::ZeropageX),
    (Opcode::STA, AddressingMode::ZeropageX),
    (Opcode::STX, AddressingMode::ZeropageY),
    (Opcode::SAX, AddressingMode::ZeropageY),
    (Opcode::TYA, AddressingMode::Implied),
    (Opcode::STA, AddressingMode::AbsoluteIndexedY),
    (Opcode::TXS, AddressingMode::Implied),
    (Opcode::TAS, AddressingMode::AbsoluteIndexedY),
    (Opcode::SHY, AddressingMode::AbsoluteIndexedX),
    (Opcode::STA, AddressingMode::AbsoluteIndexedX),
    (Opcode::SHX, AddressingMode::AbsoluteIndexedY),
    (Opcode::AHX, AddressingMode::AbsoluteIndexedY),
    // Ax
    (Opcode::LDY, AddressingMode::Immediate),
    (Opcode::LDA, AddressingMode::IndexedIndirect),
    (Opcode::LDX, AddressingMode::Immediate),
    (Opcode::LAX, AddressingMode::IndexedIndirect),
    (Opcode::LDY, AddressingMode::Zeropage),
    (Opcode::LDA, AddressingMode::Zeropage),
    (Opcode::LDX, AddressingMode::Zeropage),
    (Opcode::LAX, AddressingMode::Zeropage),
    (Opcode::TAY, AddressingMode::Implied),
    (Opcode::LDA, AddressingMode::Immediate),
    (Opcode::TAX, AddressingMode::Implied),
    (Opcode::LAX, AddressingMode::Immediate),
    (Opcode::LDY, AddressingMode::Absolute),
    (Opcode::LDA, AddressingMode::Absolute),
    (Opcode::LDX, AddressingMode::Absolute),
    (Opcode::LAX, AddressingMode::Absolute),
    // Bx
    (Opcode::BCS, AddressingMode::Relative),
    (Opcode::LDA, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::LAX, AddressingMode::IndirectIndexed),
    (Opcode::LDY, AddressingMode::ZeropageX),
    (Opcode::LDA, AddressingMode::ZeropageX),
    (Opcode::LDX, AddressingMode::ZeropageY),
    (Opcode::LAX, AddressingMode::ZeropageY),
    (Opcode::CLV, AddressingMode::Implied),
    (Opcode::LDA, AddressingMode::AbsoluteIndexedY),
    (Opcode::TSX, AddressingMode::Implied),
    (Opcode::LAS, AddressingMode::AbsoluteIndexedY),
    (Opcode::LDY, AddressingMode::AbsoluteIndexedX),
    (Opcode::LDA, AddressingMode::AbsoluteIndexedX),
    (Opcode::LDX, AddressingMode::AbsoluteIndexedY),
    (Opcode::LAX, AddressingMode::AbsoluteIndexedY),
    // Cx
    (Opcode::CPY, AddressingMode::Immediate),
    (Opcode::CMP, AddressingMode::IndexedIndirect),
    (Opcode::NOP, AddressingMode::Immediate),
    (Opcode::DCP, AddressingMode::IndexedIndirect),
    (Opcode::CPY, AddressingMode::Zeropage),
    (Opcode::CMP, AddressingMode::Zeropage),
    (Opcode::DEC, AddressingMode::Zeropage),
    (Opcode::DCP, AddressingMode::Zeropage),
    (Opcode::INY, AddressingMode::Implied),
    (Opcode::CMP, AddressingMode::Immediate),
    (Opcode::DEX, AddressingMode::Implied),
    (Opcode::AXS, AddressingMode::Immediate),
    (Opcode::CPY, AddressingMode::Absolute),
    (Opcode::CMP, AddressingMode::Absolute),
    (Opcode::DEC, AddressingMode::Absolute),
    (Opcode::DCP, AddressingMode::Absolute),
    // Dx
    (Opcode::BNE, AddressingMode::Relative),
    (Opcode::CMP, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::DCP, AddressingMode::IndirectIndexed),
    (Opcode::NOP, AddressingMode::ZeropageX),
    (Opcode::CMP, AddressingMode::ZeropageX),
    (Opcode::DEC, AddressingMode::ZeropageX),
    (Opcode::DCP, AddressingMode::ZeropageX),
    (Opcode::CLD, AddressingMode::Implied),
    (Opcode::CMP, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::Implied),
    (Opcode::DCP, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::AbsoluteIndexedX),
    (Opcode::CMP, AddressingMode::AbsoluteIndexedX),
    (Opcode::DEC, AddressingMode::AbsoluteIndexedX),
    (Opcode::DCP, AddressingMode::AbsoluteIndexedX),
    // Ex
    (Opcode::CPX, AddressingMode::Immediate),
    (Opcode::SBC, AddressingMode::IndexedIndirect),
    (Opcode::NOP, AddressingMode::Immediate),
    (Opcode::ISC, AddressingMode::IndexedIndirect),
    (Opcode::CPX, AddressingMode::Zeropage),
    (Opcode::SBC, AddressingMode::Zeropage),
    (Opcode::INC, AddressingMode::Zeropage),
    (Opcode::ISC, AddressingMode::Zeropage),
    (Opcode::INX, AddressingMode::Implied),
    (Opcode::SBC, AddressingMode::Immediate),
    (Opcode::NOP, AddressingMode::Implied),
    (Opcode::SBC, AddressingMode::Immediate),
    (Opcode::CPX, AddressingMode::Absolute),
    (Opcode::SBC, AddressingMode::Absolute),
    (Opcode::INC, AddressingMode::Absolute),
    (Opcode::ISC, AddressingMode::Absolute),
    // Fx
    (Opcode::BEQ, AddressingMode::Relative),
    (Opcode::SBC, AddressingMode::IndirectIndexed),
    (Opcode::XXX, AddressingMode::Implied),
    (Opcode::ISC, AddressingMode::IndirectIndexed),
    (Opcode::NOP, AddressingMode::ZeropageX),
    (Opcode::SBC, AddressingMode::ZeropageX),
    (Opcode::INC, AddressingMode::ZeropageX),
    (Opcode::ISC, AddressingMode::ZeropageX),
    (Opcode::SED, AddressingMode::Implied),
    (Opcode::SBC, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::Implied),
    (Opcode::ISC, AddressingMode::AbsoluteIndexedY),
    (Opcode::NOP, AddressingMode::AbsoluteIndexedX),
    (Opcode::SBC, AddressingMode::AbsoluteIndexedX),
    (Opcode::INC, AddressingMode::AbsoluteIndexedX),
    (Opcode::ISC, AddressingMode::AbsoluteIndexedX),
];
