#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::{String, ToString};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub prefixed: bool,
    pub asm: String,
    pub arg: Arg,
    pub cycles: (usize, usize), // min, max
}

#[derive(Clone, Debug)]
pub enum Arg {
    None,
    Data8(u8),
    Data16(u16),
    Addr8(u8),
    Addr16(u16),
    Offset8(i8),
}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Self {
        INSTRUCTIONS[opcode as usize].clone()
    }

    fn new(
        opcode: u8,
        asm: &str,
        arg: Arg,
        min_cycles: usize,
        max_cycles: usize
    ) -> Self {
        Self {
            opcode,
            prefixed: false,
            asm: asm.to_string(),
            arg,
            cycles: (min_cycles, max_cycles),
        }
    }

    pub(crate) fn prefixed(
        opcode: u8,
        asm: &str
    ) -> Self {
        Self {
            opcode,
            prefixed: true,
            asm: asm.to_string(),
            arg: Arg::None,
            cycles: (8, 8),
        }
    }

    fn none(opcode: u8) -> Self {
        Self {
            opcode,
            prefixed: false,
            asm: String::new(),
            arg: Arg::None,
            cycles: (0, 0)
        }
    }
}

impl Arg {
    fn d8() -> Self { Arg::Data8(0) }
    fn d16() -> Self { Arg::Data16(0) }
    fn a8() -> Self { Arg::Addr8(0) }
    fn a16() -> Self { Arg::Addr16(0) }
    fn r8() -> Self { Arg::Offset8(0) }
}

lazy_static!{
    static ref INSTRUCTIONS: [Instruction; 256] = [
        Instruction::new(0x00, "nop", Arg::None, 4, 4),
        Instruction::new(0x01, "ld BC, <d16>", Arg::d16(), 12, 12),
        Instruction::new(0x02, "ld (BC), A", Arg::None, 8, 8),
        Instruction::new(0x03, "inc BC", Arg::None, 8, 8),
        Instruction::new(0x04, "inc B", Arg::None, 4, 4),
        Instruction::new(0x05, "dec B", Arg::None, 4, 4),
        Instruction::new(0x06, "ld B, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0x07, "rlca", Arg::None, 4, 4),

        Instruction::new(0x08, "ld (<a16>), SP", Arg::a16(), 20, 20),
        Instruction::new(0x09, "add HL, BC", Arg::None, 8, 8),
        Instruction::new(0x0A, "ld A, (BC)", Arg::None, 8, 8),
        Instruction::new(0x0B, "dec BC", Arg::None, 8, 8),
        Instruction::new(0x0C, "inc C", Arg::None, 4, 4),
        Instruction::new(0x0D, "dec C", Arg::None, 4, 4),
        Instruction::new(0x0E, "ld C, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0x0F, "rrca", Arg::None, 4, 4),

        Instruction::new(0x10, "stop $00", Arg::d8(), 4, 4),
        Instruction::new(0x11, "ld DE, <d16>", Arg::d16(), 12, 12),
        Instruction::new(0x12, "ld (DE), A", Arg::None, 8, 8),
        Instruction::new(0x13, "inc DE", Arg::None, 8, 8),
        Instruction::new(0x14, "inc D", Arg::None, 4, 4),
        Instruction::new(0x15, "dec D", Arg::None, 4, 4),
        Instruction::new(0x16, "ld D, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0x17, "rla", Arg::None, 4, 4),

        Instruction::new(0x18, "jr <r8>", Arg::r8(), 12, 12),
        Instruction::new(0x19, "add HL, DE", Arg::None, 8, 8),
        Instruction::new(0x1A, "ld A, (DE)", Arg::None, 8, 8),
        Instruction::new(0x1B, "dec DE", Arg::None, 8, 8),
        Instruction::new(0x1C, "inc E", Arg::None, 4, 4),
        Instruction::new(0x1D, "dec E", Arg::None, 4, 4),
        Instruction::new(0x1E, "ld E, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0x1F, "rra", Arg::None, 4, 4),

        Instruction::new(0x20, "jr nz, <r8>", Arg::r8(), 8, 12),
        Instruction::new(0x21, "ld HL, <d16>", Arg::d16(), 12, 12),
        Instruction::new(0x22, "ld (HL+), A", Arg::None, 8, 8),
        Instruction::new(0x23, "inc HL", Arg::None, 8, 8),
        Instruction::new(0x24, "inc H", Arg::None, 4, 4),
        Instruction::new(0x25, "dec H", Arg::None, 4, 4),
        Instruction::new(0x26, "ld H, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0x27, "daa", Arg::None, 4, 4),

        Instruction::new(0x28, "jr z, <r8>", Arg::r8(), 8, 12),
        Instruction::new(0x29, "add HL, HL", Arg::None, 8, 8),
        Instruction::new(0x2A, "ld A, (HL+)", Arg::None, 8, 8),
        Instruction::new(0x2B, "dec HL", Arg::None, 8, 8),
        Instruction::new(0x2C, "inc L", Arg::None, 4, 4),
        Instruction::new(0x2D, "dec L", Arg::None, 4, 4),
        Instruction::new(0x2E, "ld L, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0x2F, "cpl", Arg::None, 4, 4),

        Instruction::new(0x30, "jr nc, <r8>", Arg::r8(), 8, 12),
        Instruction::new(0x31, "ld SP, <d16>", Arg::d16(), 12, 12),
        Instruction::new(0x32, "ld (HL-), A", Arg::None, 8, 8),
        Instruction::new(0x33, "inc SP", Arg::None, 8, 8),
        Instruction::new(0x34, "inc (HL)", Arg::None, 12, 12),
        Instruction::new(0x35, "dec (HL)", Arg::None, 12, 12),
        Instruction::new(0x36, "ld (HL), <d8>", Arg::d8(), 12, 12),
        Instruction::new(0x37, "scf", Arg::None, 4, 4),

        Instruction::new(0x38, "jr c, <r8>", Arg::r8(), 8, 12),
        Instruction::new(0x39, "add HL, SP", Arg::None, 8, 8),
        Instruction::new(0x3A, "ld A, (HL-)", Arg::None, 8, 8),
        Instruction::new(0x3B, "dec SP", Arg::None, 8, 8),
        Instruction::new(0x3C, "inc A", Arg::None, 4, 4),
        Instruction::new(0x3D, "dec A", Arg::None, 4, 4),
        Instruction::new(0x3E, "ld A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0x3F, "ccf", Arg::None, 4, 4),

        Instruction::new(0x40, "ld B, B", Arg::None, 4, 4),
        Instruction::new(0x41, "ld B, C", Arg::None, 4, 4),
        Instruction::new(0x42, "ld B, D", Arg::None, 4, 4),
        Instruction::new(0x43, "ld B, E", Arg::None, 4, 4),
        Instruction::new(0x44, "ld B, H", Arg::None, 4, 4),
        Instruction::new(0x45, "ld B, L", Arg::None, 4, 4),
        Instruction::new(0x46, "ld B, (HL)", Arg::None, 8, 8),
        Instruction::new(0x47, "ld B, A", Arg::None, 4, 4),

        Instruction::new(0x48, "ld C, B", Arg::None, 4, 4),
        Instruction::new(0x49, "ld C, C", Arg::None, 4, 4),
        Instruction::new(0x4A, "ld C, D", Arg::None, 4, 4),
        Instruction::new(0x4B, "ld C, E", Arg::None, 4, 4),
        Instruction::new(0x4C, "ld C, H", Arg::None, 4, 4),
        Instruction::new(0x4D, "ld C, L", Arg::None, 4, 4),
        Instruction::new(0x4E, "ld C, (HL)", Arg::None, 8, 8),
        Instruction::new(0x4F, "ld C, A", Arg::None, 4, 4),

        Instruction::new(0x50, "ld D, B", Arg::None, 4, 4),
        Instruction::new(0x51, "ld D, C", Arg::None, 4, 4),
        Instruction::new(0x52, "ld D, D", Arg::None, 4, 4),
        Instruction::new(0x53, "ld D, E", Arg::None, 4, 4),
        Instruction::new(0x54, "ld D, H", Arg::None, 4, 4),
        Instruction::new(0x55, "ld D, L", Arg::None, 4, 4),
        Instruction::new(0x56, "ld D, (HL)", Arg::None, 8, 8),
        Instruction::new(0x57, "ld D, A", Arg::None, 4, 4),

        Instruction::new(0x58, "ld E, B", Arg::None, 4, 4),
        Instruction::new(0x59, "ld E, C", Arg::None, 4, 4),
        Instruction::new(0x5A, "ld E, D", Arg::None, 4, 4),
        Instruction::new(0x5B, "ld E, E", Arg::None, 4, 4),
        Instruction::new(0x5C, "ld E, H", Arg::None, 4, 4),
        Instruction::new(0x5D, "ld E, L", Arg::None, 4, 4),
        Instruction::new(0x5E, "ld E, (HL)", Arg::None, 8, 8),
        Instruction::new(0x5F, "ld E, A", Arg::None, 4, 4),

        Instruction::new(0x60, "ld H, B", Arg::None, 4, 4),
        Instruction::new(0x61, "ld H, C", Arg::None, 4, 4),
        Instruction::new(0x62, "ld H, D", Arg::None, 4, 4),
        Instruction::new(0x63, "ld H, E", Arg::None, 4, 4),
        Instruction::new(0x64, "ld H, H", Arg::None, 4, 4),
        Instruction::new(0x65, "ld H, L", Arg::None, 4, 4),
        Instruction::new(0x66, "ld H, (HL)", Arg::None, 8, 8),
        Instruction::new(0x67, "ld H, A", Arg::None, 4, 4),

        Instruction::new(0x68, "ld L, B", Arg::None, 4, 4),
        Instruction::new(0x69, "ld L, C", Arg::None, 4, 4),
        Instruction::new(0x6A, "ld L, D", Arg::None, 4, 4),
        Instruction::new(0x6B, "ld L, E", Arg::None, 4, 4),
        Instruction::new(0x6C, "ld L, H", Arg::None, 4, 4),
        Instruction::new(0x6D, "ld L, L", Arg::None, 4, 4),
        Instruction::new(0x6E, "ld L, (HL)", Arg::None, 8, 8),
        Instruction::new(0x6F, "ld L, A", Arg::None, 4, 4),

        Instruction::new(0x70, "ld (HL), B", Arg::None, 8, 8),
        Instruction::new(0x71, "ld (HL), C", Arg::None, 8, 8),
        Instruction::new(0x72, "ld (HL), D", Arg::None, 8, 8),
        Instruction::new(0x73, "ld (HL), E", Arg::None, 8, 8),
        Instruction::new(0x74, "ld (HL), H", Arg::None, 8, 8),
        Instruction::new(0x75, "ld (HL), L", Arg::None, 8, 8),
        Instruction::new(0x76, "halt", Arg::None, 4, 4),
        Instruction::new(0x77, "ld (HL), A", Arg::None, 8, 8),

        Instruction::new(0x78, "ld A, B", Arg::None, 4, 4),
        Instruction::new(0x79, "ld A, C", Arg::None, 4, 4),
        Instruction::new(0x7A, "ld A, D", Arg::None, 4, 4),
        Instruction::new(0x7B, "ld A, E", Arg::None, 4, 4),
        Instruction::new(0x7C, "ld A, H", Arg::None, 4, 4),
        Instruction::new(0x7D, "ld A, L", Arg::None, 4, 4),
        Instruction::new(0x7E, "ld A, (HL)", Arg::None, 8, 8),
        Instruction::new(0x7F, "ld A, A", Arg::None, 4, 4),

        Instruction::new(0x80, "add A, B", Arg::None, 4, 4),
        Instruction::new(0x81, "add A, C", Arg::None, 4, 4),
        Instruction::new(0x82, "add A, D", Arg::None, 4, 4),
        Instruction::new(0x83, "add A, E", Arg::None, 4, 4),
        Instruction::new(0x84, "add A, H", Arg::None, 4, 4),
        Instruction::new(0x85, "add A, L", Arg::None, 4, 4),
        Instruction::new(0x86, "add A, (HL)", Arg::None, 8, 8),
        Instruction::new(0x87, "add A, A", Arg::None, 4, 4),

        Instruction::new(0x88, "adc A, B", Arg::None, 4, 4),
        Instruction::new(0x89, "adc A, C", Arg::None, 4, 4),
        Instruction::new(0x8A, "adc A, D", Arg::None, 4, 4),
        Instruction::new(0x8B, "adc A, E", Arg::None, 4, 4),
        Instruction::new(0x8C, "adc A, H", Arg::None, 4, 4),
        Instruction::new(0x8D, "adc A, L", Arg::None, 4, 4),
        Instruction::new(0x8E, "adc A, (HL)", Arg::None, 8, 8),
        Instruction::new(0x8F, "adc A, A", Arg::None, 4, 4),

        Instruction::new(0x90, "sub A, B", Arg::None, 4, 4),
        Instruction::new(0x91, "sub A, C", Arg::None, 4, 4),
        Instruction::new(0x92, "sub A, D", Arg::None, 4, 4),
        Instruction::new(0x93, "sub A, E", Arg::None, 4, 4),
        Instruction::new(0x94, "sub A, H", Arg::None, 4, 4),
        Instruction::new(0x95, "sub A, L", Arg::None, 4, 4),
        Instruction::new(0x96, "sub A, (HL)", Arg::None, 8, 8),
        Instruction::new(0x97, "sub A, A", Arg::None, 4, 4),

        Instruction::new(0x98, "sbc A, B", Arg::None, 4, 4),
        Instruction::new(0x99, "sbc A, C", Arg::None, 4, 4),
        Instruction::new(0x9A, "sbc A, D", Arg::None, 4, 4),
        Instruction::new(0x9B, "sbc A, E", Arg::None, 4, 4),
        Instruction::new(0x9C, "sbc A, H", Arg::None, 4, 4),
        Instruction::new(0x9D, "sbc A, L", Arg::None, 4, 4),
        Instruction::new(0x9E, "sbc A, (HL)", Arg::None, 8, 8),
        Instruction::new(0x9F, "sbc A, A", Arg::None, 4, 4),

        Instruction::new(0xA0, "and A, B", Arg::None, 4, 4),
        Instruction::new(0xA1, "and A, C", Arg::None, 4, 4),
        Instruction::new(0xA2, "and A, D", Arg::None, 4, 4),
        Instruction::new(0xA3, "and A, E", Arg::None, 4, 4),
        Instruction::new(0xA4, "and A, H", Arg::None, 4, 4),
        Instruction::new(0xA5, "and A, L", Arg::None, 4, 4),
        Instruction::new(0xA6, "and A, (HL)", Arg::None, 8, 8),
        Instruction::new(0xA7, "and A, A", Arg::None, 4, 4),

        Instruction::new(0xA8, "xor A, B", Arg::None, 4, 4),
        Instruction::new(0xA9, "xor A, C", Arg::None, 4, 4),
        Instruction::new(0xAA, "xor A, D", Arg::None, 4, 4),
        Instruction::new(0xAB, "xor A, E", Arg::None, 4, 4),
        Instruction::new(0xAC, "xor A, H", Arg::None, 4, 4),
        Instruction::new(0xAD, "xor A, L", Arg::None, 4, 4),
        Instruction::new(0xAE, "xor A, (HL)", Arg::None, 8, 8),
        Instruction::new(0xAF, "xor A, A", Arg::None, 4, 4),

        Instruction::new(0xB0, "or A, B", Arg::None, 4, 4),
        Instruction::new(0xB1, "or A, C", Arg::None, 4, 4),
        Instruction::new(0xB2, "or A, D", Arg::None, 4, 4),
        Instruction::new(0xB3, "or A, E", Arg::None, 4, 4),
        Instruction::new(0xB4, "or A, H", Arg::None, 4, 4),
        Instruction::new(0xB5, "or A, L", Arg::None, 4, 4),
        Instruction::new(0xB6, "or A, (HL)", Arg::None, 8, 8),
        Instruction::new(0xB7, "or A, A", Arg::None, 4, 4),

        Instruction::new(0xB8, "cp A, B", Arg::None, 4, 4),
        Instruction::new(0xB9, "cp A, C", Arg::None, 4, 4),
        Instruction::new(0xBA, "cp A, D", Arg::None, 4, 4),
        Instruction::new(0xBB, "cp A, E", Arg::None, 4, 4),
        Instruction::new(0xBC, "cp A, H", Arg::None, 4, 4),
        Instruction::new(0xBD, "cp A, L", Arg::None, 4, 4),
        Instruction::new(0xBE, "cp A, (HL)", Arg::None, 8, 8),
        Instruction::new(0xBF, "cp A, A", Arg::None, 4, 4),

        Instruction::new(0xC0, "ret nz", Arg::None, 8, 20),
        Instruction::new(0xC1, "pop BC", Arg::None, 12, 12),
        Instruction::new(0xC2, "jp nz, <a16>", Arg::a16(), 12, 16),
        Instruction::new(0xC3, "jp <a16>", Arg::a16(), 16, 16),
        Instruction::new(0xC4, "call nz, <a16>", Arg::a16(), 12, 24),
        Instruction::new(0xC5, "push BC", Arg::None, 16, 16),
        Instruction::new(0xC6, "add A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xC7, "rst $00", Arg::None, 16, 16),

        Instruction::new(0xC8, "ret z", Arg::None, 8, 20),
        Instruction::new(0xC9, "ret", Arg::None, 16, 16),
        Instruction::new(0xCA, "jp z, <a16>", Arg::a16(), 12, 16),
        Instruction::new(0xCB, "prefix $CB",  Arg::None, 4, 4),
        Instruction::new(0xCC, "call z, <a16>", Arg::a16(), 12, 24),
        Instruction::new(0xCD, "call <a16>", Arg::a16(), 24, 24),
        Instruction::new(0xCE, "adc A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xCF, "rst $08", Arg::None, 16, 16),

        Instruction::new(0xD0, "ret nc", Arg::None, 8, 20),
        Instruction::new(0xD1, "pop DE", Arg::None, 12, 12),
        Instruction::new(0xD2, "jp nc, <a16>", Arg::a16(), 12, 16),
        Instruction::none(0xD3),
        Instruction::new(0xD4, "call nc, <a16>", Arg::a16(), 12, 24),
        Instruction::new(0xD5, "push DE", Arg::None, 16, 16),
        Instruction::new(0xD6, "sub A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xD7, "rst $10", Arg::None, 16, 16),

        Instruction::new(0xD8, "ret c", Arg::None, 8, 20),
        Instruction::new(0xD9, "reti", Arg::None, 16, 16),
        Instruction::new(0xDA, "jp c, <a16>", Arg::a16(), 12, 16),
        Instruction::none(0xDB),
        Instruction::new(0xDC, "call c, <a16>", Arg::a16(), 12, 24),
        Instruction::none(0xDD),
        Instruction::new(0xDE, "sbc A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xDF, "rst $18", Arg::None, 16, 16),

        Instruction::new(0xE0, "ldh (<a8>), A", Arg::a8(), 12, 12),
        Instruction::new(0xE1, "pop HL", Arg::None, 12, 12),
        Instruction::new(0xE2, "ld (C), A", Arg::None, 8, 8),
        Instruction::none(0xE3),
        Instruction::none(0xE4),
        Instruction::new(0xE5, "push HL", Arg::None, 16, 16),
        Instruction::new(0xE6, "and A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xE7, "rst $20", Arg::None, 16, 16),

        Instruction::new(0xE8, "add SP, <r8>", Arg::r8(), 16, 16),
        Instruction::new(0xE9, "jp (HL)", Arg::None, 4, 4),
        Instruction::new(0xEA, "ld (<a16>), A", Arg::a16(), 16, 16),
        Instruction::none(0xEB),
        Instruction::none(0xEC),
        Instruction::none(0xED),
        Instruction::new(0xEE, "xor A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xEF, "rst $28", Arg::None, 16, 16),

        Instruction::new(0xF0, "ldh A, (<a8>)", Arg::a8(), 12, 12),
        Instruction::new(0xF1, "pop AF", Arg::None, 12, 12),
        Instruction::new(0xF2, "ld A, (C)", Arg::None, 8, 8),
        Instruction::new(0xF3, "di", Arg::None, 4, 4),
        Instruction::none(0xF4),
        Instruction::new(0xF5, "push AF", Arg::None, 16, 16),
        Instruction::new(0xF6, "or A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xF7, "rst $30", Arg::None, 16, 16),

        Instruction::new(0xF8, "ld HL, SP + <r8>", Arg::r8(), 12, 12),
        Instruction::new(0xF9, "ld SP, HL", Arg::None, 8, 8),
        Instruction::new(0xFA, "ld A, (<a16>)", Arg::a16(), 16, 16),
        Instruction::new(0xFB, "ei", Arg::None, 4, 4),
        Instruction::none(0xFC),
        Instruction::none(0xFD),
        Instruction::new(0xFE, "cp A, <d8>", Arg::d8(), 8, 8),
        Instruction::new(0xFF, "rst $38", Arg::None, 16, 16),
    ];
}

