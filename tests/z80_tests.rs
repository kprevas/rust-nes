extern crate emu;

use emu::gen::z80::Cpu;

#[test]
#[ignore]
fn prelim() {
    let _ = env_logger::try_init();
    let cartridge = vec![].into_boxed_slice();
    let mut cpu = Cpu::new(&cartridge, true);
    cpu.set_pc(0x100);
    cpu.load_ram(0x100, include_bytes!("z80/prelim.com"));
    while cpu.get_pc() != 0 && cpu.get_pc() != 5 && !cpu.stopped {
        cpu.tick()
    }
    assert_eq!(cpu.get_pc(), 5);
    assert_eq!(cpu.get_de(), 0x44A)
}