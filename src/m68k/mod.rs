use std::marker::PhantomData;

use input::ControllerState;
use m68k::opcodes::opcode;

mod opcodes;

const CPU_TICKS_PER_SECOND: f64 = 7_670_454.0;

pub struct Cpu<'a> {
    a: [u32; 8],
    d: [u32; 8],
    status: u16,
    pc: u32,
    ticks: f64,
    instrumented: bool,
    cycle_count: u64,

    pub speed_adj: f64,

    phantom: PhantomData<&'a u8>,
}

const INTERRUPT: u16 = 0b0000011100000000;
const INTERRUPT_SHIFT: u16 = 8;

impl<'a> Cpu<'a> {
    pub fn boot<'b>(instrumented: bool) -> Cpu<'b> {
        let mut cpu = Cpu {
            a: [0, 0, 0, 0, 0, 0, 0, 0],
            d: [0, 0, 0, 0, 0, 0, 0, 0],
            status: 0,
            pc: 0,
            ticks: 0.0,
            instrumented,
            cycle_count: 0,
            speed_adj: 1.0,
            phantom: PhantomData,
        };

        cpu.reset(false);

        cpu
    }

    fn tick(&mut self) {}

    fn set_interrupt_level(&mut self, level: u16) {
        assert!(level <= (INTERRUPT >> INTERRUPT_SHIFT));
        self.status =
            (self.status & (!INTERRUPT)) | (level << INTERRUPT_SHIFT)
    }

    fn read_word(&mut self, _addr: u32) -> u16 { 0 }

    fn read_dword_no_tick(&mut self, _addr: u32) -> u32 {
        0
    }

    fn execute_opcode(&mut self) {
        let opcode_pc = self.pc;
        let opcode_hex = self.read_word(opcode_pc);
        self.pc += 1;

        let opcode = opcode(opcode_hex);

        match opcode {
            _ => { unimplemented!("{:04X} {:?}", opcode_hex, opcode) }
        }
    }

    pub fn next_operation(&mut self, inputs: &[ControllerState<8>; 2]) {
        self.execute_opcode();
    }

    pub fn do_frame(&mut self, time_secs: f64, inputs: &[ControllerState<8>; 2]) {
        self.ticks += time_secs * CPU_TICKS_PER_SECOND * self.speed_adj;

        while self.ticks > 0.0 {
            self.next_operation(inputs);
        }
    }

    pub fn reset(&mut self, _soft: bool) {
        self.a[7] = self.read_dword_no_tick(0x000000);
        self.pc = self.read_dword_no_tick(0x000004);
        self.set_interrupt_level(7);
    }

    pub fn close(&mut self) {}
}