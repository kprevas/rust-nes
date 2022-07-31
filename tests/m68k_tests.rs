extern crate emu;
extern crate itertools;
extern crate json;

use std::cell::RefCell;

use itertools::{Itertools, Tuples};
use json::iterators::Members;
use json::JsonValue;

use emu::gen::m68k::opcodes::Opcode;
use emu::gen::vdp::bus::VdpBus;
use emu::gen::vdp::Vdp;

#[test]
fn opcode_decoding() {
    let test_cases = json::parse(include_str!("m68k/68000ops.json"));
    for (opcode_str, expected_val) in test_cases.unwrap().entries() {
        let expected = expected_val.as_str().unwrap();
        let opcode_hex = u16::from_str_radix(opcode_str, 16).unwrap();
        let opcode = emu::gen::m68k::opcodes::opcode(opcode_hex);
        assert_eq!(
            format!("{}", opcode),
            expected,
            "{:04X} {:016b}",
            opcode_hex,
            opcode_hex
        );
    }
}

#[test]
fn abcd_sbcd() {
    run_json_test(json::parse(include_str!("m68k/abcd_sbcd.json")).unwrap());
}

#[test]
fn add_sub() {
    run_json_test(json::parse(include_str!("m68k/add_sub.json")).unwrap());
}

#[test]
fn addi_subi_cmpi() {
    run_json_test(json::parse(include_str!("m68k/addi_subi_cmpi.json")).unwrap());
}

#[test]
fn addq_subq() {
    run_json_test(json::parse(include_str!("m68k/addq_subq.json")).unwrap());
}

#[test]
fn addx_subx() {
    run_json_test(json::parse(include_str!("m68k/addx_subx.json")).unwrap());
}

#[test]
fn bcc() {
    run_json_test(json::parse(include_str!("m68k/bcc.json")).unwrap());
}

#[test]
fn btst_bchg_bclr_bset() {
    run_json_test(json::parse(include_str!("m68k/btst_bchg_bclr_bset.json")).unwrap());
}

#[test]
fn chk() {
    run_json_test(json::parse(include_str!("m68k/chk.json")).unwrap());
}

#[test]
fn cmp() {
    run_json_test(json::parse(include_str!("m68k/cmp.json")).unwrap());
}

#[test]
fn divu_divs() {
    run_json_test(json::parse(include_str!("m68k/divu_divs.json")).unwrap());
}

#[test]
fn dbcc_scc() {
    run_json_test(json::parse(include_str!("m68k/dbcc_scc.json")).unwrap());
}

#[test]
fn eor_and_or() {
    run_json_test(json::parse(include_str!("m68k/eor_and_or.json")).unwrap());
}

#[test]
fn eori_andi_ori() {
    run_json_test(json::parse(include_str!("m68k/eori_andi_ori.json")).unwrap());
}

#[test]
fn exg() {
    run_json_test(json::parse(include_str!("m68k/exg.json")).unwrap());
}

#[test]
fn ext() {
    run_json_test(json::parse(include_str!("m68k/ext.json")).unwrap());
}

#[test]
fn jmp_jsr() {
    run_json_test(json::parse(include_str!("m68k/jmp_jsr.json")).unwrap());
}

#[test]
fn lea() {
    run_json_test(json::parse(include_str!("m68k/lea.json")).unwrap());
}

#[test]
fn link_unlk() {
    run_json_test(json::parse(include_str!("m68k/link_unlk.json")).unwrap());
}

#[test]
fn lslr_aslr_roxlr_rolr() {
    run_json_test(json::parse(include_str!("m68k/lslr_aslr_roxlr_rolr.json")).unwrap());
}

#[test]
fn move_() {
    run_json_test(json::parse(include_str!("m68k/move.json")).unwrap());
}

#[test]
fn movem() {
    run_json_test(json::parse(include_str!("m68k/movem.json")).unwrap());
}

#[test]
fn movep() {
    run_json_test(json::parse(include_str!("m68k/movep.json")).unwrap());
}

#[test]
fn moveq() {
    run_json_test(json::parse(include_str!("m68k/moveq.json")).unwrap());
}

#[test]
fn move_tofrom_srccr() {
    run_json_test(json::parse(include_str!("m68k/move_tofrom_srccr.json")).unwrap());
}

#[test]
fn mulu_muls() {
    run_json_test(json::parse(include_str!("m68k/mulu_muls.json")).unwrap());
}

#[test]
fn nbcd_pea() {
    run_json_test(json::parse(include_str!("m68k/nbcd_pea.json")).unwrap());
}

#[test]
fn neg_not() {
    run_json_test(json::parse(include_str!("m68k/neg_not.json")).unwrap());
}

#[test]
fn negx_clr() {
    run_json_test(json::parse(include_str!("m68k/negx_clr.json")).unwrap());
}

#[test]
fn swap() {
    run_json_test(json::parse(include_str!("m68k/swap.json")).unwrap());
}

#[test]
fn rtr() {
    run_json_test(json::parse(include_str!("m68k/rtr.json")).unwrap());
}

#[test]
fn rts() {
    run_json_test(json::parse(include_str!("m68k/rts.json")).unwrap());
}

#[test]
fn tas() {
    run_json_test(json::parse(include_str!("m68k/tas.json")).unwrap());
}

#[test]
fn tst() {
    run_json_test(json::parse(include_str!("m68k/tst.json")).unwrap());
}

fn run_json_test(test_cases: JsonValue) {
    for test_case in test_cases.members() {
        if !test_case.has_key("name") {
            continue;
        }
        let cartridge = vec![0; 8].into_boxed_slice();
        let vdp_bus = RefCell::new(VdpBus::new());
        let mut cpu = emu::gen::m68k::Cpu::boot(&cartridge, Vdp::new(&vdp_bus), &vdp_bus, true);
        cpu.expand_ram(0x1000000);
        cpu.reset(false);
        let initial_state = &test_case["initial state"];
        cpu.init_state(
            initial_state["pc"].as_u32().unwrap(),
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
            cpu.poke_ram(addr.as_u32().unwrap(), val.as_u8().unwrap());
        }
        let sr_mask = match cpu.peek_opcode() {
            Opcode::CHK { .. } => 0b1111111111111000,
            Opcode::ABCD { .. } | Opcode::NBCD { .. } | Opcode::SBCD { .. } => 0b1111111111110101,
            _ => 0b1111111111111111,
        };
        if let Opcode::ILLEGAL = cpu.peek_opcode() {
            continue;
        }
        cpu.next_operation(&[emu::input::player_1_gen(), emu::input::player_2_gen()]);
        let final_state = &test_case["final state"];
        let test_id = format!(
            "{}  {}",
            test_case["name"].as_str().unwrap(),
            cpu.peek_opcode()
        );
        cpu.verify_state(
            final_state["pc"].as_u32().unwrap(),
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
            sr_mask,
            &test_id,
        );
        let final_memory: Tuples<Members, (&JsonValue, &JsonValue)> =
            test_case["final memory"].members().tuples();
        for (addr, val) in final_memory {
            if addr.as_i32().unwrap_or(-1) == -1 {
                break;
            }
            cpu.verify_ram(addr.as_u32().unwrap(), val.as_u8().unwrap(), &test_id);
        }
    }
}

#[test]
fn test_all_opcodes() {
    let _ = env_logger::try_init();
    let rom = include_bytes!("m68k/opcodes.bin");
    let cartridge = vec![0; 8].into_boxed_slice();
    let vdp_bus = RefCell::new(VdpBus::new());
    let mut cpu = emu::gen::m68k::Cpu::boot(&cartridge, Vdp::new(&vdp_bus), &vdp_bus, true);
    cpu.set_ram(rom);
    cpu.reset(false);
    let mut last_pc = 0;
    let mut loop_count = 0;
    while loop_count < 5 {
        cpu.next_operation(&[emu::input::player_1_gen(), emu::input::player_2_gen()]);
        if last_pc == cpu.pc_for_test() {
            loop_count += 1;
        } else {
            loop_count = 0;
            last_pc = cpu.pc_for_test();
        }
    }
    assert_eq!(0xF000, cpu.pc_for_test());
}

#[test]
fn test_bcd_verifier() {
    let _ = env_logger::try_init();
    let cartridge = include_bytes!("m68k/bcd-verifier-u1.bin")
        .to_vec()
        .into_boxed_slice();
    let vdp_bus = RefCell::new(VdpBus::new());
    let mut cpu = emu::gen::m68k::Cpu::boot(&cartridge, Vdp::new(&vdp_bus), &vdp_bus, true);
    cpu.reset(false);
    while cpu.pc_for_test() != 0x123a {
        cpu.next_operation(&[emu::input::player_1_gen(), emu::input::player_2_gen()]);
    }
    assert_eq!(0, cpu.peek_ram_long(0xFFFFFF00), "ABCD flags");
    assert_eq!(0, cpu.peek_ram_long(0xFFFFFF04), "ABCD values");
    assert_eq!(0, cpu.peek_ram_long(0xFFFFFF08), "SBCD flags");
    assert_eq!(0, cpu.peek_ram_long(0xFFFFFF0c), "SBCD values");
    assert_eq!(0, cpu.peek_ram_long(0xFFFFFF10), "NBCD flags");
    assert_eq!(0, cpu.peek_ram_long(0xFFFFFF14), "NBCD values");
}
