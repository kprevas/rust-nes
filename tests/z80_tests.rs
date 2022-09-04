extern crate emu;

use emu::gen::z80::Cpu;

#[test]
fn prelim() {
    run_test(include_bytes!("z80/prelim.com"), 0x44A);
}

#[test]
#[ignore]
fn zexdoc() {
    run_test(include_bytes!("z80/zexdoc.cim"), 0x1DF9);
}

fn run_test(ram: &[u8], success_msg_addr: u16) {
    let _ = env_logger::try_init();
    let cartridge = vec![].into_boxed_slice();
    let mut cpu = Cpu::new(&cartridge, true);
    cpu.set_pc(0x100);
    cpu.load_ram(0x100, ram);
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
