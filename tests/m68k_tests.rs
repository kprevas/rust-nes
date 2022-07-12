extern crate nes;

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