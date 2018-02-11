use cartridge::Cartridge;
use cartridge::CartridgeBus;
use cartridge::Header;
use cartridge::NametableMirroring;
use cartridge::NametableMirroring::*;
use std::cell::RefCell;
use std::cmp::max;
use std::rc::Rc;

enum PrgBankMode {
    Switch32K,
    FixLowBank,
    FixHiBank,
}

enum ChrBankMode {
    Switch8K,
    Switch4K,
}

struct CtrlRegisters {
    mirroring: NametableMirroring,
    one_screen_mirroring_hi: bool,
    prg_bank_mode: PrgBankMode,
    chr_bank_mode: ChrBankMode,
    chr_bank_low: usize,
    chr_bank_hi: usize,
    prg_bank: usize,
    shift_reg: u8,
    shift_reg_written: u8,
}

impl CtrlRegisters {
    fn write(&mut self, address: u16, value: u8) {
        if value & 0xF0 > 0 {
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
                    0x8000 ... 0x9FFF => self.write_ctrl(value),
                    0xA000 ... 0xBFFF => self.chr_bank_low = value as usize,
                    0xC000 ... 0xDFFF => self.chr_bank_hi = value as usize,
                    0xE000 ... 0xFFFF => self.prg_bank = value as usize,
                    _ => unreachable!(),
                }
                self.shift_reg_written = 0;
            }
        }
    }

    fn write_ctrl(&mut self, val: u8) {
        use self::PrgBankMode::*;
        use self::ChrBankMode::*;
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
        self.chr_bank_mode = if (val >> 4) & 1 > 0 { Switch4K } else { Switch8K }
    }

    fn read_ctrl(&self) -> u8 {
        use self::PrgBankMode::*;
        use self::ChrBankMode::*;

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

    fn chr_low_bank(&self, addr: u16) -> usize {
        (addr as usize) + match self.chr_bank_mode {
            ChrBankMode::Switch8K => (self.chr_bank_low & (!1)) << 12,
            ChrBankMode::Switch4K => self.chr_bank_low << 12,
        }
    }

    fn chr_hi_bank(&self, addr: u16) -> usize {
        (addr as usize) + match self.chr_bank_mode {
            ChrBankMode::Switch8K => (self.chr_bank_low | 1) << 12,
            ChrBankMode::Switch4K => self.chr_bank_hi << 12,
        }
    }

    fn prg_low_bank(&self, addr: u16) -> usize {
        (addr as usize) + match self.prg_bank_mode {
            PrgBankMode::Switch32K => (self.prg_bank & (!1)) << 14,
            PrgBankMode::FixLowBank => 0,
            PrgBankMode::FixHiBank => self.prg_bank << 14,
        }
    }

    fn prg_hi_bank(&self, addr: u16, max_addr: usize) -> usize {
        (addr as usize) + match self.prg_bank_mode {
            PrgBankMode::Switch32K => (self.prg_bank | 1) << 14,
            PrgBankMode::FixLowBank => self.prg_bank << 14,
            PrgBankMode::FixHiBank => max_addr - 0x4000,
        }
    }
}

struct Mapper1Cpu {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    prg_ram_enabled: bool,
    ctrl: Rc<RefCell<CtrlRegisters>>,
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
        prg_bank_mode: self::PrgBankMode::Switch32K,
        chr_bank_mode: self::ChrBankMode::Switch8K,
        chr_bank_low: 0,
        chr_bank_hi: 0,
        prg_bank: 0,
        shift_reg: 0,
        shift_reg_written: 0,
    }));
    let cpu_bus = Box::new(Mapper1Cpu {
        prg_rom: prg_rom.to_vec(),
        prg_ram: vec![0; (u16::from(max(header.prg_ram_blocks, 1)) * 0x2000) as usize],
        prg_ram_enabled: true,
        ctrl: Rc::clone(&ctrl_register),
    });
    Cartridge {
        cpu_bus,
        ppu_bus: Box::new(Mapper1Ppu {
            chr_rom: if uses_chr_ram { vec!(0; 0x80000) } else { chr_rom.to_vec() },
            uses_chr_ram,
            ctrl: Rc::clone(&ctrl_register),
        }),
    }
}

impl CartridgeBus for Mapper1Cpu {
    fn read_memory(&self, address: u16) -> u8 {
        let ctrl = self.ctrl.borrow();
        match address {
            0x6000 ... 0x7FFF => self.prg_ram[(address - 0x6000) as usize],
            0x8000 ... 0xBFFF => self.prg_rom[ctrl.prg_low_bank(address - 0x8000)],
            0xC000 ... 0xFFFF => self.prg_rom[ctrl.prg_hi_bank(address - 0xC000, self.prg_rom.len())],
            _ => panic!("bad memory read 0x{:04X}", address),
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        let mut ctrl = self.ctrl.borrow_mut();
        match address {
            0x6000 ... 0x7FFF => {
                if self.prg_ram_enabled {
                    self.prg_ram[(address - 0x6000) as usize] = value;
                } else {
                    panic!("bad memory write 0x{:04X}", address);
                }
            }
            0x8000 ... 0xFFFF => ctrl.write(address, value),
            _ => panic!("bad memory write 0x{:04X}", address),
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        address
    }
}

impl CartridgeBus for Mapper1Ppu {
    fn read_memory(&self, address: u16) -> u8 {
        let ctrl = self.ctrl.borrow();
        match address {
            0x0000 ... 0x0FFF => self.chr_rom[ctrl.chr_low_bank(address)],
            0x1000 ... 0x1FFF => self.chr_rom[ctrl.chr_hi_bank(address - 0x1000)],
            _ => panic!("bad memory read 0x{:04X}", address),
        }
    }

    fn write_memory(&mut self, address: u16, value: u8) {
        let ctrl = self.ctrl.borrow();
        if self.uses_chr_ram {
            match address {
                0x0000 ... 0x0FFF => self.chr_rom[ctrl.chr_low_bank(address)] = value,
                0x1000 ... 0x1FFF => self.chr_rom[ctrl.chr_hi_bank(address - 0x1000)] = value,
                _ => panic!("bad memory write 0x{:04X}", address),
            }
        } else {
            panic!("bad memory write 0x{:04X}", address);
        }
    }

    fn mirror_nametable(&self, address: u16) -> u16 {
        let ctrl = self.ctrl.borrow();
        match address {
            0x2000 ... 0x23FF => address - 0x2000,
            0x2400 ... 0x27FF => match ctrl.mirroring {
                Vertical => address - 0x2000,
                Horizontal => address - 0x2400,
                SingleScreen => if ctrl.one_screen_mirroring_hi { address - 0x2000 } else { address - 0x2400 },
                _ => unimplemented!(),
            },
            0x2800 ... 0x2BFF => match ctrl.mirroring {
                Vertical => address - 0x2800,
                Horizontal => address - 0x2400,
                SingleScreen => if ctrl.one_screen_mirroring_hi { address - 0x2400 } else { address - 0x2800 },
                _ => unimplemented!(),
            },
            0x2C00 ... 0x2FFF => match ctrl.mirroring {
                Vertical | Horizontal => address - 0x2800,
                SingleScreen => if ctrl.one_screen_mirroring_hi { address - 0x2800 } else { address - 0x2C00 },
                _ => unimplemented!(),
            },
            _ => panic!("Bad nametable mirror request {:04X}", address),
        }
    }
}
