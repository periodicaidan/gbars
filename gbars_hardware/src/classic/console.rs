#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    vec::Vec
};

use super::{
    cpu::Cpu,
    cartridge::Cartridge
};

pub const ROM_BANK_0_START: usize = 0x0000;
pub const ROM_BANK_N_START: usize = 0x4000;
pub const CHR_RAM_START: usize = 0x8000;
pub const BG_MAP_DATA_1_START: usize = 0x9800;
pub const BG_MAP_DATA_2_START: usize = 0x9C00;
pub const CARTRIDGE_RAM_START: usize = 0xA000;
pub const WRAM_START: usize = 0xC000;
pub const ECHO_RAM_START: usize = 0xE000;
pub const OAM_START: usize = 0xFE00;
pub const OAM_END: usize = 0xFEA0;
pub const HARDWARE_IO_START: usize = 0xFF00;
pub const HIGH_RAM_START: usize = 0xFF80;
pub const IE_START: usize = 0xFFFF;

pub const CHR_RAM_SIZE: usize = BG_MAP_DATA_1_START - CHR_RAM_START;
pub const BG_MAP_DATA_SIZE: usize = CARTRIDGE_RAM_START - BG_MAP_DATA_1_START;
pub const WRAM_SIZE: usize = ECHO_RAM_START - WRAM_START;
pub const ECHO_RAM_SIZE: usize = OAM_START - ECHO_RAM_START;
pub const OAM_SIZE: usize = OAM_END - OAM_START;
pub const HARDWARE_IO_SIZE: usize = HIGH_RAM_START - HARDWARE_IO_START;
pub const HIGH_RAM_SIZE: usize = IE_START - HIGH_RAM_START;

pub struct Console {
    pub cartridge: Option<Cartridge>,

    // internal RAM
    pub chr_ram: Vec<u8>, // Character RAM
    pub bg_data: Vec<u8>, // Background Map Data
    pub wram: Vec<u8>, // Work RAM
    pub oam: Vec<u8>,
    pub hardware: Vec<u8>,
    pub hi_ram: Vec<u8>,
    pub ie: bool,
}

impl Console {
    pub fn start(cartridge: Option<Cartridge>) -> Self {
        Self {
            cartridge,
            chr_ram: vec![0; CHR_RAM_SIZE],
            bg_data: vec![0; BG_MAP_DATA_SIZE],
            wram: vec![0; WRAM_SIZE],
            oam: vec![0; OAM_SIZE],
            hardware: vec![0; HARDWARE_IO_SIZE],
            hi_ram: vec![0; HIGH_RAM_SIZE],
            ie: false
        }
    }

    pub fn read(&self, offset: usize) -> Option<u8> {
        match offset {
            // Overflow (offset larger than a short)
            over if over > 0xFFFF => panic!(),

            // Mapped to cartridge ROM
            0x0000 ..=  0x7FFF => if let Some(cart) = &self.cartridge {
                cart.read_rom(offset)
            } else {
                None
            },

            // Character RAM
            0x8000 ..= 0x97FF => self.chr_ram.get(offset - CHR_RAM_START).map(|b| *b),

            // Background map data
            0x9800 ..= 0x9FFF => self.bg_data.get(offset - BG_MAP_DATA_1_START).map(|b| *b),

            // Mapped to cartridge RAM
            0xA000 ..= 0xBFFF => if let Some(cart) = &self.cartridge {
                cart.mbc.read_ram(offset - CARTRIDGE_RAM_START)
            } else {
                None
            },

            // Work RAM
            0xC000 ..= 0xDFFF => self.wram.get(offset - WRAM_START).map(|b| *b),

            // Echo RAM
            0xE000 ..= 0xFDFF => self.wram.get(offset - (ECHO_RAM_START - WRAM_START)).map(|b| *b),

            // OAM (Sprite data)
            0xFE00 ..= 0xFE9F => self.oam.get(offset - OAM_START).map(|b| *b),

            // Unused
            0xFEA0 ..= 0xFEFF => None,

            // Hardware I/O
            0xFF00 ..= 0xFF7F => self.hardware.get(offset - HARDWARE_IO_START).map(|b| *b),

            // High RAM Area
            0xFF80 ..= 0xFFFE => self.hi_ram.get(offset - HIGH_RAM_START).map(|b| *b),

            // Interrupt Enable Register
            0xFFFF => Some(self.ie as u8),

            _ => None
        }
    }

    pub fn write(&mut self, offset: usize, data: u8) -> Option<()> {
        match offset {
            // Overflow (offset larger than a short)
            over if over > 0xFFFF => panic!(),

            // Mapped to cartridge ROM
            0x0000 ..=  0x7FFF => if let Some(cart) = &mut self.cartridge {
                Some(cart.mbc.write_rom(offset, data))
            } else {
                None
            },

            // Character RAM
            0x8000 ..= 0x97FF =>
                self.chr_ram.get_mut(offset - CHR_RAM_START).map(|b| *b = data),

            // Background map data
            0x9800 ..= 0x9FFF =>
                self.bg_data.get_mut(offset - BG_MAP_DATA_1_START).map(|b| *b = data),

            // Mapped to cartridge RAM
            0xA000 ..= 0xBFFF => if let Some(cart) = &mut self.cartridge {
                Some(cart.mbc.write_rom(offset - CARTRIDGE_RAM_START, data))
            } else {
                None
            },

            // Work RAM
            0xC000 ..= 0xDFFF =>
                self.wram.get_mut(offset - WRAM_START).map(|b| *b = data),

            // Echo RAM
            0xE000 ..= 0xFDFF =>
                self.wram.get_mut(offset - (ECHO_RAM_START - WRAM_START)).map(|b| *b = data),

            // OAM (Sprite data)
            0xFE00 ..= 0xFE9F =>
                self.oam.get_mut(offset - OAM_START).map(|b| *b = data),

            // Unused
            0xFEA0 ..= 0xFEFF => None,

            // Hardware I/O
            0xFF00 ..= 0xFF7F =>
                self.hardware.get_mut(offset - HARDWARE_IO_START).map(|b| *b = data),

            // High RAM Area
            0xFF80 ..= 0xFFFE =>
                self.hi_ram.get_mut(offset - HIGH_RAM_START).map(|b| *b = data),

            // Interrupt Enable Register
            0xFFFF => Some(self.ie = data != 0),

            _ => None
        }
    }

    pub fn alter(&mut self, offset: usize, f: fn (u8) -> u8) -> Option<()> {
        self.read(offset).and_then(|data| self.write(offset, f(data)))
    }
}