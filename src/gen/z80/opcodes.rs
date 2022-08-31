#[derive(Debug, Copy, Clone)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterPair {
    BC,
    DE,
    HL,
}

#[derive(Debug, Copy, Clone)]
pub enum IndexRegister {
    IX,
    IY,
}

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    Immediate,
    Relative,
    Extended,
    Indexed(IndexRegister),
    Register(Register),
    RegisterIndirect(RegisterPair),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum Opcode {
    NOP,
}

pub const OPCODES: [Opcode; 256] = [Opcode::NOP; 256];