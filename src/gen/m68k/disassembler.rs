use std::error::Error;
use std::io::prelude::*;

use gen::m68k::opcodes::opcode;

pub fn disassemble(cartridge: Box<[u8]>, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    let mut pc = 0x200;

    let boot = ((cartridge[4] as u32) << 24)
        | ((cartridge[5] as u32) << 16)
        | ((cartridge[6] as u32) << 8)
        | cartridge[7] as u32;

    let hblank = ((cartridge[112] as u32) << 24)
        | ((cartridge[113] as u32) << 16)
        | ((cartridge[114] as u32) << 8)
        | cartridge[115] as u32;
    let vblank = ((cartridge[120] as u32) << 24)
        | ((cartridge[121] as u32) << 16)
        | ((cartridge[122] as u32) << 8)
        | cartridge[123] as u32;

    loop {
        let opcode_hex =
            ((cartridge[pc as usize] as u16) << 8) as u16 | cartridge[pc as usize + 1] as u16;
        let opcode = opcode(opcode_hex);

        if pc == boot {
            writeln!(out, "BOOT:")?;
        } else if pc == hblank {
            writeln!(out, "HBLANK:")?;
        } else if pc == vblank {
            writeln!(out, "VBLANK:")?;
        }

        writeln!(
            out,
            "{:06X}\t{:04X} {}",
            pc,
            opcode_hex,
            opcode.disassemble(Some(&cartridge[pc as usize + 2..]), Some(pc))
        )?;

        pc += 2 + opcode.extension_bytes() as u32;

        if pc >= 0x3FFFFF || pc >= cartridge.len() as u32 {
            break;
        }
    }
    Ok(())
}
