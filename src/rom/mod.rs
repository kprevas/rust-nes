use nes::cartridge::Cartridge;

pub enum Rom {
    Nes(Cartridge),
    Genesis(Box<[u8]>),
}