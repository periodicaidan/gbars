use std::ops::{Deref, DerefMut};
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, Read, Write};
use core::fmt;
use std::time::{Instant, SystemTime};

use super::memory::*;

/// Represents a physical GB cartridge and its associated metadata
pub struct Cartridge {
    pub title: String,
    // The Cartridge holds an MBC that holds the ROM, rather than holding ROM directly
    // If the Cartridge doesn't have an MBC, this will just be ROM
    pub mbc: MBC,
    pub features: Vec<CartridgeFeature>,
    pub rom_size: usize,
    pub rom_banks: usize,
    pub ram_size: usize,
    pub ram_banks: usize,
    pub locale: String,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cartridge ( {}, ROM size: {}, {:?}, {} )", self.title, self.rom_size, self.features, self.locale)
    }
}

/// All the possible features of a cartridge
#[derive(Debug, PartialEq)]
pub enum CartridgeFeature {
    Unknown,
    ROM, // If it has no MBC
    RAM, // Some cartridges have extra RAM for things like saves
    MBC1, MBC2, MBC3, MBC5, MBC6, MBC7, // Memory Bank Controllers
    MMM01, // A weird special kind of MBC
    Battery, // Games used batteries for things like saving and in-game time
    Timer,
    Rumble,
    Sensor,
    PocketCamera, // GameBoy Camera, baby!!
    BandaiTama5, // Some Tamagotchi thing idk
    HuC1, HuC3, // MBCs for some HudsonSoft games. I believe they have IR capabilities
}

impl Cartridge {
    /// Loads up a ROM from a file and returns a new Cartridge object on success, or an error
    pub fn load(path_to_rom: &str) -> Result<Self, String> {
        match File::open(path_to_rom)  {
            Ok(f) => {
                // Read the contents of the ROM
                let mut contents = vec![];
                {
                    let mut reader = BufReader::new(f);
                    if let Err(e) = reader.read_to_end(&mut contents) {
                        return Err(format!("Error reading data from {}: {}", path_to_rom, e.description()));
                    }
                }

                // Get the title
                let title = {
                    let mut t = String::new();
                    for i in 0x134..0x143usize {
                        if let Some(ch) = contents.get(i) {
                            if *ch == 0x00 { continue; }
                            t.push(*ch as char);
                        }
                    }
                    t
                };

                // Specify the list of features
                let features = {
                    use self::CartridgeFeature::*;
                    if let Some(n) = contents.get(0x147) {
                        match *n {
                            0x00 => vec![ROM],
                            0x01 => vec![MBC1],
                            0x02 => vec![MBC1, RAM],
                            0x03 => vec![MBC1, RAM, Battery],
                            0x05 => vec![MBC2],
                            0x06 => vec![MBC2, Battery],
                            0x08 => vec![ROM, RAM],
                            0x09 => vec![ROM, RAM, Battery],
                            0x0B => vec![MMM01],
                            0x0C => vec![MMM01, RAM],
                            0x0D => vec![MMM01, RAM, Battery],
                            0x0F => vec![MBC3, Battery, Timer],
                            0x10 => vec![MBC3, Battery, Timer, RAM],
                            0x11 => vec![MBC3],
                            0x12 => vec![MBC3, RAM],
                            0x13 => vec![MBC3, RAM, Battery],
                            0x19 => vec![MBC5],
                            0x1A => vec![MBC5, RAM],
                            0x1B => vec![MBC5, RAM, Battery],
                            0x1C => vec![MBC5, Rumble],
                            0x1D => vec![MBC5, Rumble, RAM],
                            0x1E => vec![MBC5, Rumble, RAM, Battery],
                            0x20 => vec![MBC6],
                            0x22 => vec![MBC7, Sensor, Rumble, RAM, Battery],
                            0xFC => vec![PocketCamera],
                            0xFD => vec![BandaiTama5],
                            0xFE => vec![HuC3],
                            0xFF => vec![HuC1, RAM, Battery],
                            _    => vec![Unknown]
                        }
                    } else {
                        vec![Unknown]
                    }
                };

                // Get the ROM size and the number of ROM banks
                let (rom_size, rom_banks) =
                    if let Some(n) = contents.get(0x148) {
                        match *n {
                            0x00 => (0x8_000, 1),
                            0x01...0x08 => ((0x8_000 << *n) as usize, (2 << *n) as usize),
                            0x52 => (0x120_000, 72),
                            0x53 => (0x140_000, 80),
                            0x54 => (0x180_000, 96),
                            _ => (0, 0)
                        }
                    } else {
                        (0, 0)
                    };

                // Get the RAM size (if applicable) and the number of RAM banks
                let (ram_size, ram_banks) =
                    if let Some(n) = contents.get(0x149) {
                        match *n {
                            0x00 => (0, 0),
                            0x01 => (0x800, 1),
                            0x02 => (0x2_000, 1),
                            0x03 => (0x8_000, 4),
                            0x04 => (0x20_000, 16),
                            0x05 => (0x10_000, 8),
                            _ => (0, 0)
                        }
                    } else {
                        (0, 0)
                    };

                // Get the memory bank controller, which is part of the features
                // Currently only four are documented, but they cover most cases. MBC6, MBC7,
                // MMM01, and the HudsonSoft MBCs were not very prevalent
                let mbc = if features.contains(&CartridgeFeature::MBC1) {
                    MBC::MBC1(MBC1 {
                        rom: ROM::new(contents.clone()),
                        ram: RAM::new(ram_size),
                        active_rom_bank: 1,
                        active_ram_bank: 1,
                        ram_enabled: false,
                        mode: MbcMode::RomSelect,
                    })
                } else {
                    MBC::RomOnly(ROM::new(contents.clone()))
                };

                // Two locales: Japanese and Non-Japanese
                let locale = if let Some(n) = contents.get(0x14A) {
                    match *n {
                        0 => "Japanese",
                        1 => "Non-Japanese",
                        _ => "Unknown"
                    }
                } else {
                    "Unknown"
                }.to_string();

                // Get the header checksum, which is one byte long
                let header_checksum = match contents.get(0x14D) {
                    Some(n) => *n,
                    None => 0
                };

                // Get the global checksum, which is two bytes long
                let global_checksum = {
                    let upper_byte = match contents.get(0x14E) {
                        Some(n) => *n,
                        None => 0
                    } as u16;

                    let lower_byte = match contents.get(0x14F) {
                        Some(n) => *n,
                        None => 0
                    } as u16;

                    upper_byte << 8 | lower_byte
                };

                Ok(
                    Self {
                        title,
                        mbc,
                        features,
                        rom_size,
                        rom_banks,
                        ram_size,
                        ram_banks,
                        locale,
                        header_checksum,
                        global_checksum,
                    }
                )
            },
            Err(e) => Err(format!("Could not open file {}: {}", path_to_rom, e.description())),
        }
    }

    /// There are two criteria that the GameBoy checks for to validate ROMs: the scrolling
    /// NintendoⓇ graphic and the header checksum.
    ///
    /// As I was reading the docs for this bit it struck me just how pitiful of a security measure
    /// this is. You can basically just stick the header of an officially-licensed GameBoy game onto
    /// whatever you want and the GameBoy should have no problem trying to play it.
    pub fn validate(&self) -> Result<(), String> {
        // The scrolling NintendoⓇ graphic is a short program that runs when you turn on the GB (it
        // does exactly what you think). It is 48 bytes long, starting at offset 0x104, and must be
        // exactly as follows
        let scrolling_nintendo_graphic = [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
            0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
            0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
            0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
            0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
            0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
        ];

        // Rather than doing a slice comparison I'm checking each value in a loop for better
        // debugging and error reporting.
        for i in 0..48usize {
            if let Some(b) = self.mbc.read_rom(0x104 + i) {
                if b != scrolling_nintendo_graphic[i] {
                    return Err(
                        format!(
                            "Error validating Nintendo graphic: Byte at offset 0x{:04X} must be 0x{:02X}; found 0x{:02X}",
                            0x104 + i,
                            scrolling_nintendo_graphic[i],
                            b
                        )
                    );
                }
            } else {
                return Err(format!("Could not get byte {:04X} for validation", 0x104 + i))
            }
        }

        // The checksum starts from 0 and the value of one less than each byte from offset 0x0134 to
        // 0x014D is subtracted from it (with wrapping)
        let mut checksum = 0u8;
        for x in self.mbc.read_rom_slice(0x134, 0x14D).unwrap().iter() {
            // checksum = checksum - x - 1
            checksum = checksum.wrapping_sub(*x).wrapping_sub(1);
        }

        if checksum != self.header_checksum {
            return Err(
                format!(
                    "Invalid header checksum: Expected {}; actual sum is {}",
                    self.header_checksum,
                    checksum
                )
            )
        }

        Ok(())
    }

    /// Returns true if the result of `validate` is `Ok`.
    pub fn is_valid(&self) -> bool { self.validate().is_ok() }

    pub fn read_rom(&self, offset: usize) -> Option<u8> {
        self.mbc.read_rom(offset)
    }
}