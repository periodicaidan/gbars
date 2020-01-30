use super::instruction::{Instruction, Arg};

pub enum CpuState {
    OpRead(OpRead),
    DataRead(DataRead),
    Exec,
}

pub enum OpRead {
    General,
    PrefixCB,
}

pub enum DataRead {
    Byte,
    ShortHi,
    ShortLo
}

/// The Zilog Z80 has an accumulator (A) and flag (F) register, along with 6 general-purpose
/// registers (B, C, D, E, H, and L). All of these are 8-bit but can double up (as AF, BC, DE, and
/// HL) to act as 16-bit registers, where A, B, D, and H store the high byte and F, C, E, and L
/// store the low byte. There are of course the two pointer registers SP (for the stack pointer) and
/// PC (for the program counter/instruction pointer).
pub struct Registers {
    pub a: u8, // accumulator
    pub f: u8, // flags
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16, // stack pointer
    pub pc: u16, // program counter
}

pub struct Cpu {
    state: CpuState,
    instruction: Instruction,
    registers: Registers,
    ip: usize,
}

impl Cpu {
    pub fn step(&mut self, memory: &mut [u8]) -> Result<(), String> {
        match self.state {
            CpuState::OpRead(OpRead::General) => {
                let opcode = memory.get(self.ip)?;
                let instr = Instruction::from_opcode(*opcode);
                match instr.arg {
                    Arg::None => if instr.opcode == 0xCB {
                        self.state = CpuState::OpRead(OpRead::PrefixCB);
                    } else {
                        self.state = CpuState::Exec
                    },

                    Arg::Addr8(_) |
                    Arg::Data8(_) |
                    Arg::Rel8(_) => self.state = CpuState::DataRead(DataRead::Byte),

                    Arg::Addr16(_) |
                    Arg::Data16(_) => self.state = CpuState::DataRead(DataRead::ShortHi),
                }

                self.ip += 1;
            },

            CpuState::OpRead(OpRead::PrefixCB) => {
                self.state = CpuState::Exec;
                self.ip += 1;
            },

            CpuState::DataRead(DataRead::Byte) => {
                let byte = memory.get(self.ip)?;
                match self.instruction.arg {
                    Arg::Addr8(mut addr) => addr = *byte,
                    Arg::Data8(mut data) => data = *byte,
                    Arg::Rel8(mut rel) => rel = *byte as i8,
                    _ => {}
                }

                self.state = CpuState::Exec;
                self.ip += 1
            },

            CpuState::DataRead(DataRead::ShortHi) => {
                let byte = memory.get(self.ip)?;
                match self.instruction.arg {
                    Arg::Addr16(mut addr) => addr |= ((*byte as u16) << 8),
                    Arg::Data16(mut data) => data |= ((*byte as u16) << 8),
                    _ => {}
                }

                self.state = CpuState::DataRead(DataRead::ShortLo);
                self.ip += 1;
            },

            CpuState::DataRead(DataRead::ShortLo) => {
                let byte = memory.get(self.ip)?;
                match self.instruction.arg {
                    Arg::Addr16(mut addr) => addr |= *byte as u16,
                    Arg::Data16(mut data) => data |= *byte as u16,
                    _ => {}
                }

                self.state = CpuState::Exec;
                self.ip += 1;
            },

            CpuState::Exec => {
                self.execute_instruction(memory);

                self.state = CpuState::OpRead(OpRead::General);
            }
        }

        Ok(())
    }

    fn execute_instruction(&mut self, memory: &mut [u8]) -> Result<(), String> {
        Ok(())
    }
}

impl Registers {
    pub fn get_bc(&mut self) -> u16 {
        
    }
}