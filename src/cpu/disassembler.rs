use cartridge::CartridgeBus;
use std::error::Error;
use std::io::prelude::*;

pub fn disassemble(cartridge: Box<CartridgeBus>, start: u16, out: &mut Write) -> Result<(), Box<Error>> {
    use super::opcodes::OPCODES;

    let mut pc = start;

    loop {
        let opcode_hex = cartridge.read_memory(pc, 0);

        let (ref opcode, ref mode) = OPCODES[usize::from(opcode_hex)];
        write!(out, "{:04X}\t{:02X} ", pc, opcode_hex)?;

        pc += 1;

        let mut operand = 0u16;
        let mut shift = 0;
        for _ in 0..mode.bytes() {
            let operand_byte = cartridge.read_memory(pc, 0);
            operand += u16::from(operand_byte) << shift;
            shift += 8;
            write!(out, "{:02X} ", operand_byte)?;
            if pc < 0xffff {
                pc += 1;
            }
        }

        write!(out, "\t{:?} {}\n", opcode, mode.format_operand(operand, pc))?;
        if pc >= 0xffff {
            break;
        }
    }
    Ok(())
}