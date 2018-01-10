use std::io::prelude::*;
use std::error::Error;
use cartridge::Cartridge;

pub fn disassemble(cartridge: Box<Cartridge>, start: u16, out: &mut Write) -> Result<(), Box<Error>> {
    use super::opcodes::OPCODES;

    let mut pc = start;

    loop {
        let opcode_hex = cartridge.read_memory(pc);

        let (ref opcode, ref mode) = OPCODES[usize::from(opcode_hex)];
        write!(out, "{:04X}\t{:02X} ", pc, opcode_hex)?;

        pc += 1;

        let mut operand = 0u16;
        let mut shift = 0;
        for _ in 0..mode.bytes() {
            let operand_byte = cartridge.read_memory(pc);
            operand += u16::from(operand_byte) << shift;
            shift += 8;
            write!(out, "{:02X} ", operand_byte)?;
            pc += 1;
        }

        write!(out, "\t{:?} {}\n", opcode, mode.format_operand(operand, pc))?;
        if pc >= 0xffff {
            break;
        }
    }
    Ok(())
}