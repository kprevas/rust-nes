extern crate emu;
extern crate itertools;
extern crate json;

use itertools::Itertools;
use json::JsonValue;

use emu::gen::z80::Cpu;

#[test]
fn prelim() {
    run_zex_test(include_bytes!("z80/prelim.com"), 0x44A);
}

#[test]
#[ignore]
fn zexdoc() {
    run_zex_test(include_bytes!("z80/zexdoc.cim"), 0x1DF9);
}

fn run_zex_test(ram: &[u8], success_msg_addr: u16) {
    let _ = env_logger::try_init();
    let cartridge = vec![].into_boxed_slice();
    let mut cpu = Cpu::new(&cartridge, true);
    cpu.set_pc(0x100);
    cpu.load_ram(0x100, ram);
    cpu.init_zex_test_vectors();
    let mut output = false;
    while cpu.get_pc() != 0 && !cpu.stopped && cpu.get_cycle_count() < 46734978649 {
        if cpu.get_pc() == 5 {
            if !output {
                cpu.output_test_string();
                output = true;
            }
        } else {
            output = false;
        }
        cpu.step()
    }
    assert_eq!(cpu.get_pc(), 0);
    assert_eq!(cpu.get_de(), success_msg_addr)
}

#[test]
#[ignore]
pub fn json_tests() {
    let initials = json::parse(include_str!("z80/tests.in.json"));
    let expecteds = json::parse(include_str!("z80/tests.expected.json"));

    for (initial, expected) in initials
        .unwrap()
        .members()
        .zip(expecteds.unwrap().members())
    {
        run_json_test(initial, expected);
    }
}

fn run_json_test(initial: &JsonValue, expected: &JsonValue) {
    let _ = env_logger::try_init();
    let cartridge = vec![].into_boxed_slice();
    let mut cpu = Cpu::new(&cartridge, true);
    let ram = vec![0; 0x10000].into_boxed_slice();
    cpu.load_ram(0, &ram);

    let initial_state = &initial["state"];
    cpu.init_state(
        [
            initial_state["af"].as_u16().unwrap(),
            initial_state["afDash"].as_u16().unwrap(),
        ],
        [
            initial_state["bc"].as_u16().unwrap(),
            initial_state["bcDash"].as_u16().unwrap(),
        ],
        [
            initial_state["de"].as_u16().unwrap(),
            initial_state["deDash"].as_u16().unwrap(),
        ],
        [
            initial_state["hl"].as_u16().unwrap(),
            initial_state["hlDash"].as_u16().unwrap(),
        ],
        initial_state["ix"].as_u16().unwrap(),
        initial_state["iy"].as_u16().unwrap(),
        initial_state["sp"].as_u16().unwrap(),
        initial_state["pc"].as_u16().unwrap(),
        initial_state["i"].as_u8().unwrap(),
        initial_state["r"].as_u8().unwrap(),
        initial_state["iff1"].as_bool().unwrap(),
    );
    for mem in initial["memory"].members() {
        cpu.poke_ram(
            mem["address"].as_usize().unwrap(),
            &mem["data"].members().map(|d| d.as_u8().unwrap()).collect_vec(),
        );
    }
    let test_id = format!("{} - {:?}", initial["name"].as_str().unwrap(), cpu.peek_opcode());

    cpu.step();

    let expected_state = &expected["state"];
    cpu.verify_state(
        [
            expected_state["af"].as_u16().unwrap(),
            expected_state["afDash"].as_u16().unwrap(),
        ],
        [
            expected_state["bc"].as_u16().unwrap(),
            expected_state["bcDash"].as_u16().unwrap(),
        ],
        [
            expected_state["de"].as_u16().unwrap(),
            expected_state["deDash"].as_u16().unwrap(),
        ],
        [
            expected_state["hl"].as_u16().unwrap(),
            expected_state["hlDash"].as_u16().unwrap(),
        ],
        expected_state["ix"].as_u16().unwrap(),
        expected_state["iy"].as_u16().unwrap(),
        expected_state["sp"].as_u16().unwrap(),
        expected_state["pc"].as_u16().unwrap(),
        expected_state["i"].as_u8().unwrap(),
        expected_state["r"].as_u8().unwrap(),
        expected_state["iff1"].as_bool().unwrap(),
        &test_id,
    );
    for mem in expected["memory"].members() {
        cpu.verify_ram(
            mem["address"].as_usize().unwrap(),
            &mem["data"].members().map(|d| d.as_u8().unwrap()).collect_vec(),
            &test_id,
        );
    }
}
