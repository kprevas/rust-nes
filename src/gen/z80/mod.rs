use gen::z80::opcodes::{BIT_INSTRUCTIONS, IX_BIT_INSTRUCTIONS, IX_INSTRUCTIONS, IY_BIT_INSTRUCTIONS, IY_INSTRUCTIONS, MISC_INSTRUCTIONS, Opcode, OPCODES};

mod opcodes;

pub struct Cpu<'a> {
    pc: u16,
    sp: u16,
    ix: u16,
    iy: u16,
    i: u8,
    r: u8,
    a: [u8; 2],
    f: [u8; 2],
    bc: [u16; 2],
    de: [u16; 2],
    hl: [u16; 2],

    interrupt_mode: u8,
    pub stopped: bool,

    ram: [u8; 0x2000],
    cartridge: &'a Box<[u8]>,

    ticks: u8,
}

impl Cpu<'_> {
    pub fn new(cartridge: &Box<[u8]>) -> Cpu {
        Cpu {
            pc: 0,
            sp: 0,
            ix: 0,
            iy: 0,
            i: 0,
            r: 0,
            a: [0, 0],
            f: [0, 0],
            bc: [0, 0],
            de: [0, 0],
            hl: [0, 0],
            interrupt_mode: 0,
            stopped: false,
            ram: [0; 0x2000],
            cartridge,
            ticks: 0,
        }
    }

    fn read_addr(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize],
            0x2000..=0x3FFF => self.ram[(addr - 0x2000) as usize],
            0x4000..=0x5FFF => 0, // TODO: YM2612
            0x6000..=0x60FF => 0xFF,
            0x6100..=0x7EFF => 0xFF,
            0x7F00..=0x7F1F => 0, // TODO: VDP
            0x7F20..=0x7FFF => 0xFF,
            0x8000..=0xFFFF => 0, // TODO: M68k
            _ => panic!(),
        }
    }

    fn write_addr(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize] = val,
            0x2000..=0x3FFF => self.ram[(addr - 0x2000) as usize] = val,
            0x4000..=0x5FFF => {} // TODO: YM2612
            0x6000..=0x60FF => {} // TODO: bank addr register
            0x6100..=0x7EFF => {}
            0x7F00..=0x7F1F => {} // TODO: VDP
            0x7F20..=0x7FFF => panic!(),
            0x8000..=0xFFFF => {}
            _ => panic!(),
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.i = 0;
        self.r = 0;
        self.interrupt_mode = 0;
    }

    pub fn tick(&mut self) {
        if self.ticks == 0 {
            self.next_operation();
        }
        self.ticks = self.ticks.saturating_sub(1);
    }

    fn next_operation(&mut self) {
        if self.stopped {
            self.ticks = 0;
        } else {
            self.execute_opcode();
        }
    }

    fn execute_opcode(&mut self) {
        let mut opcode = OPCODES[self.read_addr(self.pc) as usize];
        self.pc += 1;
        match opcode {
            Opcode::Bit => {
                opcode = BIT_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::Ix => {
                opcode = IX_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::Iy => {
                opcode = IY_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::Misc => {
                opcode = MISC_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            _ => {}
        }
        match opcode {
            Opcode::IxBit => {
                opcode = IX_BIT_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            Opcode::IyBit => {
                opcode = IY_BIT_INSTRUCTIONS[self.read_addr(self.pc) as usize];
                self.pc += 1;
            }
            _ => {}
        }
        match opcode {
            Opcode::NOP => {
                self.ticks += 4 * 15;
            }
            _ => {}
        }
    }
}

#[cfg(feature = "test")]
#[allow(dead_code)]
pub mod testing {
    use gen::z80::Cpu;

    impl Cpu<'_> {
        pub fn get_de(&mut self) -> u16 {
            self.de[0]
        }

        pub fn get_pc(&mut self) -> u16 {
            self.pc
        }

        pub fn set_pc(&mut self, pc: u16) {
            self.pc = pc;
        }

        pub fn load_ram(&mut self, start: usize, ram: &[u8]) {
            self.ram[start..start + ram.len()].copy_from_slice(ram);
        }
    }
}