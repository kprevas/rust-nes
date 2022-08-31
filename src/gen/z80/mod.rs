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
            ram: [0; 0x2000],
            cartridge,
            ticks: 0,
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
        self.ticks -= 1;
    }

    fn next_operation(&mut self) {}
}