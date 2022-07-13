extern crate itertools;
extern crate json;
extern crate nes;

use itertools::{Itertools, Tuples};
use json::iterators::Members;
use json::JsonValue;

use nes::input::ControllerState;
use nes::m68k::opcodes::Opcode;

#[test]
fn opcode_decoding() {
    let test_cases = json::parse(include_str!("m68k/68000ops.json"));
    for (opcode_str, expected_val) in test_cases.unwrap().entries() {
        let expected = expected_val.as_str().unwrap();
        let opcode_hex = u16::from_str_radix(opcode_str, 16).unwrap();
        let opcode = nes::m68k::opcodes::opcode(opcode_hex);
        assert_eq!(format!("{}", opcode), expected, "{:04X} {:016b}", opcode_hex, opcode_hex);
    }
}

#[test]
fn jmp_jsr() {
    run_json_test(json::parse(include_str!("m68k/jmp_jsr.json")).unwrap());
}

#[test]
fn move_() {
    run_json_test(json::parse(include_str!("m68k/move.json")).unwrap());
}

fn run_json_test(test_cases: JsonValue) {
    for test_case in test_cases.members() {
        if !test_case.has_key("name") { continue; }
        println!("{}", test_case["name"].as_str().unwrap());
        let mut cpu = nes::m68k::Cpu::boot(true);
        cpu.expand_ram(0x1000000);
        cpu.reset(false);
        let initial_state = &test_case["initial state"];
        cpu.init_state(initial_state["pc"].as_u32().unwrap(),
                       initial_state["sr"].as_u16().unwrap(),
                       [
                           initial_state["d0"].as_u32().unwrap(),
                           initial_state["d1"].as_u32().unwrap(),
                           initial_state["d2"].as_u32().unwrap(),
                           initial_state["d3"].as_u32().unwrap(),
                           initial_state["d4"].as_u32().unwrap(),
                           initial_state["d5"].as_u32().unwrap(),
                           initial_state["d6"].as_u32().unwrap(),
                           initial_state["d7"].as_u32().unwrap(),
                       ],
                       [
                           initial_state["a0"].as_u32().unwrap(),
                           initial_state["a1"].as_u32().unwrap(),
                           initial_state["a2"].as_u32().unwrap(),
                           initial_state["a3"].as_u32().unwrap(),
                           initial_state["a4"].as_u32().unwrap(),
                           initial_state["a5"].as_u32().unwrap(),
                           initial_state["a6"].as_u32().unwrap(),
                           initial_state["usp"].as_u32().unwrap(),
                       ],
                       initial_state["a7"].as_u32().unwrap(),
        );
        let initial_memory: Tuples<Members, (&JsonValue, &JsonValue)> =
            test_case["initial memory"].members().tuples();
        for (addr, val) in initial_memory {
            if addr.as_i32().unwrap_or(-1) == -1 {
                break;
            }
            cpu.poke_ram(addr.as_usize().unwrap(), val.as_u8().unwrap());
        }
        if let Opcode::ILLEGAL = cpu.peek_opcode() { continue; }
        if let
        Opcode::ADDI { .. }
        | Opcode::ANDI { .. }
        | Opcode::ANDI_to_CCR { .. }
        | Opcode::ANDI_to_SR { .. }
        | Opcode::BCHG { .. }
        | Opcode::BCLR { .. }
        | Opcode::BSET { .. }
        | Opcode::BTST { .. }
        | Opcode::CMPI { .. }
        | Opcode::EORI { .. }
        | Opcode::EORI_to_CCR { .. }
        | Opcode::EORI_to_SR { .. }
        | Opcode::MOVEP { .. }
        | Opcode::ORI { .. }
        | Opcode::ORI_to_CCR { .. }
        | Opcode::ORI_to_SR { .. }
        | Opcode::SUBI { .. }
        = cpu.peek_opcode() { continue; } // TODO
        println!("  {}", cpu.peek_opcode());
        cpu.next_operation(&[nes::input::player_1_nes(), nes::input::player_2_nes()]);
        let final_state = &test_case["final state"];
        cpu.verify_state(final_state["pc"].as_u32().unwrap(),
                         final_state["sr"].as_u16().unwrap(),
                         [
                             final_state["d0"].as_u32().unwrap(),
                             final_state["d1"].as_u32().unwrap(),
                             final_state["d2"].as_u32().unwrap(),
                             final_state["d3"].as_u32().unwrap(),
                             final_state["d4"].as_u32().unwrap(),
                             final_state["d5"].as_u32().unwrap(),
                             final_state["d6"].as_u32().unwrap(),
                             final_state["d7"].as_u32().unwrap(),
                         ],
                         [
                             final_state["a0"].as_u32().unwrap(),
                             final_state["a1"].as_u32().unwrap(),
                             final_state["a2"].as_u32().unwrap(),
                             final_state["a3"].as_u32().unwrap(),
                             final_state["a4"].as_u32().unwrap(),
                             final_state["a5"].as_u32().unwrap(),
                             final_state["a6"].as_u32().unwrap(),
                             final_state["usp"].as_u32().unwrap(),
                         ],
                         final_state["a7"].as_u32().unwrap(),
        );
        let final_memory: Tuples<Members, (&JsonValue, &JsonValue)> =
            test_case["final memory"].members().tuples();
        for (addr, val) in final_memory {
            if addr.as_i32().unwrap_or(-1) == -1 {
                break;
            }
            cpu.verify_ram(addr.as_usize().unwrap(), val.as_u8().unwrap());
        }
    }
}