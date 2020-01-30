use self::super::utils::*;
use self::super::registers::Registers;
use std::collections::*;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;

pub enum Instruction<'a> {
    Nop,
    Load8BitRegister(&'a Fn(&mut Registers, u8)),
    Load16BitRegister(&'a Fn(&mut Registers, u16)),
    LoadMemoryAddress(&'a Fn(&mut [u8], u16, &u16)),
    Arithmetic8Bit(&'a Fn(&mut Registers)),
    Arithmetic16Bit(&'a Fn(&mut Registers)),
    Bitwise(&'a Fn(&mut Registers))
}

pub struct Command {
    z80: String,
    opcode: u8,
    cmd: Instruction<'static>,
    description: String,
    bytes: usize
}

impl Command {
    pub fn new(code: u8, inst: &str, desc: &str, bytes: usize, cmd: Instruction<'static>) -> Command {
        Command {
            opcode: code,
            z80: String::from(inst),
            description: String::from(desc),
            cmd: cmd,
            bytes: bytes
        }
    }

    pub fn exec(&mut self, regs: &Registers, pc: u16, memory: &[u8]) {
        use self::Instruction::*;

        let cmd = &self.cmd;
        let mut args = Vec::<u8>::new();

        for i in 1..self.bytes {
            args.push(memory[pc as usize + i])
        }
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.z80)
    }
}

//lazy_static! {
//    pub static ref OPCODES: [Option<Command>; 0x100] =
//}