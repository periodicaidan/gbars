use std::vec::Vec;
use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::fmt;

use super::utils::*;

const CLOCK_SPEED: u32 = 4_194_304;

//lazy_static! {
//    pub static ref INSTRUCTIONS: [Option<Instruction>; 256] = [
//
//    ]
//}

pub trait Serial {
    fn serialize(&self) -> String;
    fn deserialize(_: String) -> Self;
}

pub struct Console {
    pub cpu: CPU,
    pub ram: Memory,
    pub rom: Option<Cartridge>
}

impl Console {
    pub fn init(path_to_rom: Option<&str>) -> Self {
        let rom = if let Some(path) = path_to_rom {
            let c = Cartridge::load(path);
            match c {
                Ok(rom) => Some(rom),
                Err(why) => {
                    println!("Error trying to load {}: {}", path, why);
                    None
                }
            }
        } else {
            None
        };

        let mut ram = Memory::init();

        Self {
            cpu: CPU::init(),
            ram,
            rom
        }
    }

    pub fn load_rom(&mut self, path_to_rom: &str) {
        self.rom = match Cartridge::load(path_to_rom) {
            Ok(rom) => Some(rom),
            Err(why) => {
                println!("Error trying to load {}: {}", path_to_rom, why);
                None
            }
        };
    }

    // TODO
    pub fn step_seconds(&mut self, mut dt: f64) -> f64 {
        if dt < 1.0 / CLOCK_SPEED as f64 && dt > 0.0 {
            return dt;
        }

        while dt > 0.0 {
            println!("{}", dt);
            dt -= 1.0 / CLOCK_SPEED as f64
//            match self.cpu.step() {
//                CPUStatus::Continue => dt -= 1.0 / CLOCK_SPEED as f64,
//                _ => {}
//            }
        }

        if dt < 0.0 { 0.0 } else { dt }
    }

    pub fn step(&mut self) -> Result<(), String> {
        let opcode = read_byte(self, self.cpu.pc);
        match self.exec() {
            CPUStatus::Continue => Ok(()),
            _ => Ok(())
        }
    }

    pub fn exec(&mut self) -> CPUStatus {
        use self::CPUStatus::*;
        use self::Register::*;

        let opcode = read_byte(&self, self.cpu.pc);

//        println!("Opcode: {:02X} PC: {:04X}", opcode, self.cpu.pc);

        match opcode {
            0x00 => {
                println!("nop")
            },
            0x01 => {
                let lo_byte = read_byte(&self, self.cpu.pc + 1);
                let hi_byte = read_byte(&self, self.cpu.pc + 2);

                self.cpu.load_16bit_literal(BC, (hi_byte as u16) << 8 | lo_byte as u16);
                self.cpu.pc += 2;
                println!("ld BC, ${:02X}{:02X}", hi_byte, lo_byte);
            },
            0x06 => {
                let byte = read_byte(&self, self.cpu.pc + 1);
                self.cpu.load_8bit_literal(B, byte);
                self.cpu.pc += 1;
                println!("ld B, ${:02X}", byte);
            },
            0x10 => {
                println!("stop $00");
                return Stop;
            },
            0x20 => {
                let rel = read_byte(&self, self.cpu.pc + 1) as i8;
                println!("jr NZ, ${:02X}", rel);
                if self.cpu.get_zero() != 1 {
                    // relative jump

                    if rel < 0 {
                        self.cpu.pc -= (rel as f64).abs() as u16;
                    } else {
                        self.cpu.pc += rel as u16;
                    }

                    self.cpu.pc -= 1;
                }
            },
            0x21 => {
                let lo_byte = read_byte(&self, self.cpu.pc + 1);
                let hi_byte = read_byte(&self, self.cpu.pc + 2);
                println!("ld HL, ${:02X}{:02X}", hi_byte, lo_byte);
                let addr = (hi_byte as u16) << 8 | lo_byte as u16;
                self.cpu.load_16bit_literal(HL, addr);
                self.cpu.pc += 2;
            },
            0x3B => {
                println!("dec SP");
                self.cpu.sp.wrapping_sub(1);
            },
            0x3C => {
                println!("inc A");
                self.cpu.inc(A);
            },
            0x44 => {
                println!("ld B, H");
                self.cpu.load_8bit_reg(B, H);
            },
            0x4D => {
                println!("ld C, L");
                self.cpu.load_8bit_reg(C, L);
            },
            0x50 => {
                println!("ld D, B");
                self.cpu.load_8bit_reg(D, B);
            },
            0x51 => {
                println!("ld D, C");
                self.cpu.load_8bit_reg(D, C);
            },
            0x61 => {
                println!("ld H, C");
                self.cpu.load_8bit_reg(H, C);
            },
            0x76 => {
                println!("halt");
                return Halt;
            },
            0x78 => {
                println!("ld A, B");
                self.cpu.load_8bit_reg(A, B);
            },
            0x7E => {
                println!("ld A, (HL)");
                let addr = self.cpu.get_hl();
                let val = self.ram.read_byte(addr);
                self.cpu.a = val;
            },
            0x7F => {
                println!("ld A, A");
                self.cpu.load_8bit_reg(A, A);
            },
            0xC1 => {
                println!("pop BC");
                self.cpu.pop(BC);
            },
            0xC3 => {
                let lo_byte = read_byte(&self, self.cpu.pc + 1);
                let hi_byte = read_byte(&self, self.cpu.pc + 2);
                println!("jp ${:02X}{:02X}", hi_byte, lo_byte);
                let addr = (hi_byte as u16) << 8 | lo_byte as u16;
                self.cpu.pc = addr - 1;
            },
            0xC5 => {
                println!("push BC");
                self.cpu.push(BC);
            },
            0xCB => {
                let prefixed_opcode = read_byte(self, self.cpu.pc + 1);
                let div = prefixed_opcode / 8;
                let rem = prefixed_opcode % 8;

                let target = match rem {
                    0 => B,
                    1 => C,
                    2 => D,
                    3 => E,
                    4 => H,
                    5 => L,
                    6 => Address(&mut self.ram, self.cpu.get_hl()),
                    7 => A,
                    _ => panic!("Invalid remainder for division by 8")
                };

                match div {
                    0 => self.cpu.rlc(target),
                    1 => self.cpu.rrc(target),
                    2 => self.cpu.rl(target),
                    3 => self.cpu.rr(target),
                    4 => self.cpu.sla(target),
                    5 => self.cpu.sra(target),
                    6 => self.cpu.swap(target),
                    7 => self.cpu.srl(target),
                    8...15 => self.cpu.check_bit(div - 8, target),
                    16...23 => self.cpu.reset_bit(div - 16, target),
                    24...31 => self.cpu.set_bit(div - 24, target),
                    _ => panic!("Invalid quotient for division of u8 by 8")
                }

                match prefixed_opcode {
                    0x7F => {
                        println!("bit 7, A");
                        let a = self.cpu.a;
                        self.cpu.set_flags(
                            Some(0b1000_0000 & a),
                            Some(0),
                            Some(1),
                            None
                        );
                    },
                    _ => panic!("UNKNOWN OPCODE WITH PREFIX CB: {:02X}", prefixed_opcode)
                }

                self.cpu.pc += 1;
            },
            0xE0 => {
                let lo_byte = read_byte(&self, self.cpu.pc + 1);
                println!("ldh (${:02X}), A", lo_byte);
                let data = self.cpu.a;
                write_byte(self, 0xFF00 | lo_byte as u16, data);
                self.cpu.pc += 1;
            },
            0xE9 => {
                println!("jp (HL)");
                self.cpu.pc = self.cpu.get_hl()},
            0xEA => {
                let hi_byte = read_byte(&self, self.cpu.pc + 1);
                let lo_byte = read_byte(&self, self.cpu.pc + 2);
                println!("ld (${:02X}{:02X}), A", hi_byte, lo_byte);
                let addr = (lo_byte as u16) << 8 | hi_byte as u16;
                write_byte(self, addr, self.cpu.a);
                self.cpu.pc += 2;
            },
            0xF0 => {
                // Load data from an IO register into A
                let lo_byte = read_byte(&self, self.cpu.pc + 1);
                println!("ldh A, (${:02X})", lo_byte);
                let data = read_byte(&self, 0xFF00 | lo_byte as u16);
                self.cpu.a = data;
                self.cpu.pc += 1;
            },
            0xF3 => {
                println!("di");
                self.ram.write_byte(0xFFFF, 0);
            },
            0xF5 => {
                println!("push AF");
                self.cpu.push(AF);
            },
            0xFB => {
                println!("ei");
            },
            0xFE => {
                let other = read_byte(&self, self.cpu.pc + 1);
                println!("cmp ${:02X}", other);
                self.cpu.cmp(other);
                self.cpu.pc += 1;
            },
            _ => {
                panic!("UNKNOWN OPCODE: {:02X}", opcode)
            }
        }

        self.cpu.pc += 1;

        Continue
    }
}

pub enum CPUStatus {
    Continue,
    Halt,
    Stop,
    Start,
    Interrupt
}

#[derive(Debug)]
pub enum Register<'r> {
    A, B, C, D, E, F, H, L,
    AF, BC, DE, HL, SP, PC,
    Address(&'r mut Memory, u16)
}


// CPU Things
pub struct CPU {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub stack: Vec<u16>,
    pub sp: u16,
    pub pc: u16
}

impl CPU {
    pub fn init() -> Self {
        CPU {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: 0xB0,
            h: 0x01,
            l: 0x4D,
            stack: vec![0; 0x10_000],
            sp: 0xFFFE,
            pc: 0x100
        }
    }
}

pub struct Instruction {
    pub opcode: u8,
    pub min_cycles: u8,
    pub max_cycles: u8,
    pub length: u8,
    pub steps: u16,
}

#[derive(Debug)]
pub struct Memory {
    inner: Vec<u8>,
//    rom0: Vec<u8>, // non-switchable ROM bank
//    romx: Vec<u8>, // switchable ROM bank
//    vram: Vec<u8>,
//    sram: Vec<u8>,
//    wram0: Vec<u8>,
//    wramx: Vec<u8>,
//    echo: Vec<u8>,
//    oam: Vec<u8>,
//    unused: Vec<u8>,
//    io_reg: Vec<u8>,
//    hram: Vec<u8>,
//    ie: u8
}

impl Memory {
    pub fn init() -> Self {
        Self {
            inner: make_vec(0x10_000)
        }
    }

    pub fn get(&self, idx: usize) -> Option<&u8> {
        self.inner.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut u8> {
        self.inner.get_mut(idx)
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.inner[addr as usize]
    }

    pub fn read_buf(&self, start: u16, end: u16) -> Vec<u8> {
        self.inner[start as usize..end as usize].to_vec()
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        self.inner[addr as usize] = byte;
    }

    pub fn write_buf(&mut self, addr: u16, buf: &[u8]) {
        for (i, byte) in buf.iter().enumerate() {
            self.inner[addr as usize + i] = *byte;
        }
    }
}

pub enum JoyPad {
    None,
    Start, Select, B, A,
    Down, Up, Left, Right
}

impl From<&Memory> for JoyPad {
    fn from(m: &Memory) -> Self {
        let joypad_reg: u8 = m.read_byte(0xFF00);

        match joypad_reg & 0b0011_1111 {
            0b0010_1000 => JoyPad::Start,
            0b0010_0100 => JoyPad::Select,
            0b0010_0010 => JoyPad::B,
            0b0010_0001 => JoyPad::A,
            0b0001_1000 => JoyPad::Down,
            0b0001_0100 => JoyPad::Up,
            0b0001_0010 => JoyPad::Left,
            0b0001_0001 => JoyPad::Right,
            _ => JoyPad::None
        }
    }
}

// Color palette stuff

// Two palettes: Monochrome (for GB) and Color (for CGB)
pub enum Palette {
    Monochrome,
    Color
}

// The monochrome color palette has four shades corresponding to every combination of 4 bits
pub enum MonoShade {
    White,
    LightGrey,
    DarkGrey,
    Black
}

impl From<u8> for MonoShade {
    fn from(b: u8) -> Self {
        match b {
            0 => MonoShade::White,
            1 => MonoShade::LightGrey,
            2 => MonoShade::DarkGrey,
            3 => MonoShade::Black,
            _ => MonoShade::Black
        }
    }
}

// A color palette on the GB is taken from offset $FF47 in RAM
// It consists of 4 2-bit values representing one of four shades
pub struct MonoPaletteData(MonoShade, MonoShade, MonoShade, MonoShade);

impl From<u8> for MonoPaletteData {
    fn from(b: u8) -> Self {
        let color0 = (0b0000_0011 & b);
        let color1 = (0b0000_1100 & b) >> 2;
        let color2 = (0b0011_0000 & b) >> 4;
        let color3 = (0b1100_0000 & b) >> 6;

        MonoPaletteData(
            MonoShade::from(color0),
            MonoShade::from(color1),
            MonoShade::from(color2),
            MonoShade::from(color3)
        )
    }
}

pub struct MonoPalettes {
    background: MonoPaletteData,
    sprites: [MonoPaletteData; 2]
}

impl From<Memory> for MonoPalettes {
    fn from(m: Memory) -> Self {
        let bg = m.get(0xFF47).unwrap() as &u8;
        let sp0 = m.get(0xFF48).unwrap() as &u8;
        let sp1 = m.get(0xFF49).unwrap() as &u8;

        MonoPalettes {
            background: MonoPaletteData::from(*bg),
            sprites: [
                MonoPaletteData::from(*sp0),
                MonoPaletteData::from(*sp1)
            ]
        }
    }
}


pub type RGB = (u8, u8, u8);


pub enum MonoShadeColors {
    // _        A           B
    Brown,      Red,        DarkBrown,  // Up
    Pastel,     Orange,     Yellow,     // Down
    Blue,       DarkBlue,   Greyscale,  // Left
    Green,      DarkGreen,  Inverted,   // Right

    Custom { background: Vec<RGB>, sprite0: Vec<RGB>, sprite1: Vec<RGB> }
}

impl MonoShadeColors {
    pub fn custom_from_rgb(bg: &[RGB; 4], sp0: &[RGB; 4], sp1: &[RGB; 4]) -> Self {
        MonoShadeColors::Custom {
            background: bg.to_vec(),
            sprite0: sp0.to_vec(),
            sprite1: sp1.to_vec()
        }
    }

    pub fn to_rgb(&self) -> Vec<RGB> {
        match self {
            MonoShadeColors::Brown => vec![
                (255, 255, 255), (255, 173, 99), (131, 49, 0), (0, 0, 0),
                (255, 255, 255), (255, 173, 99), (131, 49, 0), (0, 0, 0),
                (255, 255, 255), (255, 173, 99), (131, 49, 0), (0, 0, 0)
            ],

            MonoShadeColors::Red => vec![
                (255, 255, 255), (255, 133, 132), (148, 58, 58), (0, 0, 0),
                (255, 255, 255), (101, 164, 155), (0, 0, 254), (0, 0, 0),
                (255, 255, 255), (123, 255, 48), (0, 131, 0), (0, 0, 0)
            ],

            MonoShadeColors::DarkBrown => vec![
                (255, 255, 255), (255, 173, 99), (131, 49, 0), (0, 0, 0),
                (255, 255, 255), (255, 173, 99), (131, 49, 0), (0, 0, 0),
                (255, 231, 197), (206, 156, 133), (132, 107, 41), (91, 49, 9)
            ],

            MonoShadeColors::Pastel => vec![
                (255, 255, 165), (254, 148, 148), (147, 148, 254), (0, 0, 0),
                (255, 255, 165), (254, 148, 148), (147, 148, 254), (0, 0, 0),
                (255, 255, 165), (254, 148, 148), (147, 148, 254), (0, 0, 0)
            ],

            MonoShadeColors::Orange => vec![
                (255, 255, 255), (255, 255, 0), (254, 0, 0), (0, 0, 0),
                (255, 255, 255), (255, 255, 0), (254, 0, 0), (0, 0, 0),
                (255, 255, 255), (255, 255, 0), (254, 0, 0), (0, 0, 0)
            ],

            MonoShadeColors::Yellow => vec![
                (255, 255, 255), (255, 255, 0), (125, 73, 0), (0, 0, 0),
                (255, 255, 255), (101, 164, 155), (0, 0, 254), (0, 0, 0),
                (255, 255, 255), (123, 255, 48), (0, 131, 0), (0, 0, 0)
            ],

            MonoShadeColors::Blue =>vec![
                (255, 255, 255), (101, 164, 155), (0, 0, 254), (0, 0, 0),
                (255, 255, 255), (255, 133, 132), (131, 49, 0), (0, 0, 0),
                (255, 255, 255), (123, 255, 48), (0, 131, 0), (0, 0, 0)
            ],

            MonoShadeColors::DarkBlue => vec![
                (255, 255, 255), (139, 140, 222), (83, 82, 140), (0, 0, 0),
                (255, 255, 255), (255, 133, 132), (148, 58, 58), (0, 0, 0),
                (255, 255, 255), (255, 173, 99), (131, 49, 0), (0, 0, 0)
            ],

            MonoShadeColors::Greyscale => vec![
                (255, 255, 255), (165, 165, 165), (82, 82, 82), (0, 0, 0),
                (255, 255, 255), (165, 165, 165), (82, 82, 82), (0, 0, 0),
                (255, 255, 255), (165, 165, 165), (82, 82, 82), (0, 0, 0)
            ],

            MonoShadeColors::Green => vec![
                (255, 255, 255), (81, 255, 0), (255, 66, 0), (0, 0, 0),
                (255, 255, 255), (81, 255, 0), (255, 66, 0), (0, 0, 0),
                (255, 255, 255), (81, 255, 0), (255, 66, 0), (0, 0, 0)
            ],

            MonoShadeColors::DarkGreen => vec![
                (255, 255, 255), (123, 255, 48), (1, 99, 198), (0, 0, 0),
                (255, 255, 255), (255, 133, 132), (148, 58, 58), (0, 0, 0),
                (255, 255, 255), (255, 133, 132), (148, 58, 58), (0, 0, 0)
            ],

            MonoShadeColors::Inverted => vec![
                (0, 0, 0), (0, 132, 134), (255, 222, 0), (255, 255, 255),
                (0, 0, 0), (0, 132, 134), (255, 222, 0), (255, 255, 255),
                (0, 0, 0), (0, 132, 134), (255, 222, 0), (255, 255, 255)
            ],

            MonoShadeColors::Custom { background, sprite0, sprite1 } => {
                let mut v: Vec<RGB> = vec![];
                v.extend(background);
                v.extend(sprite0);
                v.extend(sprite1);

                v
            }
        }
    }
}

pub enum ScrollDirection {
    Up, Down, Left, Right
}

pub struct ScreenBuffer {
    pub pixels: Vec<u8>,
//    pub colors: ScreenBufferPalette,
    pub scale: usize,
    pub scy: usize, // $FF42
    pub scx: usize, // $FF43
    pub ly: usize, // $FF44
    pub lyc: u8, // $FF45
    pub wy: usize, // $FF4A
    pub wx: usize  // $FF4B
}

impl ScreenBuffer {
    pub const DIMENSION: usize = 256;
    pub const VISIBLE_X: usize = 160;
    pub const VISIBLE_Y: usize = 144;

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<&u8> {
        self.pixels.get(Self::DIMENSION * y + x)
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        self.pixels[Self::DIMENSION * y + x] = value;
    }

    pub fn get_visible(&self) -> Vec<u8> {
        let mut vis: Vec<u8> = Vec::with_capacity(Self::VISIBLE_X * Self::VISIBLE_Y);

        for y in self.scy..self.scy + Self::VISIBLE_Y {
            let scanline = &self.pixels[Self::DIMENSION * y..Self::DIMENSION * y + self.scx + Self::VISIBLE_X];
            vis.extend_from_slice(&scanline);
        }

        vis
    }

    pub fn get_tile(&self, tx: usize, ty: usize) {}

    pub fn draw_sprite(&mut self, x: usize, y: usize) {

    }

    pub fn scroll(&mut self, dir: ScrollDirection, value: usize) {
        // The top-left corner is where the scroll is normalized to
        match dir {
            ScrollDirection::Up => self.scy.wrapping_sub(value),
            ScrollDirection::Down => self.scy.wrapping_add(value),
            ScrollDirection::Left => self.scx.wrapping_sub(value),
            ScrollDirection::Right => self.scx.wrapping_add(value)
        };
    }

    pub fn gl_rgb_pixels(&self) -> Vec<f32> {
        let mut rgb_pixels: Vec<f32> = vec![];
        for pixel in &self.get_visible() {
            rgb_pixels.extend(match *pixel {
                0 => [1.0],
                1 => [165.0 / 255.0],
                2 => [82.0 / 255.0],
                3 => [0.0],
                _ => [0.0]
            }.iter().cycle().take(3));
        }

        rgb_pixels
    }

    pub fn scaled_dimension(&self) -> usize {
        Self::DIMENSION * self.scale
    }
}

impl From<&Memory> for ScreenBuffer {
    fn from(m: &Memory) -> Self {
        let scy= m.read_byte(0xFF42);
        let scx = m.read_byte(0xFF43);
        let ly = m.read_byte(0xFF44);
        let lyc = m.read_byte(0xFF45);
        let wy = m.read_byte(0xFF4A);
        let wx = m.read_byte(0xFF4B);

        Self {
            pixels: Vec::with_capacity(256 * 256),
            scale: 1,
            scy: scy as usize,
            scx: scx as usize,
            ly: ly as usize,
            lyc: lyc,
            wy: wy as usize,
            wx: wx as usize
        }
    }
}


pub enum SpriteSize {
    Eight, Sixteen
}


pub struct Sprite {
    size: SpriteSize,
    pixels: Vec<u8>
}

impl Sprite {

}

pub struct ToneSweepChannel {}
pub struct ToneChannel {}
pub struct WaveChannel {}
pub struct NoiseChannel {}

pub struct SoundTerminal {}

pub struct SoundController {}

#[derive(Debug, Clone, Copy)]
pub enum CartFeature {
    Unknown,
    ROM,
    RAM,
    MBC1, MBC2, MBC3, MBC5, MBC6, MBC7,
    MMM01,
    Battery,
    Timer,
    Rumble,
    Sensor,
    PocketCamera,
    BandaiTama5,
    HuC1, HuC3
}

pub struct Cartridge {
    pub title: String,
    pub contents: Vec<u8>,
    pub features: Vec<CartFeature>,
    pub rom_size: usize,
    pub rom_banks: usize,
    pub ram_size: usize,
    pub ram_banks: usize,
    pub locale: String,
    pub header_checksum: u8,
    pub global_checksum: u16
}

impl Cartridge {
    pub fn load(path_to_rom: &str) -> Result<Self, String> {
        match File::open(path_to_rom) {
            Ok(f) => {
                // Read in the contents of the ROM
                let mut contents: Vec<u8> = vec![];
                let mut reader = BufReader::new(f);
                reader.read_to_end(&mut contents);

                // get the title
                let mut title = String::new();
                for i in 0x134..0x143usize {
                    if let Some(ch) = contents.get(i) {
                        if *ch == 0x00 { continue; }
                        title.push(*ch as char);
                    }
                }

                // Create a list of cart features
                let features = {
                    use self::CartFeature::*;
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

                let (rom_size, rom_banks) = {
                    if let Some(n) = contents.get(0x148) {
                        match *n {
                            0x00 => (0x8_000, 1),
                            0x01...0x08 => (0x8_000usize << n, 2usize << n),
                            0x52 => (0x120_000, 72),
                            0x53 => (0x140_000, 80),
                            0x54 => (0x180_000, 96),
                            _ => (0, 0)
                        }
                    } else {
                        (0, 0)
                    }
                };

                let (ram_size, ram_banks) = {
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
                    }
                };

                let locale = {
                    if let Some(n) = contents.get(0x14A) {
                        match *n {
                            0 => "Japanese",
                            1 => "Non-Japanese",
                            _ => "Unknown"
                        }
                    } else {
                        "Unknown"
                    }
                }.to_string();

                let header_checksum: u8 = match contents.get(0x14D) {
                    Some(n) => *n,
                    None => 0
                };

                let global_checksum: u16 = {
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
                        contents,
                        features,
                        rom_size,
                        rom_banks,
                        ram_size,
                        ram_banks,
                        locale,
                        header_checksum,
                        global_checksum
                    }
                )
            },

            Err(why) => {
                Err(format!("Could not open file {}: {}", path_to_rom, why.description()))
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let scrolling_nintendo_graphic = [
            0xCEu8, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
            0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
            0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
            0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
            0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
            0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E
        ];

        for i in 0..48usize {
            if let Some(b) = self.contents.get(0x104 + i) {
                if *b != scrolling_nintendo_graphic[i] {
                    return Err(
                        format!(
                            "Byte 0x{:04X} must be 0x{:02X}; found 0x{:02X}",
                            0x104 + i,
                            scrolling_nintendo_graphic[i],
                            *b
                        )
                    );
                }
            } else {
                return Err(format!("Could not get byte {:04X} for validation", 0x104 + i))
            }
        }

        let mut sum = 0u8;
        for x in self.contents[0x134..0x14C].iter() {
            sum += *x;
        }

        if sum != self.header_checksum {
            return Err(format!("Invalid header checksum: Expected {}; actual sum is {}", self.header_checksum, sum))
        }

        Ok(())
    }

    pub fn dump(&self, as_chars: bool) {
        let mut hex_dump = String::new();
        if as_chars {
            for i in 0..self.contents.len() {
                if i % 16 == 0 {
                    hex_dump.push('\n');
                    hex_dump.push_str(&format!("0x{:08X} ", i));
                }

                let ch = self.contents[i];
                if ch.is_ascii_control() || ch.is_ascii_whitespace() {
                    hex_dump.push_str(". ");
                } else {
                    hex_dump.push_str(&format!("{} ", ch as char));
                }
            }
        } else {
            for i in 0..self.contents.len() {
                if i % 16 == 0 {
                    hex_dump.push('\n');
                    hex_dump.push_str(&format!("0x{:08X} ", i));
                }

                let ch = self.contents[i];
                hex_dump.push_str(&format!("{:02X} ", ch));
            }
        }


        println!("{}", &hex_dump);
    }
}

impl fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cartridge ( {}, ROM size: {}, {:?}, {} )", self.title, self.rom_size, self.features, self.locale)
    }
}

pub mod mbc {
    pub enum MemoryBankController {
        None,
        MBC1(MBC1<'static>),
        MBC2(MBC2<'static>),
        MBC3(MBC3<'static>),
        MBC5(MBC5<'static>),
    }

    pub enum MBCMode {
        ROM, RAM
    }

    pub struct MBC1<'m> {
        pub rom_bank_0: &'m [u8; 0x4000],
        pub rom_banks: Vec<&'m [u8; 0x4000]>,
        pub ram_banks: Vec<super::Memory>,
        pub ram_enabled: bool,
        pub current_rom_bank: u8,
        pub current_ram_bank: u8,
        pub mode: MBCMode
    }

    pub struct MBC2<'m> {
        pub rom_bank_0: &'m [u8; 0x4000],
        pub rom_banks: Vec<&'m [u8; 0x4000]>,
        pub ram_enabled: &'m [bool; 4],
        pub current_rom_bank: u8,
        pub ram: super::Memory // Yes, a memory bank controller is also memory. It's weird.
    }

    pub struct MBC3<'m> {
        pub rom_bank_0: &'m [u8; 0x4000],
        pub rom_banks: Vec<&'m [u8; 0x4000]>,
        pub ram_banks: Vec<super::Memory>,
        pub ram_timer_enabled: bool,
        pub current_rom_bank: u8,
        pub current_ram_timer_bank: u8,
    }

    pub struct MBC5<'m> {
        pub rom_bank_0: &'m [u8; 0x4000],
        pub rom_banks: Vec<&'m [u8;0x4000]>,
        pub ram_banks: Vec<super::Memory>,
        pub ram_enabled: bool,
        pub current_rom_bank: u16,
        pub current_ram_bank: u8
    }

    pub fn set_rom_bank(controller: &mut MemoryBankController, bank: u16) {
        use self::MemoryBankController as Controller;
        match controller {
            Controller::None => {},
            Controller::MBC1(mbc1) => mbc1.current_rom_bank = bank as u8,
            Controller::MBC2(mbc2) => mbc2.current_rom_bank = bank as u8,
            Controller::MBC3(mbc3) => mbc3.current_rom_bank = bank as u8,
            Controller::MBC5(mbc5) => mbc5.current_rom_bank = bank
        };
    }

    pub fn set_ram_bank(controller: &mut MemoryBankController, bank: u8) {
        use self::MemoryBankController as Controller;
        match controller {
            Controller::None => {},
            Controller::MBC1(mbc1) => mbc1.current_ram_bank = bank,
            Controller::MBC2(_) => {},
            Controller::MBC3(mbc3) => mbc3.current_ram_timer_bank = bank,
            Controller::MBC5(mbc5) => mbc5.current_ram_bank = bank
        }
    }
}