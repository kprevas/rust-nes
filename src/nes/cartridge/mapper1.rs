use std::cell::RefCell;
use std::cmp::max;
use std::io::prelude::*;
use std::io::Result;
use std::ops::Deref;
use std::rc::Rc;

use bincode::{deserialize_from, serialize};
use bytes::*;

use nes::cartridge::Cartridge;
use nes::cartridge::CartridgeBus;
use nes::cartridge::Header;
use nes::cartridge::NametableMirroring;
use nes::cartridge::NametableMirroring::*;

#[derive(Serialize, Deserialize)]
enum PrgBankMode {
    Switch32K,
    FixLowBank,
    FixHiBank,
}

#[derive(Serialize, Deserialize)]
enum ChrBankMode {
    Switch8K,
    Switch4K,
}

#[derive(Serialize, Deserialize)]
struct CtrlRegisters {
    mirroring: NametableMirroring,
    one_screen_mirroring_hi: bool,
    prg_bank_mode: PrgBankMode,
    chr_bank_mode: ChrBankMode,
    chr_bank_low: usize,
    chr_bank_hi: usize,
    prg_bank: usize,
    prg_ram_enabled: bool,
    shift_reg: u8,
    shift_reg_written: u8,
}

impl CtrlRegisters {
    fn write(&mut self, address: u16, value: u8) {
        if value & 0x80 > 0 {
            let new_val = self.read_ctrl() | 0xC;
            self.write_ctrl(new_val);
            self.shift_reg_written = 0;
        } else {
            self.shift_reg >>= 1;
            self.shift_reg |= (value & 1) << 4;
            self.shift_reg_written += 1;
            if self.shift_reg_written == 5 {
                let value = self.shift_reg;
                match address {
                    0x8000..=0x9FFF => self.write_ctrl(value),
                    0xA000..=0xBFFF => self.chr_bank_low = value as usize,
                    0xC000..=0xDFFF => self.chr_bank_hi = value as usize,
                    0xE000..=0xFFFF => {
                        self.prg_ram_enabled = value & 0b10000 == 0;
                        self.prg_bank = (value & 0b1111) as usize;
                    }
                    _ => unreachable!(),
                }
                self.shift_reg_written = 0;
            }
        }
    }

    fn write_ctrl(&mut self, val: u8) {
        use self::ChrBankMode::*;
        use self::PrgBankMode::*;
        self.mirroring = match val & 3 {
            0 => {
                self.one_screen_mirroring_hi = false;
                SingleScreen
            }
            1 => {
                self.one_screen_mirroring_hi = true;
                SingleScreen
            }
            2 => Vertical,
            3 => Horizontal,
            _ => unreachable!(),
        };
        self.prg_bank_mode = match (val >> 2) & 3 {
            0 | 1 => Switch32K,
            2 => FixLowBank,
            3 => FixHiBank,
            _ => unreachable!(),
        };
        self.chr_bank_mode = if (val >> 4) & 1 > 0 {
            Switch4K
        } else {
            Switch8K
        }
    }

    fn read_ctrl(&self) -> u8 {
        use self::ChrBankMode::*;
        use self::PrgBankMode::*;

        let mut val = 0;
        match self.mirroring {
            SingleScreen => val |= if self.one_screen_mirroring_hi { 1 } else { 0 },
            Vertical => val |= 2,
            Horizontal => val |= 3,
            _ => unreachable!(),
        };
        match self.prg_bank_mode {
            Switch32K => (),
            FixLowBank => val |= 2 << 2,
            FixHiBank => val |= 3 << 2,
        };
        match self.chr_bank_mode {
            Switch4K => (),
            Switch8K => val |= 1 << 4,
        };
        val
    }

    fn chr_low_bank(&self, addr: u16, max_addr: usize) -> usize {
        ((addr as usize)
            + match self.chr_bank_mode {
            ChrBankMode::Switch8K => (self.chr_bank_low & (!1)) << 12,
            ChrBankMode::Switch4K => self.chr_bank_low << 12,
        })
            % max_addr
    }

    fn chr_hi_bank(&self, addr: u16, max_addr: usize) -> usize {
        ((addr as usize)
            + match self.chr_bank_mode {
            ChrBankMode::Switch8K => (self.chr_bank_low | 1) << 12,
            ChrBankMode::Switch4K => self.chr_bank_hi << 12,
        })
            % max_addr
    }

    fn prg_low_bank(&self, addr: u16, max_addr: usize) -> usize {
        ((addr as usize)
            + match self.prg_bank_mode {
            PrgBankMode::Switch32K => (self.prg_bank & (!1)) << 14,
            PrgBankMode::FixLowBank => 0,
            PrgBankMode::FixHiBank => self.prg_bank << 14,
        })
            % max_addr
    }

    fn prg_hi_bank(&self, addr: u16, max_addr: usize) -> usize {
        ((addr as usize)
            + match self.prg_bank_mode {
            PrgBankMode::Switch32K => (self.prg_bank | 1) << 14,
            PrgBankMode::FixLowBank => self.prg_bank << 14,
            PrgBankMode::FixHiBank => max_addr - 0x4000,
        })
            % max_addr
    }
}

struct Mapper1Cpu {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    ctrl: Rc<RefCell<CtrlRegisters>>,
    battery_save: bool,
    last_write_cycle: u64,
}

struct Mapper1Ppu {
    chr_rom: Vec<u8>,
    uses_chr_ram: bool,
    ctrl: Rc<RefCell<CtrlRegisters>>,
}

pub fn read(header: &Header, prg_rom: &[u8], chr_rom: &[u8]) -> Cartridge {
    let uses_chr_ram = chr_rom.len() == 0;
    let ctrl_register = Rc::new(RefCell::new(CtrlRegisters {
        mirroring: SingleScreen,
        one_screen_mirroring_hi: false,
        prg_bank_mode: self::PrgBankMode::FixHiBank,
        chr_bank_mode: self::ChrBankMode::Switch8K,
        chr_bank_low: 0,
        chr_bank_hi: 0,
        prg_bank: 0,
        prg_ram_enabled: true,
        shift_reg: 0,
        shift_reg_written: 0,
    }));
    let cpu_bus = Box::new(Mapper1Cpu {
        prg_rom: prg_rom.to_vec(),
        prg_ram: vec![0; (u16::from(max(header.prg_ram_blocks, 1)) * 0x2000) as usize],
        ctrl: Rc::clone(&ctrl_register),
        battery_save: header.battery_save,
        last_write_cycle: 0,
    });
    Cartridge {
        cpu_bus,
        ppu_bus: Box::new(Mapper1Ppu {
            chr_rom: if uses_chr_ram {
                vec![0; 0x80000]
            } else {
                chr_rom.to_vec()
            },
            uses_chr_ram,
            ctrl: Rc::clone(&ctrl_register),
        }),
    }
}

impl CartridgeBus for Mapper1Cpu {
    fn read_memory(&self, address: u16, open_bus: u8) -> u8 {
        let ctrl = self.ctrl.borrow();
        match address {
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize],
            0x8000..=0xBFFF => {
                self.prg_rom[ctrl.prg_low_bank(address - 0x8000, self.prg_rom.len())]
            }
            0xC000..=0xFFFF => self.prg_rom[ctrl.prg_hi_bank(address - 0xC000, self.prg_rom.len())],
            _ => open_bus,
        }
    }

    fn write_memory(&mut self, address: u16, value: u8, cpu_cycle: u64) {
        let mut ctrl = self.ctrl.borrow_mut();
        match address {
            0x6000..=0x7FFF => {
                if ctrl.prg_ram_enabled {
                    self.prg_ram[(address - 0x6000) as usize] = value;
                }
            }
            0x8000..=0xFFFF => {
                if cpu_cycle - self.last_write_cycle > 2 {
                    ctrl.write(address, value);
                }
            }
            _ => (),
        }
        self.last_write_cycle = cpu_cycle;
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        address
    }

    fn save_to_battery(&self, out: &mut dyn Write) -> Result<usize> {
        if self.battery_save {
            out.write(self.prg_ram.as_slice())
        } else {
            Ok(0)
        }
    }

    fn load_from_battery(&mut self, inp: &mut dyn Read) -> Result<usize> {
        if self.battery_save {
            self.prg_ram.clear();
            inp.read_to_end(&mut self.prg_ram)
        } else {
            Ok(0)
        }
    }

    fn save_state(&self, out: &mut Vec<u8>) {
        out.put_slice(&self.prg_ram);
        out.put_slice(&serialize(&self.ctrl.borrow().deref()).unwrap());
        out.put_u8(if self.battery_save { 1 } else { 0 });
        out.put_u64(self.last_write_cycle);
    }

    fn load_state(&mut self, state: &mut dyn Buf) {
        state.copy_to_slice(&mut self.prg_ram);
        self.ctrl.replace(deserialize_from(state.reader()).unwrap());
        self.battery_save = state.get_u8() == 1;
        self.last_write_cycle = state.get_u64();
    }
}

impl CartridgeBus for Mapper1Ppu {
    fn read_memory(&self, address: u16, open_bus: u8) -> u8 {
        let ctrl = self.ctrl.borrow();
        let max_addr = self.chr_rom.len();
        match address {
            0x0000..=0x0FFF => self.chr_rom[ctrl.chr_low_bank(address, max_addr)],
            0x1000..=0x1FFF => self.chr_rom[ctrl.chr_hi_bank(address - 0x1000, max_addr)],
            _ => open_bus,
        }
    }

    fn write_memory(&mut self, address: u16, value: u8, _cpu_cycle: u64) {
        let ctrl = self.ctrl.borrow();
        if self.uses_chr_ram {
            let max_addr = self.chr_rom.len();
            match address {
                0x0000..=0x0FFF => self.chr_rom[ctrl.chr_low_bank(address, max_addr)] = value,
                0x1000..=0x1FFF => {
                    self.chr_rom[ctrl.chr_hi_bank(address - 0x1000, max_addr)] = value
                }
                _ => (),
            }
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        let ctrl = self.ctrl.borrow();
        match address {
            0x2000..=0x23FF => address - 0x2000,
            0x2400..=0x27FF => match ctrl.mirroring {
                Vertical => address - 0x2000,
                Horizontal => address - 0x2400,
                SingleScreen => {
                    if ctrl.one_screen_mirroring_hi {
                        address - 0x2000
                    } else {
                        address - 0x2400
                    }
                }
                _ => unimplemented!(),
            },
            0x2800..=0x2BFF => match ctrl.mirroring {
                Vertical => address - 0x2800,
                Horizontal => address - 0x2400,
                SingleScreen => {
                    if ctrl.one_screen_mirroring_hi {
                        address - 0x2400
                    } else {
                        address - 0x2800
                    }
                }
                _ => unimplemented!(),
            },
            0x2C00..=0x2FFF => match ctrl.mirroring {
                Vertical | Horizontal => address - 0x2800,
                SingleScreen => {
                    if ctrl.one_screen_mirroring_hi {
                        address - 0x2800
                    } else {
                        address - 0x2C00
                    }
                }
                _ => unimplemented!(),
            },
            _ => panic!("Bad nametable mirror request {:04X}", address),
        }
    }

    fn save_to_battery(&self, _out: &mut dyn Write) -> Result<usize> {
        unimplemented!();
    }

    fn load_from_battery(&mut self, _inp: &mut dyn Read) -> Result<usize> {
        unimplemented!();
    }

    fn save_state(&self, out: &mut Vec<u8>) {
        if self.uses_chr_ram {
            out.put_slice(&self.chr_rom);
        }
    }

    fn load_state(&mut self, state: &mut dyn Buf) {
        if self.uses_chr_ram {
            state.copy_to_slice(&mut self.chr_rom);
        }
    }
}
