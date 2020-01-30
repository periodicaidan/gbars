use self::super::registers::Registers;
use self::super::opcodes::*;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Formatter;
use core::fmt;
use std::ops::BitOr;
use std::io::prelude::*;
use std::io::{BufReader};

#[derive(Debug)]
pub enum GameBoyType {
    None,
    Classic,
    Color,
    Advance
}

pub struct Emulator {
    pub cpu: Registers,                     // The CPU registers
    pub stack: Vec<u16>,                    // The Stack™
    pub memory: [u8; 0x200000],             // The memory bank
    pub screen_buffer: [u8; 160 * 144],     // An internal representation of the screen
    pub screen_scale: f64,                  // The size of the screen when it gets drawn
    pub gbtype: GameBoyType,                // The type of GameBoy being emulated
    pub gbs_compatible: bool,               // Whether or not the ROM is compatible w/ Super GameBoy features
    pub title: String,                      // The game's title
    pub rom: Option<ROM>,                   // The currently running game (or no game)
    opcodes: [Option<Instruction>; 0x100]
}

impl Emulator {
    pub fn start(rom: Option<&str>) -> Result<Emulator, &'static str> {
        println!("Initializing GBARS...");
        print!("Loading instruction set...");
        let opcodes = [
            Some(Instruction::new(0x00, "nop", "No operation", 1, 4)),
            Some(Instruction::new(0x01, "ld BC, <u16>", "Load a short into BC", 3, 12)),
            Some(Instruction::new(0x02, "ld (BC), A", "Load the value in A into the address stored in BC", 1, 8)),
            Some(Instruction::new(0x03, "inc BC", "Increment BC", 1, 8)),
            Some(Instruction::new(0x04, "inc B", "Increment B", 1, 4)),
            Some(Instruction::new(0x05, "dec B", "Decrement B", 1, 4)),
            Some(Instruction::new(0x06, "ld B, <u8>", "Load a byte into B", 2, 8)),
            Some(Instruction::new(0x07, "rlca", "Rotate the value in A to the left, storing the leftmost bit in Carry", 1, 4)),

            Some(Instruction::new(0x08, "ld (<u16>), SP", "Load the value of the stack pointer into memory", 3, 20)),
            Some(Instruction::new(0x09, "add HL, BC", "Add the value in BC to the value in HL, storing the result in HL", 1, 8)),
            Some(Instruction::new(0x0A, "ld A, (BC)", "Load the value in the memory location stored in BC into A", 1, 8)),
            Some(Instruction::new(0x0B, "dec BC", "Decrement BC", 1, 8)),
            Some(Instruction::new(0x0C, "inc C", "Increment C", 1, 4)),
            Some(Instruction::new(0x0D, "dec C", "Decrement C", 1, 4)),
            Some(Instruction::new(0x0E, "ld C, <u8>", "Load a byte into C", 2, 8)),
            Some(Instruction::new(0x0F, "rrca", "Rotate the value in A to the right, storing the rightmost bit in Carry", 1, 4)),

            Some(Instruction::new(0x10, "stop $00", "Halt CPU and screen until a button is pressed", 2, 4)),
            Some(Instruction::new(0x11, "ld DE, <u16>", "Load a short into DE", 3, 12)),
            Some(Instruction::new(0x12, "ld (DE), A", "Load the value in A into the address stored in BC", 1, 8)),
            Some(Instruction::new(0x13, "inc DE", "Increment DE", 1, 8)),
            Some(Instruction::new(0x14, "inc D", "Increment D", 1, 4)),
            Some(Instruction::new(0x15, "dec D", "Decrement D", 1, 4)),
            Some(Instruction::new(0x16, "ld D, <u8>", "Load a byte into D", 2, 8)),
            Some(Instruction::new(0x17, "rla", "Rotate the value in A to the left", 1, 4)),

            Some(Instruction::new(0x18, "jr <i8>", "Jump relative to the current position", 2, 12)),
            Some(Instruction::new(0x19, "add HL, DE", "Add the value in DE to the value in HL, storing the result in HL", 1, 8)),
            Some(Instruction::new(0x1A, "ld A, (DE)", "Load the value in the memory location stored in DE into A", 1, 8)),
            Some(Instruction::new(0x1B, "dec DE", "Decrement DE", 1, 8)),
            Some(Instruction::new(0x1C, "inc E", "Increment E", 1, 4)),
            Some(Instruction::new(0x1D, "dec E", "Decrement E", 1, 4)),
            Some(Instruction::new(0x1E, "ld E, <u8>", "Load a byte into E", 2, 8)),
            Some(Instruction::new(0x1F, "rra", "Rotate the value in A to the right", 1, 4)),

            Some(Instruction::new(0x20, "jr NZ, <i8>", "Jump relative to current position if Zero is not set", 2, 8)),
            Some(Instruction::new(0x21, "ld HL, <u16>", "Load a short into HL", 3, 12)),
            Some(Instruction::new(0x22, "ld (HL+), A", "Load the value in A into the address stored in HL and increment HL", 1, 8)),
            Some(Instruction::new(0x23, "inc HL", "Increment HL", 1, 8)),
            Some(Instruction::new(0x24, "inc H", "Increment H", 1, 4)),
            Some(Instruction::new(0x25, "dec H", "Decrement H", 1, 4)),
            Some(Instruction::new(0x26, "ld H, <u8>", "Load a byte into H", 2, 8)),
            Some(Instruction::new(0x27, "daa", "Convert the value in A to a binary-encoded decimal", 1, 4)),

            Some(Instruction::new(0x28, "jr Z, <i8>", "Jump relative to current position if Zero is set", 2, 8)),
            Some(Instruction::new(0x29, "add HL, HL", "Add the value stored in HL to the value in HL and store the result in HL", 1, 8)),
            Some(Instruction::new(0x2A, "ld A, (HL+)", "Load the value at the address stored in HL to A and increment HL", 1, 8)),
            Some(Instruction::new(0x2B, "dec HL", "Decrement HL", 1, 8)),
            Some(Instruction::new(0x2C, "inc L", "Increment L", 1, 4)),
            Some(Instruction::new(0x2D, "dec L", "Decrement L", 1, 4)),
            Some(Instruction::new(0x2E, "ld L, <u8>", "Load a byte into L", 2, 8)),
            Some(Instruction::new(0x2F, "cpl", "Flip all the bits of A", 1, 4)),

            Some(Instruction::new(0x30, "jr NC, <i8>", "Jump relative to current position if Carry is not set", 2, 8)),
            Some(Instruction::new(0x31, "ld SP, <u16>", "Load a short into the stack pointer", 3, 12)),
            Some(Instruction::new(0x32, "ld (HL-), A", "Load the value in A into the address stored in HL and decrement HL", 1, 8)),
            Some(Instruction::new(0x33, "inc SP", "Increment the stack pointer", 1, 8)),
            Some(Instruction::new(0x34, "inc (HL)", "Increment the value at the address stored in HL", 1, 12)),
            Some(Instruction::new(0x35, "dec (HL)", "Decrement the value at the address stored in HL", 1, 12)),
            Some(Instruction::new(0x36, "ld (HL), <u8>", "Load a byte into the address stored in HL", 2, 8)),
            Some(Instruction::new(0x37, "scf", "Set the Carry flag", 1, 4)),

            Some(Instruction::new(0x38, "jr C, <i8>", "Jump relative to current position if Carry is set", 2, 8)),
            Some(Instruction::new(0x39, "add HL, SP", "Add the value of the stack pointer to the value in HL and store the result in HL", 1, 8)),
            Some(Instruction::new(0x3A, "ld A, (HL-)", "Load the value at the address stored in HL to A and decrement HL", 1, 8)),
            Some(Instruction::new(0x3B, "dec SP", "Decrement the stack pointer", 1, 8)),
            Some(Instruction::new(0x3C, "inc A", "Increment A", 1, 4)),
            Some(Instruction::new(0x3D, "dec A", "Decrement A", 1, 4)),
            Some(Instruction::new(0x3E, "ld A, <u8>", "Load a byte into A", 2, 8)),
            Some(Instruction::new(0x3F, "ccf", "Flip the Carry flag", 1, 4)),

            Some(Instruction::new(0x40, "ld B, B", "Load the value in B into B", 1, 4)),
            Some(Instruction::new(0x41, "ld B, C", "Load the value in C into B", 1, 4)),
            Some(Instruction::new(0x42, "ld B, D", "Load the value in D into B", 1, 4)),
            Some(Instruction::new(0x43, "ld B, E", "Load the value in E into B", 1, 4)),
            Some(Instruction::new(0x44, "ld B, H", "Load the value in H into B", 1, 4)),
            Some(Instruction::new(0x45, "ld B, H", "Load the value in L into B", 1, 4)),
            Some(Instruction::new(0x46, "ld B, (HL)", "Load the value at the address stored in HL into B", 1, 8)),
            Some(Instruction::new(0x47, "ld B, A", "Load the value in A into B", 1, 4)),

            Some(Instruction::new(0x48, "ld C, B", "Load the value in B into C", 1, 4)),
            Some(Instruction::new(0x49, "ld C, C", "Load the value in C into C", 1, 4)),
            Some(Instruction::new(0x4A, "ld C, D", "Load the value in D into C", 1, 4)),
            Some(Instruction::new(0x4B, "ld C, E", "Load the value in E into C", 1, 4)),
            Some(Instruction::new(0x4C, "ld C, H", "Load the value in H into C", 1, 4)),
            Some(Instruction::new(0x4D, "ld C, H", "Load the value in L into C", 1, 4)),
            Some(Instruction::new(0x4E, "ld C, (HL)", "Load the value at the address stored in HL into C", 1, 8)),
            Some(Instruction::new(0x4F, "ld C, A", "Load the value in A into C", 1, 4)),

            Some(Instruction::new(0x50, "ld D, B", "Load the value in B into D", 1, 4)),
            Some(Instruction::new(0x51, "ld D, C", "Load the value in C into D", 1, 4)),
            Some(Instruction::new(0x52, "ld D, D", "Load the value in D into D", 1, 4)),
            Some(Instruction::new(0x53, "ld D, E", "Load the value in E into D", 1, 4)),
            Some(Instruction::new(0x54, "ld D, H", "Load the value in H into D", 1, 4)),
            Some(Instruction::new(0x55, "ld D, H", "Load the value in L into D", 1, 4)),
            Some(Instruction::new(0x56, "ld D, (HL)", "Load the value at the address stored in HL into D", 1, 8)),
            Some(Instruction::new(0x57, "ld D, A", "Load the value in A into D", 1, 4)),

            Some(Instruction::new(0x58, "ld E, B", "Load the value in B into E", 1, 4)),
            Some(Instruction::new(0x59, "ld E, C", "Load the value in C into E", 1, 4)),
            Some(Instruction::new(0x5A, "ld E, D", "Load the value in D into E", 1, 4)),
            Some(Instruction::new(0x5B, "ld E, E", "Load the value in E into E", 1, 4)),
            Some(Instruction::new(0x5C, "ld E, H", "Load the value in H into E", 1, 4)),
            Some(Instruction::new(0x5D, "ld E, H", "Load the value in L into E", 1, 4)),
            Some(Instruction::new(0x5E, "ld E, (HL)", "Load the value at the address stored in HL into E", 1, 8)),
            Some(Instruction::new(0x5F, "ld E, A", "Load the value in A into E", 1, 4)),

            Some(Instruction::new(0x60, "ld H, B", "Load the value in B into H", 1, 4)),
            Some(Instruction::new(0x61, "ld H, C", "Load the value in C into H", 1, 4)),
            Some(Instruction::new(0x62, "ld H, D", "Load the value in D into H", 1, 4)),
            Some(Instruction::new(0x63, "ld H, E", "Load the value in E into H", 1, 4)),
            Some(Instruction::new(0x64, "ld H, H", "Load the value in H into H", 1, 4)),
            Some(Instruction::new(0x65, "ld H, H", "Load the value in L into H", 1, 4)),
            Some(Instruction::new(0x66, "ld H, (HL)", "Load the value at the address stored in HL into H", 1, 8)),
            Some(Instruction::new(0x67, "ld H, A", "Load the value in A into H", 1, 4)),

            Some(Instruction::new(0x68, "ld L, B", "Load the value in B into L", 1, 4)),
            Some(Instruction::new(0x69, "ld L, C", "Load the value in C into L", 1, 4)),
            Some(Instruction::new(0x6A, "ld L, D", "Load the value in D into L", 1, 4)),
            Some(Instruction::new(0x6B, "ld L, E", "Load the value in E into L", 1, 4)),
            Some(Instruction::new(0x6C, "ld L, H", "Load the value in H into L", 1, 4)),
            Some(Instruction::new(0x6D, "ld L, H", "Load the value in L into L", 1, 4)),
            Some(Instruction::new(0x6E, "ld L, (HL)", "Load the value at the address stored in HL into L", 1, 8)),
            Some(Instruction::new(0x6F, "ld L, A", "Load the value in A into L", 1, 4)),

            Some(Instruction::new(0x70, "ld (HL), B", "Load the value in B into the address stored in HL", 1, 8)),
            Some(Instruction::new(0x71, "ld (HL), C", "Load the value in C into the address stored in HL", 1, 8)),
            Some(Instruction::new(0x72, "ld (HL), D", "Load the value in D into the address stored in HL", 1, 8)),
            Some(Instruction::new(0x73, "ld (HL), E", "Load the value in E into the address stored in HL", 1, 8)),
            Some(Instruction::new(0x74, "ld (HL), H", "Load the value in H into the address stored in HL", 1, 8)),
            Some(Instruction::new(0x75, "ld (HL), H", "Load the value in L into the address stored in HL", 1, 8)),
            Some(Instruction::new(0x76, "halt", "Power down CPU until an interrupt occurs", 1, 4)),
            Some(Instruction::new(0x77, "ld (HL), A", "Load the value in A into the address stored in HL", 1, 8)),

            Some(Instruction::new(0x78, "ld A, B", "Load the value in B into A", 1, 4)),
            Some(Instruction::new(0x79, "ld A, C", "Load the value in C into A", 1, 4)),
            Some(Instruction::new(0x7A, "ld A, D", "Load the value in D into A", 1, 4)),
            Some(Instruction::new(0x7B, "ld A, E", "Load the value in E into A", 1, 4)),
            Some(Instruction::new(0x7C, "ld A, H", "Load the value in H into A", 1, 4)),
            Some(Instruction::new(0x7D, "ld A, H", "Load the value in L into A", 1, 4)),
            Some(Instruction::new(0x7E, "ld A, (HL)", "Load the value at the address stored in HL into A", 1, 8)),
            Some(Instruction::new(0x7F, "ld A, A", "Load the value in A into A", 1, 4)),

            Some(Instruction::new(0x80, "add A, B", "Add the value in B to A", 1, 4)),
            Some(Instruction::new(0x81, "add A, C", "Add the value in C to A", 1, 4)),
            Some(Instruction::new(0x82, "add A, D", "Add the value in D to A", 1, 4)),
            Some(Instruction::new(0x83, "add A, E", "Add the value in E to A", 1, 4)),
            Some(Instruction::new(0x84, "add A, H", "Add the value in H to A", 1, 4)),
            Some(Instruction::new(0x85, "add A, L", "Add the value in L to A", 1, 4)),
            Some(Instruction::new(0x86, "add A, (HL)", "Add the value at the address stored in HL to A", 1, 8)),
            Some(Instruction::new(0x87, "add A, A", "Add the value in A to A", 1, 4)),

            Some(Instruction::new(0x88, "adc A, B", "Add the value in B plus Carry to A", 1, 4)),
            Some(Instruction::new(0x89, "adc A, C", "Add the value in C plus Carry to A", 1, 4)),
            Some(Instruction::new(0x8A, "adc A, D", "Add the value in D plus Carry to A", 1, 4)),
            Some(Instruction::new(0x8B, "adc A, E", "Add the value in E plus Carry to A", 1, 4)),
            Some(Instruction::new(0x8C, "adc A, H", "Add the value in H plus Carry to A", 1, 4)),
            Some(Instruction::new(0x8D, "adc A, L", "Add the value in L plus Carry to A", 1, 4)),
            Some(Instruction::new(0x8E, "adc A, (HL)", "Add the value at the address stored in HL plus Carry to A", 1, 8)),
            Some(Instruction::new(0x8F, "adc A, A", "Add the value in A plus Carry to A", 1, 4)),

            Some(Instruction::new(0x90, "sub B", "Subtract the value in B from A", 1, 4)),
            Some(Instruction::new(0x91, "sub C", "Subtract the value in C from A", 1, 4)),
            Some(Instruction::new(0x92, "sub D", "Subtract the value in D from A", 1, 4)),
            Some(Instruction::new(0x93, "sub E", "Subtract the value in E from A", 1, 4)),
            Some(Instruction::new(0x94, "sub H", "Subtract the value in H from A", 1, 4)),
            Some(Instruction::new(0x95, "sub L", "Subtract the value in L from A", 1, 4)),
            Some(Instruction::new(0x96, "sub (HL)", "Subtract the value at the address stored in HL from A", 1, 8)),
            Some(Instruction::new(0x97, "sub A, A", "Subtract the value in A from A", 1, 4)),

            Some(Instruction::new(0x98, "sbc A, B", "Subtract the value in B plus Carry from A", 1, 4)),
            Some(Instruction::new(0x99, "sbc A, C", "Subtract the value in C plus Carry from A", 1, 4)),
            Some(Instruction::new(0x9A, "sbc A, D", "Subtract the value in D plus Carry from A", 1, 4)),
            Some(Instruction::new(0x9B, "sbc A, E", "Subtract the value in E plus Carry from A", 1, 4)),
            Some(Instruction::new(0x9C, "sbc A, H", "Subtract the value in H plus Carry from A", 1, 4)),
            Some(Instruction::new(0x9D, "sbc A, L", "Subtract the value in L plus Carry from A", 1, 4)),
            Some(Instruction::new(0x9E, "sbc A, (HL)", "Subtract the value at the address stored in HL plus Carry from A", 1, 8)),
            Some(Instruction::new(0x9F, "sbc A, A", "Subtract the value in A plus Carry from A", 1, 4)),

            Some(Instruction::new(0xA0, "and B", "Bitwise and the value in B with A", 1, 4)),
            Some(Instruction::new(0xA1, "and C", "Bitwise and the value in C from A", 1, 4)),
            Some(Instruction::new(0xA2, "and D", "Bitwise and the value in D from A", 1, 4)),
            Some(Instruction::new(0xA3, "and E", "Bitwise and the value in E from A", 1, 4)),
            Some(Instruction::new(0xA4, "and H", "Bitwise and the value in H from A", 1, 4)),
            Some(Instruction::new(0xA5, "and L", "Bitwise and the value in L from A", 1, 4)),
            Some(Instruction::new(0xA6, "and (HL)", "Bitwise and the value at the address stored in HL with A", 1, 8)),
            Some(Instruction::new(0xA7, "and A", "Bitwise and the value in A with A", 1, 4)),

            Some(Instruction::new(0xA8, "xor B", "Bitwise xor the value in B with A", 1, 4)),
            Some(Instruction::new(0xA9, "xor C", "Bitwise xor the value in C with A", 1, 4)),
            Some(Instruction::new(0xAA, "xor D", "Bitwise xor the value in D with A", 1, 4)),
            Some(Instruction::new(0xAB, "xor E", "Bitwise xor the value in E with A", 1, 4)),
            Some(Instruction::new(0xAC, "xor H", "Bitwise xor the value in H with A", 1, 4)),
            Some(Instruction::new(0xAD, "xor L", "Bitwise xor the value in L with A", 1, 4)),
            Some(Instruction::new(0xAE, "xor (HL)", "Bitwise xor the value at the address stored in HL with A", 1, 8)),
            Some(Instruction::new(0xAF, "xor A", "Bitwise xor the value in A with A", 1, 4)),

            Some(Instruction::new(0xB0, "or B", "Bitwise or the value in B with A", 1, 4)),
            Some(Instruction::new(0xB1, "or C", "Bitwise or the value in C from A", 1, 4)),
            Some(Instruction::new(0xB2, "or D", "Bitwise or the value in D from A", 1, 4)),
            Some(Instruction::new(0xB3, "or E", "Bitwise or the value in E from A", 1, 4)),
            Some(Instruction::new(0xB4, "or H", "Bitwise or the value in H from A", 1, 4)),
            Some(Instruction::new(0xB5, "or L", "Bitwise or the value in L from A", 1, 4)),
            Some(Instruction::new(0xB6, "or (HL)", "Bitwise or the value at the address stored in HL with A", 1, 8)),
            Some(Instruction::new(0xB7, "or A", "Bitwise or the value in A with A", 1, 4)),

            Some(Instruction::new(0xB8, "cp B", "Compare the value in B to that in A", 1, 4)),
            Some(Instruction::new(0xB9, "cp C", "Compare the value in C to that in A", 1, 4)),
            Some(Instruction::new(0xBA, "cp D", "Compare the value in D to that in A", 1, 4)),
            Some(Instruction::new(0xBB, "cp E", "Compare the value in E to that in A", 1, 4)),
            Some(Instruction::new(0xBC, "cp H", "Compare the value in H to that in A", 1, 4)),
            Some(Instruction::new(0xBD, "cp L", "Compare the value in L to that in A", 1, 4)),
            Some(Instruction::new(0xBE, "cp (HL)", "Compare the value at the address stored in HL to that A", 1, 8)),
            Some(Instruction::new(0xBF, "cp A", "Compare the value in A to that in A", 1, 4)),

            Some(Instruction::new(0xC0, "ret NZ", "Return from a function if Zero is not set", 1, 8)),
            Some(Instruction::new(0xC1, "pop BC", "Pop a value off the stack and store it in BC", 1, 12)),
            Some(Instruction::new(0xC2, "jp NZ, <u16>", "Jump somewhere in memory if Zero is not set", 3, 12)),
            Some(Instruction::new(0xC3, "jp <u16>", "Jump somewhere in memory", 3, 16)),
            Some(Instruction::new(0xC4, "call NZ, <u16>", "Call a function beginning at an address if Zero is not set", 3, 24)),
            Some(Instruction::new(0xC5, "push BC", "Push the value in BC onto the stack", 1, 16)),
            Some(Instruction::new(0xC6, "add A, <u8>", "Add a byte to A", 2, 8)),
            Some(Instruction::new(0xC7, "rst $00", "Push present address onto stack and jump to address $0000", 1, 16)),

            Some(Instruction::new(0xC8, "ret Z", "Return from a function if Zero is set", 1, 16)),
            Some(Instruction::new(0xC9, "ret", "Return from a function", 1, 16)),
            Some(Instruction::new(0xCA, "jp Z, <u16>", "Jump somewhere in memory if Zero is set", 3, 12)),
            Some(Instruction::new(0xCB, "prefix CB", "Prefix for bitwise operations", 1, 4)),
            Some(Instruction::new(0xCC, "call Z, <u16>", "Call a function beginning at an address if Zero is set", 3, 24)),
            Some(Instruction::new(0xCD, "call <u16>", "Call a function beginning at some address", 3, 24)),
            Some(Instruction::new(0xCE, "adc A, <u8>", "Add a byte plus Carry to A", 2, 8)),
            Some(Instruction::new(0xCF, "rst $08", "Push present address onto stack and jump to address $0008", 1, 16)),

            Some(Instruction::new(0xD0, "ret NC", "Return from a function if Carry is not set", 1, 8)),
            Some(Instruction::new(0xD1, "pop DE", "Pop a value off the stack and store it in DE", 1, 12)),
            Some(Instruction::new(0xD2, "jp NC, <u16>", "Jump somewhere in memory if Carry is not set", 3, 12)),
            None, // 0xD3
            Some(Instruction::new(0xD4, "call NC, <u16>", "Call a function beginning at an address if Carry is not set", 3, 24)),
            Some(Instruction::new(0xD5, "push DE", "Push the value in DE onto the stack", 1, 16)),
            Some(Instruction::new(0xD6, "sub <u8>", "Subtract a byte from A", 2, 8)),
            Some(Instruction::new(0xD7, "rst $10", "Push present address onto stack and jump to address $0010", 1, 16)),

            Some(Instruction::new(0xD8, "ret C", "Return from a function if Carry is set", 1, 8)),
            Some(Instruction::new(0xD9, "reti", "Return from a function and enable interrupts", 1, 16)),
            Some(Instruction::new(0xDA, "jp C, <u16>", "Jump somewhere in memory if Carry is set", 3, 12)),
            None, // 0xDB
            Some(Instruction::new(0xDC, "call C, <u16>", "Call a function beginning at an address if Carry is set", 3, 24)),
            None, // 0xDD
            Some(Instruction::new(0xDE, "sbc <u8>", "Subtract a byte plus Carry from A", 2, 8)),
            Some(Instruction::new(0xDF, "rst $18", "Push present address onto stack and jump to address $0018", 1, 16)),

            Some(Instruction::new(0xE0, "ldh (<u8>), A", "Load the value in A into memory address $FF00 + a byte", 2, 12)),
            Some(Instruction::new(0xE1, "pop HL", "Pop a value off the stack and store it in HL", 1, 12)),
            Some(Instruction::new(0xE2, "ld (C), A", "Load the value in A into memory address $FF00 + C", 2, 8)),
            None, // 0xE3
            None, // 0xE4
            Some(Instruction::new(0xE5, "push HL", "Push the value in HL onto the stack", 1, 16)),
            Some(Instruction::new(0xE6, "and <u8>", "Bitwise and a byte with A", 2, 8)),
            Some(Instruction::new(0xE7, "rst $20", "Push present address onto stack and jump to address $0020", 1, 16)),

            Some(Instruction::new(0xE8, "add SP, <u8>", "Add a byte to the stack pointer", 2, 16)),
            Some(Instruction::new(0xE9, "jp (HL)", "Jump to the address stored in HL", 1, 4)),
            Some(Instruction::new(0xEA, "ld (<u16>), A", "Load A into a memory address", 3, 16)),
            None, // 0xEB
            None, // 0xEC
            None, // 0xED
            Some(Instruction::new(0xEE, "xor <u8>", "Bitwise xor a byte with A", 2, 8)),
            Some(Instruction::new(0xEF, "rst $28", "Push present address onto stack and jump to address $0028", 1, 16)),

            Some(Instruction::new(0xF0, "ldh A, (<u8>)", "Load the value at memory address $FF00 + a byte into A", 2, 12)),
            Some(Instruction::new(0xF1, "pop AF", "Pop a value off the stack and store it in AF", 1, 12)),
            Some(Instruction::new(0xF2, "ld A, (C)", "Load the value at memory address $FF00 + C into A", 2, 8)),
            Some(Instruction::new(0xF3, "di", "Disable interrupts starting after the next instruction", 1, 4)),
            None, // 0xF4
            Some(Instruction::new(0xF5, "push AF", "Push the value in AF onto the stack", 1, 16)),
            Some(Instruction::new(0xF6, "or <u8>", "Bitwise or a byte to A", 2, 8)),
            Some(Instruction::new(0xF7, "rst $30", "Push present address onto stack and jump to address $0030", 1, 16)),

            Some(Instruction::new(0xF8, "ld HL, SP+<u8>", "Add a byte to the value of the stack pointer, storing the result in HL", 2, 12)),
            Some(Instruction::new(0xF9, "ld SP, HL", "Load the value in HL into the stack pointer", 1, 8)),
            Some(Instruction::new(0xFA, "ld A, (<u16>)", "Load the value at some memory address into A", 3, 16)),
            Some(Instruction::new(0xFB, "ei", "Enable interrupts starting after the next instruction", 1, 4)),
            None, // 0xFC
            None, // 0xFD
            Some(Instruction::new(0xFE, "cp <u8>", "Compare a byte with A", 2, 8)),
            Some(Instruction::new(0xFF, "rst $38", "Push present address onto stack and jump to address $0038", 1, 16)),
        ];

        println!("Done.");

        let mut e = Emulator{
            cpu: Registers::init(),
            stack: Vec::with_capacity(32),
            memory: [0u8; 0x200000],
            screen_buffer: [0u8; 160 * 144],
            screen_scale: 4.0,
            gbtype: GameBoyType::None,
            gbs_compatible: false,
            title: String::with_capacity(14),
            rom: None,
            opcodes: opcodes
        };

        // Load ROM if present
        if let Some(r) = rom {
            println!("Loading ROM {}", r);
            e.load(r);

            // Parse header information
            let mut loc = 0x100usize;

            // Execute the first 4 bytes
            for _ in 0..4 {
                e.exec(e.memory[loc]);
                loc += 1;
            }

            let nintendo_graphic: [u8; 48] = [
                0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
                0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
                0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
                0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
                0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
                0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E
            ];

            // Check that the next 48 bytes are the Nintendo graphic and execute it
            for i in 0..48usize {
                if e.memory[loc] != nintendo_graphic[i] {
                    return Err("Invalid ROM");
                }

                e.exec(e.memory[loc]);

                loc += 1;
            }

            // Read the game title and set it
            for i in 0..15usize {
                let ch = e.memory[loc];
                if ch != 0 {
                    e.title.push(ch as char);
                }

                loc += 1;
            }

            // If this byte is $80 or $C0, then this is a GBC cart
            e.gbtype = match e.memory[loc] {
                0x80 | 0xC0 => GameBoyType::Color,
                _ => GameBoyType::Classic
            };

            loc += 1;

            // Skip the licensee flags for now
            loc += 2;

            // This byte will be set to $03 if the cartridge is compatible with Super GameBoy
            if e.memory[loc] == 0x03 {
                e.gbs_compatible = true;
            }

            loc += 1;

            // Skip cartridge type for now

            loc += 1;

            // Skip ROM size for now; may use it later

            loc += 1;

            // Ditto for the RAM size

            loc += 1;

            // Destination code. May just have it in there for shits and giggles



        }

        // TODO: Piston things...

        Ok(e)
    }

    pub fn load(&mut self, path_to_rom: &str) {
        println!("Loading file");

        let path = Path::new(path_to_rom);

        let mut file = match File::open(&path) {
            Err(why) => panic!("Could not open file {}: {}", path.display(), why.description()),
            Ok(file) => file
        };

        let mut data = file.bytes();
        let mut memloc = 0 as usize;
        for byte in data {
            if let Ok(b) = byte {
                self.memory[memloc] = b;
                memloc += 1;
            }
        }
    }

    pub fn exec(&mut self, code: u8) -> Option<u16> {
        let inst = &self.opcodes[code as usize];

        // To paraphrase an article I read somewhere:
        // "An emulator is just a giant switch statement"
        if let Some(i) = inst {
            match i.opcode {
                // NOP
                0x00 => {},

                // 8-bit increments and decrements
                0x04 => self.cpu.b += 1,
                0x05 => self.cpu.b -= 1,
                0x0C => self.cpu.c += 1,
                0x0D => self.cpu.c -= 1,
                0x14 => self.cpu.d += 1,
                0x15 => self.cpu.d -= 1,
                0x1C => self.cpu.e += 1,
                0x1D => self.cpu.e -= 1,
                0x24 => self.cpu.h += 1,
                0x25 => self.cpu.h -= 1,
                0x2C => self.cpu.l += 1,
                0x2D => self.cpu.l -= 1,
                0x34 => {
                    let addr = self.cpu.get_hl() as usize;
                    self.memory[addr] += 1;
                },
                0x35 => {
                    let addr = self.cpu.get_hl() as usize;
                    self.memory[addr] -= 1;
                },
                0x3C => self.cpu.a += 1,
                0x3D => self.cpu.a -= 1,

                // 16-bit increments and decrements
                0x03 => self.cpu.add_to_bc(1),
                0x0B => self.cpu.sub_from_bc(1),
                0x13 => self.cpu.add_to_de(1),
                0x1B => self.cpu.sub_from_de(1),
                0x23 => self.cpu.add_to_hl(1),
                0x2B => self.cpu.sub_from_hl(1),
                0x33 => self.cpu.sp += 1,
                0x3B => self.cpu.sp -= 1,

                // 8-bit arithmetic
                0x80 => self.cpu.a += self.cpu.b,

                _ => panic!("Unknown instruction {:02X}", i.opcode)
            }

            return Some(i.size as u16);
        }

        None
    }

    pub fn step(&mut self) {
        let counter = self.cpu.pc as usize;
        let opcode = self.memory[counter];
        let skip = self.exec(opcode).expect("Unknown Opcode");

        self.cpu.pc += skip;
    }
}

impl Debug for Emulator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Emulator ({}, {:?}, {:?}, {:?})", self.title, self.gbtype, self.cpu, self.stack)
    }
}

#[derive(Debug)]
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

pub struct ROM {
    pub path: String,
    pub size: usize,
    pub contents: Vec<u8>, // TODO: Experiment with a BufReader instead
    pub title: String,
    pub licensee: String,
    pub cart_type: Vec<CartFeature>,
    pub version_no: u8,
    pub gbs_compatible: bool,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl ROM {
    pub fn new(path: &str) -> ROM {
        let path = Path::new(path);

        let mut contents = Vec::new();

        let mut file = File::open(&path)
            .expect(&format!("Could not open file {}", path.display()));

        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut contents);

        // Get the title from the ROM in memory locations [0x134, 0x143)
        let mut title = String::new();
        for i in 0x134..0x143usize {
            if *contents.get(i).unwrap() == 0x00 { continue; }
            title.push(*contents.get(i).unwrap() as char);
        }

        let gbs_compatible = *contents.get(0x146).unwrap() == 0x03;

        // Work out the licensee from the code in memory locations 0x144 and 0x145 (or 0x14B for
        // older titles)
        let licensee: &str;

        if *contents.get(0x14B).unwrap() == 0x33u8 { // This means it's a newer title
            let mut licensee_code = String::new();
            licensee_code.push(*contents.get(0x144).unwrap() as char);
            licensee_code.push(*contents.get(0x145).unwrap() as char);

            licensee = match licensee_code.as_str() {
                "00" => "None",
                "01" => "Nintendo R&D1",
                "08" => "Capcom",
                "13" => "EA (Electronic Arts)",
                "18" => "Hudson Soft",
                "19" => "b-ai",
                "20" => "KSS",
                "22" => "POW",
                "24" => "PCM Complete",
                "25" => "San-X",
                "28" => "Kemco Japan",
                "29" => "Seta",
                "30" => "Viacom",
                "31" => "Nintendo",
                "32" => "Bandai",
                "33" => "Ocean/Acclaim",
                "34" => "Konami",
                "35" => "Hector",
                "37" => "Taito",
                "38" => "Hudson",
                "39" => "Banpresto",
                "41" => "Ubisoft",
                "42" => "Atlus",
                "44" => "Malibu",
                "46" => "Angel",
                "47" => "Bullet-Proof",
                "49" => "IREM",
                "50" => "Absolute",
                "51" => "Acclaim",
                "52" => "Activision",
                "53" => "American Sammy",
                "54" => "Konami",
                "55" => "Hi Tech Entertainment",
                "56" => "LJN",
                "57" => "Matchbox",
                "58" => "Mattel",
                "59" => "Milton Bradley",
                "60" => "Titus",
                "61" => "Virgin",
                "64" => "LucasArts",
                "67" => "Ocean",
                "69" => "EA (Electronic Arts)",
                "70" => "Infogrames",
                "71" => "Interplay",
                "72" => "Broderbund",
                "73" => "Sculptured",
                "75" => "sci",
                "78" => "THQ",
                "79" => "Accolade",
                "80" => "misawa",
                "83" => "lozc",
                "86" => "Tokuma Shoten",
                "87" => "Tsukoda Ori",
                "91" => "Chunsoft",
                "92" => "Video System",
                "93" => "Ocean/Acclaim",
                "95" => "Varie",
                "96" => "Yonezawa/s'pal",
                "97" => "Kaneko",
                "98" => "Pack in Soft",
                "A4" => "Konami (Yu-Gi-Oh!)",
                _ => "Unknown"
            }
        } else {
            // still todo
            licensee = match contents.get(0x14B).unwrap() {
                0x00 => "None",
                0x01 | 0x31 => "Nintendo",
                0x08 | 0x38 => "Capcom",
                0x09 => "hot-b",
                0x0A => "Jaleco",
                0x0B => "Coconuts",
                0x0C | 0x6E => "Elite Systems",
                0x13 | 0x69 => "EA (Electronic Arts)",
                0x18 => "Hudson Soft",
                0x19 => "ITC Entertainment",
                0x1A => "Yanoman",
                0x1D => "Clary",
                0x1F | 0x4A | 0x61 => "Virgin",
                0x20 => "KSS",
                0x24 => "PCM Complete",
                0x25 => "San-X",
                0x28 => "Kotobuki Systems",
                0x29 => "Seta",
                0x30 | 0x70 => "Infogrames",
                0x32 => "Bandai",
                0x34 => "Konami",
                0x35 => "Hector",
                0x39 => "Banpresto",
                0x3C => "*entertainment i",
                0x3E => "Gremlin",
                0x41 => "Ubisoft",
                0x42 => "Atlus",
                0x44 | 0x4D => "Malibu",
                0x46 => "Angel",
                0x47 => "Spectrum Holoby",
                0x49 => "IREM",
                0x4F => "U.S. Gold",
                0x50 => "Absolute",
                0x51 => "Acclaim",
                0x52 => "Activision",
                0x53 => "American Sammy",
                0x54 => "Gametek",
                0x55 => "Park Place",
                0x56 => "LJN",
                0x57 => "Matchbox",
                0x59 => "Milton Bradley",
                0x5A => "Mindscape",
                0x5B => "Romstar",
                0x5C => "Naxat Soft",
                0x5D => "Tradewest",
                0x60 => "Titus",
                0x67 => "Ocean",
                0x6F => "Electro Brain",
                0x71 => "Interplay",
                0x72 => "Broderbund",
                0x73 => "Sculptured Soft",
                0x75 => "The Sales Curve",
                0x78 => "THQ",
                0x79 => "Accolade",
                0x7A => "Traffix Entertainment",
                0x7C => "Microprose",
                0x7F => "Kemco",
                0x80 => "Misawa Entertainment",
                0x83 => "LOZC",
                0x86 => "Tokuma Shoten Intermedia",
                0x8B => "Bullet-Proof Software",
                0x8C => "Vic Tokai",
                0x8E => "Ape",
                0x8F => "I'MAX",
                0x91 => "Chunsoft",
                0x92 => "Video System",
                0x93 => "Tsuburava",
                _ => "Unknown"
            }
        }

        // Now get the cartridge type to set the features of the cart
        let cart_features: Vec<CartFeature> = match *contents.get(0x147).unwrap() {
            0x00 => vec![CartFeature::ROM],
            0x01 => vec![CartFeature::MBC1],
            0x02 => vec![CartFeature::MBC1, CartFeature::RAM],
            0x03 => vec![CartFeature::MBC1, CartFeature::RAM, CartFeature::Battery],
            0x05 => vec![CartFeature::MBC2],
            0x06 => vec![CartFeature::MBC2, CartFeature::Battery],
            0x08 => vec![CartFeature::ROM, CartFeature::RAM],
            0x09 => vec![CartFeature::ROM, CartFeature::RAM, CartFeature::Battery],
            0x0B => vec![CartFeature::MMM01],
            0x0C => vec![CartFeature::MMM01, CartFeature::RAM],
            0x0D => vec![CartFeature::MMM01, CartFeature::RAM, CartFeature::Battery],
            0x0F => vec![CartFeature::MBC3, CartFeature::Timer, CartFeature::Battery],
            0x10 => vec![CartFeature::MBC3, CartFeature::Timer, CartFeature::RAM, CartFeature::Battery],
            0x11 => vec![CartFeature::MBC3],
            0x12 => vec![CartFeature::MBC3, CartFeature::RAM],
            0x13 => vec![CartFeature::MBC3, CartFeature::RAM, CartFeature::Battery],
            0x19 => vec![CartFeature::MBC5],
            0x1A => vec![CartFeature::MBC5, CartFeature::RAM],
            0x1B => vec![CartFeature::MBC5, CartFeature::RAM, CartFeature::Battery],
            0x1C => vec![CartFeature::MBC5, CartFeature::Rumble],
            0x1D => vec![CartFeature::MBC5, CartFeature::Rumble, CartFeature::RAM],
            0x1E => vec![CartFeature::MBC5, CartFeature::Rumble, CartFeature::RAM, CartFeature::Battery],
            0x20 => vec![CartFeature::MBC6],
            0x22 => vec![CartFeature::MBC7, CartFeature::Sensor, CartFeature::Rumble, CartFeature::RAM, CartFeature::Rumble],
            0xFC => vec![CartFeature::PocketCamera],
            0xFD => vec![CartFeature::BandaiTama5],
            0xFE => vec![CartFeature::HuC3],
            0xFF => vec![CartFeature::HuC1, CartFeature::RAM, CartFeature::Battery],

            _ => vec![CartFeature::Unknown]
        };

        // The Mask ROM Version Number (usu. 0)
        let version: u8 = *contents.get(0x14C).unwrap();

        // Checksum for the header section only
        let header_checksum: u8 = *contents.get(0x14D).unwrap();

        // Checksum for the whole ROM
        let global_checksum: u16 = ((*contents.get(0x14E).unwrap() as u16) << 8) | (*contents.get(0x14F).unwrap() as u16);

        ROM {
            path: path.display().to_string(),
            contents: contents.clone(),
            size: contents.len(),
            title: title,
            licensee: String::from(licensee),
            cart_type: cart_features,
            version_no: version,
            header_checksum: header_checksum,
            global_checksum: global_checksum,
            gbs_compatible: gbs_compatible
        }
    }

    /// Verifies the ROM by checking a number of header features
    ///
    /// The following must all be true for a ROM to run properly
    ///
    /// - The 48 bytes starting from offset 0x104 must be *exactly* as follows:
    ///     CE ED 66 66 CC 0D 00 0B 03 73 00 83 00 0C 00 0D
    ///     00 08 11 1F 88 89 00 0E DC CC 6E E6 DD DD D9 99
    ///     BB BB 67 63 6E 0E EC CC DD DC 99 9F BB B9 33 3E
    /// This is the scrolling Nintendo™ graphic you see when you boot up a GameBoy
    ///
    /// - The header checksum must be correct. The header checksum is the sum of
    /// bytes 0x134 - 0x14C (i.e., the whole header starting after the scrolling
    /// Nintendo™ graphic and before the header checksum)
    ///
    /// The global checksum is not actually checked by the GameBoy. It is found by
    /// adding up all the bytes on the ROM except for the global checksum bytes.
    /// For the sake of emulating the hardware as closely as possible, an incorrect
    /// global checksum won't cause an error but a warning will be produced.
    ///
//    pub fn verify() -> Result<(), Error> {
//        // TODO
//    }

    pub fn info(&mut self) -> String {
        let mut cart_type = String::new();
        for feature in &self.cart_type {
            cart_type.push_str(&format!("{:?}", feature));
            cart_type.push('+');
        }

        cart_type.pop();

        format!("\
Verbose ROM information on {}\n\
Title:\t\t\t{}\n\
Version:\t\t{}\n\
Licensee:\t\t{}\n\
Cart Type:\t\t{}\n\
GBS Features:\t{}\n\
Checksum:\t\t0x{:04X}",
        self.path, self.title, self.version_no, self.licensee, cart_type,
        if self.gbs_compatible {"Available"} else {"Unavailable"}, self.global_checksum)
    }

    pub fn dump(&mut self) {
        for i in 0..self.contents.len() {
            if i % 16 == 0 {
                println!();
                print!("0x{:08X} ", i);
            }

            let ch: u8 = *self.contents.get(i).unwrap();
            if ch.is_ascii_control() || ch.is_ascii_whitespace() {
                print!(". ");
            } else {
                print!("{} ", ch as char);
            }
        }
    }
}

impl Debug for ROM {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut cart_type = String::new();
        for feature in &self.cart_type {
            cart_type.push_str(&format!("{:?}", feature));
            cart_type.push('+');
        }

        // Removes the trailing '+'
        cart_type.pop();

        write!(f, "ROM ({}, {}, {}, {})", self.title, self.version_no, cart_type, self.licensee)
    }
}

pub struct Instruction {
    asm: String,
    opcode: u8,
    description: String,
    size: usize,
    cycles: u32
}

impl Instruction {
    pub fn new(code: u8, asm: &str, desc: &str, bytes: usize, cycles: u32) -> Instruction {
        Instruction {
            opcode: code,
            asm: String::from(asm),
            description: String::from(desc),
            size: bytes,
            cycles: cycles
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.asm)
    }
}

