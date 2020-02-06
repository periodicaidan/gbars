use std::ops::{Deref, DerefMut};
use bitmatch::bitmatch;

/// The ROM of the cartridge, which is a pointer to a vector of bytes
pub struct ROM(Vec<u8>);

impl Deref for ROM {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The RAM of the cartridge, which is a read/write pointer to a vector of bytes
pub struct RAM(Vec<u8>);

impl Deref for RAM {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RAM {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The memory bank controller is a hack built into the cartridge to allow the GameBoy to play
/// games larger than its available RAM. It does this by dividing the ROM into "banks" and switching
/// between them by writing to certain address spaces in the ROM.
pub enum MBC {
    MBC1(MBC1),
    MBC2(MBC2),
    MBC3(MBC3),
    MBC5(MBC5),
    RomOnly(ROM),
}

pub enum MbcMode {
    RomSelect,
    RamSelect,
}

pub struct MBC1 {
    pub rom: ROM,
    pub ram: RAM,
    pub active_rom_bank: usize,
    pub active_ram_bank: usize,
    pub ram_enabled: bool,
    pub mode: MbcMode,
}

pub struct MBC2 {
    pub rom: ROM,
    pub ram: RAM,
    pub active_rom_bank: usize,
    pub active_ram_bank: usize,
    pub ram_enabled: bool,
}

pub struct MBC3 {
    pub rom: ROM,
    pub ram: RAM,
    pub active_rom_bank: usize,
    pub active_ram_bank: usize,
    pub ram_and_timer_enabled: bool,
}

pub struct MBC5 {
    pub rom: ROM,
    pub ram: RAM,
    pub active_rom_bank: usize,
    pub active_ram_bank: usize,
    pub ram_enabled: bool,
}

impl ROM {
    pub fn new(contents: Vec<u8>) -> Self {
        Self(contents)
    }

    pub fn read_byte(&self, offset: usize) -> Option<u8> {
        match self.get(offset) {
            Some(b) => Some(*b),
            None => None
        }
    }

    pub fn read_bytes(&self, start: usize, end: usize) -> Option<Vec<u8>> {
        if end > self.len() || start > end {
            None
        } else {
            Some(Vec::from(&self[start..end]))
        }
    }
}

impl RAM {
    pub fn new(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }

    pub fn read_byte(&self, offset: usize) -> Option<u8> {
        match self.get(offset) {
            Some(b) => Some(*b),
            None => None
        }
    }

    pub fn read_bytes(&self, start: usize, end: usize) -> Option<Vec<u8>> {
        if end > self.len() || start > end {
            None
        } else {
            Some(Vec::from(&self[start..end]))
        }
    }

    pub fn write_byte(&mut self, offset: usize, data: u8) -> Result<usize, String> {
        if offset > self.len() {
            Err(format!("Could not write data at offset {:04X}: Out of bounds", offset))
        } else {
            self[offset] = data;
            Ok(1)
        }
    }

    pub fn write_bytes(&mut self, start: usize, data: &[u8]) -> Result<usize, String> {
        if start > self.len() {
            Err(format!("Could not write data to cartridge RAM at offset {:04X}: Out of bounds", start))
        } else if self.len() - start < data.len() {
            Err(format!(
                "Could not write data to cartridge RAM: source data are longer than target range ({} > {})",
                data.len(),
                self.len() - start
            ))
        } else {
            for (i, byte) in data.iter().enumerate() {
                self[start + i] = *byte;
            }

            Ok(data.len())
        }
    }
}

impl MBC {
    pub fn read_rom(&self, offset: usize) -> Option<u8> {
        #[inline]
        fn read_rom_bank(rom: &ROM, offset: usize, bank: usize) -> Option<u8> {
            if offset < 0x4000 {
                rom.read_byte(offset)
            } else {
                rom.read_byte(0x4000 * bank + offset)
            }
        }

        match self {
            MBC::MBC1(mbc) => {
                let mut active_rom_bank = match mbc.mode {
                    MbcMode::RomSelect => mbc.active_rom_bank & 0x1F,
                    MbcMode::RamSelect => mbc.active_rom_bank
                };

                // Bank 0 isn't switchable and banks 0x20, 0x40, and 0x60 are not usable. Attempting
                // to access one of these accesses the following one (1, 0x21, etc.)
                if [0, 0x20, 0x40, 0x60].contains(&active_rom_bank) {
                    active_rom_bank += 1;
                }

                read_rom_bank(&mbc.rom, offset, active_rom_bank)
            },

            MBC::MBC2(mbc) => read_rom_bank(&mbc.rom, offset, mbc.active_rom_bank),
            MBC::MBC3(mbc) => read_rom_bank(&mbc.rom, offset, mbc.active_rom_bank),
            MBC::MBC5(mbc) => read_rom_bank(&mbc.rom, offset, mbc.active_rom_bank),
            MBC::RomOnly(rom) => rom.read_byte(offset)
        }
    }

    pub fn read_rom_slice(&self, start: usize, end: usize) -> Option<Vec<u8>> {
        #[inline]
        fn read_rom_bank_slice(rom: &ROM, start: usize, end: usize, bank: usize) -> Option<Vec<u8>> {
            if start < 0x4000 {
                rom.read_bytes(start, end)
            } else {
                rom.read_bytes(
                    0x4000 * bank + start,
                    0x4000 * bank + end
                )
            }
        }

        match self {
            MBC::MBC1(mbc) => read_rom_bank_slice(&mbc.rom, start, end, mbc.active_rom_bank),
            MBC::MBC2(mbc) => read_rom_bank_slice(&mbc.rom, start, end, mbc.active_rom_bank),
            MBC::MBC3(mbc) => read_rom_bank_slice(&mbc.rom, start, end, mbc.active_rom_bank),
            MBC::MBC5(mbc) => read_rom_bank_slice(&mbc.rom, start, end, mbc.active_rom_bank),
            MBC::RomOnly(rom) => rom.read_bytes(start, end),
        }
    }

    /// Yes, you can write to the ROM. Doing so is used for various controls like switching the
    /// ROM bank, or enabling the RAM
    pub fn write_rom(&mut self, offset: usize, data: u8) {
        match self {
            MBC::MBC1(mbc) => match offset {
                // RAM enable register
                // Writing 0 into this address space disables the RAM
                // Writing any number with lower nibble 0xA enables the RAM
                0...0x1FFF => if data == 0 {
                    mbc.ram_enabled = false;
                } else if data & 0x0F == 0x0A {
                    mbc.ram_enabled = true;
                },

                // (Lower) ROM bank select
                0x2000...0x3FFF => {
                    // This is used to select the lower 5 bits of the ROM bank number. The upper
                    // 2 bits (if applicable) are selected below.
                    let mut bank_number = (data & 0x1F) as usize;
                    bank_number |= mbc.active_rom_bank & 0x60;

                    mbc.active_rom_bank = bank_number;
                },

                // RAM bank select or (Upper) ROM Bank select
                0x4000...0x5FFF => {
                    let mut bank_number = (data & 0x02) as usize;
                    if mbc.ram_enabled {
                        mbc.active_ram_bank = bank_number;
                    } else {
                        bank_number <<= 5;
                        bank_number |= mbc.active_rom_bank & 0x1F;

                        mbc.active_rom_bank = bank_number;
                    }
                },

                // ROM/RAM mode select
                0x6000...0x7FFF => match data {
                    0 => mbc.mode = MbcMode::RomSelect,
                    1 => mbc.mode = MbcMode::RamSelect,
                    _ => {}
                }

                _ => {}
            },

            MBC::MBC2(mbc) => match offset {
                // RAM enable register. Same as for MBC1 but the least significant bit of the upper
                // address byte must be zero
                //
                //      0bXXXX_XXXB_XXXX_XXXX
                //                |
                //             this one
                0...0x1FFF => if offset & 0x0100 == 0 {
                    if data == 0 {
                        mbc.ram_enabled = false;
                    } else if data & 0x0F == 0x0A {
                        mbc.ram_enabled = true;
                    }
                },

                // ROM bank selection. We take the lower 4 bits only because MBC2 only has 16 banks.
                // Additionally, the least significant bit of the upper address byte must be 1.
                // This is the same byte as above.
                0x2000...0x3FFF => if offset & 0x0100 == 1 {
                    let bank_number = data & 0x0F;
                    mbc.active_rom_bank = bank_number as usize;
                },

                _ => {}
            },

            // This one has an internal clock, the maximum value of which is 511 days, 23 hours,
            // 59 minutes, and 59 seconds. This is the MBC that Pokémon Gold, Silver, and Crystal
            // use and it's how they accomplish things like daily events and time-variant encounters
            MBC::MBC3(mbc) => match offset {
                // RAM and timer enable
                0...0x1FFF => if data == 0 {
                    mbc.ram_and_timer_enabled = false;
                } else if data & 0x0F == 0x0A {
                    mbc.ram_and_timer_enabled = true;
                },

                // ROM bank select
                0x2000...0x3FFF => {
                    let mut bank_number = (0x7F & data) as usize;
                    if bank_number == 0 {
                        bank_number = 1;
                    }

                    mbc.active_rom_bank = bank_number;
                },

                // RAM bank select
                0x4000...0x5FFF => if (0..=0x0C).contains(&data) {
                    mbc.active_ram_bank = data as usize;
                },

                // Latches the time to the time register
                0x6000...0x7FFF => if data == 1 && mbc.rom[offset] == 0 {
                    // TODO: Figure out a way to implement this
                },

                _ => {}
            },

            MBC::MBC5(mbc) => match offset {
                0...0x1FFF => if data == 0 {
                    mbc.ram_enabled = false;
                } else if data & 0x0F == 0x0A {
                    mbc.ram_enabled = true;
                },

                0x2000...0x2FFF => {
                    let mut bank_number = data as usize;
                    bank_number |= mbc.active_rom_bank & 0x0100;

                    mbc.active_rom_bank = bank_number;
                },

                0x3000...0x3FFF => {
                    let mut bank_number = ((1 & data) << 8) as usize;
                    bank_number |= mbc.active_ram_bank & 0x00FF;

                    mbc.active_rom_bank = bank_number;
                },

                0x4000...0x5FFF => {
                    mbc.active_ram_bank = (0x0F & data) as usize;
                },

                _ => {}
            },

            _ => {}
        }
    }

    pub fn read_ram(&self, offset: usize) -> Option<u8> {
        match self {
            MBC::MBC1(mbc) => mbc.ram.read_byte(offset),
            MBC::MBC2(mbc) => mbc.ram.read_byte(offset),
            MBC::MBC3(mbc) => mbc.ram.read_byte(offset),
            MBC::MBC5(mbc) => mbc.ram.read_byte(offset),
            MBC::RomOnly(_) => None,
        }
    }

    pub fn read_ram_slice(&self, start: usize, end: usize) -> Option<Vec<u8>> {
        match self {
            MBC::MBC1(mbc) => mbc.ram.read_bytes(start, end),
            MBC::MBC2(mbc) => mbc.ram.read_bytes(start, end),
            MBC::MBC3(mbc) => mbc.ram.read_bytes(start, end),
            MBC::MBC5(mbc) => mbc.ram.read_bytes(start, end),
            MBC::RomOnly(_) => None,
        }
    }

    pub fn write_ram(&mut self, offset: usize, data: u8) -> Result<usize, String> {
        match self {
            MBC::MBC1(mbc) => mbc.ram.write_byte(offset, data),
            MBC::MBC2(mbc) => mbc.ram.write_byte(offset, data),
            MBC::MBC3(mbc) => mbc.ram.write_byte(offset, data),
            MBC::MBC5(mbc) => mbc.ram.write_byte(offset, data),
            MBC::RomOnly(_) => Ok(0),
        }
    }

    pub fn write_ram_slice(&mut self, start: usize, data: &[u8]) -> Result<usize, String> {
        match self {
            MBC::MBC1(mbc) => mbc.ram.write_bytes(start, data),
            MBC::MBC2(mbc) => mbc.ram.write_bytes(start, data),
            MBC::MBC3(mbc) => mbc.ram.write_bytes(start, data),
            MBC::MBC5(mbc) => mbc.ram.write_bytes(start, data),
            MBC::RomOnly(_) => Ok(0),
        }
    }
}