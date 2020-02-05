use super::instruction::{Instruction, Arg};
use super::registers::Registers;
use bitmatch::bitmatch;
use std::ops::Add;
use super::registers::Reg8;
use super::memory::MBC;
//use glutin::platform::macos::ActivationPolicy::Regular;

/// The CPU here is conceptualized as a state machine with some frills. Consuming a byte from memory
/// changes its state.
pub struct Cpu {
    state: CpuState,
    instruction: Instruction,
    registers: Registers,
    ip: usize,
}

/// There are 3 basic state. In the `OpRead` state, the CPU reads the next byte in memory as an
/// opcode. In the `DataRead` state, the CPU reads it as data or partial data (a byte, an address,
/// an offset, etc.). And in the `Exec` state, the CPU executes the current instruction.
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

impl Cpu {
    /// Performs some action based on the CPU's state, and then transitions to the next state.
    pub fn step(&mut self, memory_controller: &mut MBC) -> Result<(), String> {
        match self.state {
            // This is the initial state of the CPU. In this state, it reads the next byte in memory
            // as an opcode and decodes it as an instruction. The CPU then transitions to the next
            // state based on the argument the instruction expects.
            CpuState::OpRead(OpRead::General) => {
                let opcode = memory_controller.read_rom(self.registers.pc as usize).unwrap();
                let instr = Instruction::from_opcode(opcode);
                match instr.arg {
                    // If the instruction requires no arguments, we first check if it's a prefixed
                    // instruction (with opcode 0xCB). If it is, we transition to the
                    // `OpRead::PrefixCB` state. Otherwise, we move right on to the `Exec` state.
                    Arg::None => if instr.opcode == 0xCB {
                        self.state = CpuState::OpRead(OpRead::PrefixCB);
                    } else {
                        self.state = CpuState::Exec
                    },

                    // If the instruction requires 8-bit data, we transition to the
                    // `DataRead::Byte` state.
                    Arg::Addr8(_) |
                    Arg::Data8(_) |
                    Arg::Offset8(_) => self.state = CpuState::DataRead(DataRead::Byte),

                    // And if the instruction requires 16-bit data, it transitions to the
                    // `DataRead::ShortHi` state (since the next byte is the high-byte of whatever
                    // data it needs)
                    Arg::Addr16(_) |
                    Arg::Data16(_) => self.state = CpuState::DataRead(DataRead::ShortHi),
                }

                self.registers.pc.wrapping_add(1);
            },

            // In this state, the next byte in memory is read as a *prefixed* opcode, which has its
            // own instruction set.
            CpuState::OpRead(OpRead::PrefixCB) => {
                let byte = memory_controller.read_rom(self.registers.pc as usize).unwrap();
                self.state = CpuState::Exec;
                self.registers.pc.wrapping_add(1);
            },

            // In this state the next byte in memory is read as a literal byte and then the
            // CPU transitions to the `Exec` state.
            CpuState::DataRead(DataRead::Byte) => {
                let byte = memory_controller.read_rom(self.registers.pc as usize).unwrap();
                match self.instruction.arg {
                    Arg::Addr8(mut addr) => addr = byte,
                    Arg::Data8(mut data) => data = byte,
                    Arg::Offset8(mut offset) => offset = byte as i8,
                    _ => {}
                }

                self.state = CpuState::Exec;
                self.registers.pc.wrapping_add(1);
            },

            // The next byte in memory is read as the high nibble of a literal short and then the
            // CPU transitions to the `DataRead::ShortLo` state to get the low nibble.
            CpuState::DataRead(DataRead::ShortHi) => {
                let byte = memory_controller.read_rom(self.registers.pc as usize).unwrap();
                match self.instruction.arg {
                    Arg::Addr16(mut addr) => addr |= ((byte as u16) << 8),
                    Arg::Data16(mut data) => data |= ((byte as u16) << 8),
                    _ => {}
                }

                self.state = CpuState::DataRead(DataRead::ShortLo);
                self.registers.pc.wrapping_add(1);
            },

            // The next byte in memory is read as the low nibble of a literal short. This is
            // combined with the high nibble obtained in the previous state to form a whole 16-bit
            // unsigned short. Then the CPU transitions to the `Exec` state.
            CpuState::DataRead(DataRead::ShortLo) => {
                let byte = memory_controller.read_rom(self.registers.pc as usize).unwrap();
                match self.instruction.arg {
                    Arg::Addr16(mut addr) => addr |= byte as u16,
                    Arg::Data16(mut data) => data |= byte as u16,
                    _ => {}
                }

                self.state = CpuState::Exec;
                self.registers.pc.wrapping_add(1);
            },

            // In this state no bytes are read from memory and the program counter is not
            // progressed. Instead, the full instruction with its argument is executed by the CPU
            // and then the CPU is put back into the `OpRead::General` state to begin formulating
            // the next instruction.
            CpuState::Exec => {
                if self.instruction.prefixed {
                    self.execute_prefixed_instruction(memory_controller);
                } else {
                    self.execute_instruction(memory_controller);
                }

                self.state = CpuState::OpRead(OpRead::General);
            }
        }

        Ok(())
    }

    /// Executes the current (unprefixed) instruction
    ///
    /// TODO:
    ///     - rl[c]a
    ///     - rr[c]a
    #[bitmatch]
    fn execute_instruction(&mut self, memory_controller: &mut MBC) -> Result<(), String> {
        let opcode = self.instruction.opcode;
        let arg = &self.instruction.arg;

        #[bitmatch]
        match opcode {
            // no operation
            "0000_0000" => {},

            // stop
            "0001_0000" => {},

            // disable interrupts
            "1111_0011" => {},

            // enable interrupts
            "1111_1011" => {},

            // prefixed instruction (this case isn't possible with this setup but cases must be exhaustive)
            "1100_1011" => {},

            // decimal-adjust register A
            "0010_0111" => self.registers.daa(),

            // complement register A
            "0010_1111" => self.registers.cpl(),

            // set carry flag
            "0011_0111" => self.registers.set_flags(
                None,
                Some(false),
                Some(false),
                Some(true)
            ),

            // complement carry flag
            "0011_1111" => self.registers.set_flags(
                None,
                Some(false),
                Some(false),
                Some(!self.registers.carry())
            ),

            // load immediate 16-bit value
            "00xx_0001" => if let &Arg::Data16(data) = arg {
                match x {
                    0b00 => self.registers.set_bc(data),
                    0b01 => self.registers.set_de(data),
                    0b10 => self.registers.set_hl(data),
                    0b11 => self.registers.sp = data,
                    _ => {}
                }
            },

            // load A into a stored memory location
            "00xx_0010" => match x {
                0b00 => {
                    memory_controller.write_ram(self.registers.get_bc() as usize, self.registers.a.0);
                },
                0b01 => {
                    memory_controller.write_ram(self.registers.get_de() as usize, self.registers.a.0);
                },
                0b10 => {
                    let res = memory_controller.write_ram(self.registers.get_hl() as usize, self.registers.a.0);
                    self.registers.do_hl(|hl| hl.wrapping_add(1));
                },
                0b11 => {
                    let res = memory_controller.write_ram(self.registers.get_hl() as usize, self.registers.a.0);
                    self.registers.do_hl(|hl| hl.wrapping_sub(1));
                },
                _ => {}
            },

            // load the data at a memory location stored into A
            "00xx_1010" => match x {
                0b00 => self.registers.a.0 = memory_controller.read_ram(self.registers.get_bc() as usize).unwrap(),
                0b01 => self.registers.a.0 = memory_controller.read_ram(self.registers.get_de() as usize).unwrap(),
                0b10 => {
                    self.registers.a.0 = memory_controller.read_ram(self.registers.get_hl() as usize).unwrap();
                    self.registers.do_hl(|hl| hl.wrapping_add(1));
                },
                0b11 => {
                    self.registers.a.0 = memory_controller.read_ram(self.registers.get_hl() as usize).unwrap();
                    self.registers.do_hl(|hl| hl.wrapping_sub(1));
                },
                _ => {}
            }

            // 16-bit increment
            "00xx_0011" => if let Arg::None = arg {
                match x {
                    0b00 => self.registers.do_bc(|bc| bc + 1),
                    0b01 => self.registers.do_de(|de| de + 1),
                    0b10 => self.registers.do_hl(|hl| hl + 1),
                    0b11 => self.registers.sp = self.registers.sp.wrapping_add(1),
                    _ => {}
                }
            },

            // 16-bit decrement
            "00xx_1011" => if let Arg::None = arg {
                match x {
                    0b00 => self.registers.do_bc(|bc| bc - 1),
                    0b01 => self.registers.do_de(|de| de - 1),
                    0b10 => self.registers.do_hl(|hl| hl - 1),
                    0b11 => self.registers.sp = self.registers.sp.wrapping_sub(1),
                    _ => {}
                }
            }

            // 8-bit increment
            "00xx_x100" => if let Arg::None = arg {
                match x {
                    0b000 => self.registers.b += 1,
                    0b001 => self.registers.c += 1,
                    0b010 => self.registers.d += 1,
                    0b011 => self.registers.e += 1,
                    0b100 => self.registers.h += 1,
                    0b101 => self.registers.l += 1,
                    0b110 => {
                        let data = memory_controller.read_ram(self.registers.get_hl() as usize).unwrap();
                        memory_controller.write_ram(self.registers.get_hl() as usize, data + 1);
                    },
                    0b111 => self.registers.a += 1,
                    _ => {}
                }
            }

            // 8-bit decrement
            "00xx_x101" => if let Arg::None = arg {
                match x {
                    0b000 => self.registers.b -= 1,
                    0b001 => self.registers.c -= 1,
                    0b010 => self.registers.d -= 1,
                    0b011 => self.registers.e -= 1,
                    0b100 => self.registers.h -= 1,
                    0b101 => self.registers.l -= 1,
                    0b110 => {
                        let data = memory_controller.read_ram(self.registers.get_hl() as usize).unwrap();
                        memory_controller.write_ram(self.registers.get_hl() as usize, data - 1);
                    },
                    0b111 => self.registers.a -= 1,
                    _ => {}
                }
            },

            // load immediate 8-bit value
            "00xx_x110" => if let &Arg::Data8(data) = arg {
                match x {
                    0b000 => self.registers.b.load(data),
                    0b001 => self.registers.c.load(data),
                    0b010 => self.registers.d.load(data),
                    0b011 => self.registers.e.load(data),
                    0b100 => self.registers.h.load(data),
                    0b101 => self.registers.l.load(data),
                    0b110 => {
                        memory_controller.write_ram(self.registers.get_hl() as usize, data);
                    },
                    0b111 => self.registers.a.load(data),
                    _ => {}
                }
            },

            // load stored 8-bit value
            "01xx_xxxx" => if let Arg::None = arg {
                // halt
                if opcode == 0x76 {

                }

                #[bitmatch] let "ootttsss" = x;

                let data = match s {
                    0b000 => self.registers.b.0,
                    0b001 => self.registers.c.0,
                    0b010 => self.registers.d.0,
                    0b011 => self.registers.e.0,
                    0b100 => self.registers.h.0,
                    0b101 => self.registers.l.0,
                    0b110 => memory_controller.read_ram(self.registers.get_hl() as usize).unwrap(),
                    0b111 => self.registers.a.0,
                    _ => panic!()
                };

                match t {
                    0b000 => self.registers.b.load(data),
                    0b001 => self.registers.c.load(data),
                    0b010 => self.registers.d.load(data),
                    0b011 => self.registers.e.load(data),
                    0b100 => self.registers.h.load(data),
                    0b101 => self.registers.l.load(data),
                    0b110 => {
                        memory_controller.write_ram(self.registers.get_hl() as usize, data);
                    },
                    0b111 => self.registers.a.load(data),
                    _ => panic!()
                }
            },

            // accumulator arithmetic
            "10xx_xxxx" => if let Arg::None = arg {
                #[bitmatch] let "ooff_ffsss" = x;
                let data = match s {
                    0b000 => self.registers.b.0,
                    0b001 => self.registers.c.0,
                    0b010 => self.registers.d.0,
                    0b011 => self.registers.e.0,
                    0b100 => self.registers.h.0,
                    0b101 => self.registers.l.0,
                    0b110 => memory_controller.read_ram(self.registers.get_hl() as usize).unwrap(),
                    0b111 => self.registers.a.0,
                    _ => panic!()
                };

                match f {
                    0b000 => self.registers.add(data),
                    0b001 => self.registers.adc(data),
                    0b010 => self.registers.sub(data),
                    0b011 => self.registers.sbc(data),
                    0b100 => self.registers.and(data),
                    0b101 => self.registers.xor(data),
                    0b110 => self.registers.or(data),
                    0b111 => self.registers.cp(data),
                    _ => {}
                }
            },

            "11xx_x110" => if let &Arg::Data8(data) = arg {
                match x {
                    0b000 => self.registers.add(data),
                    0b001 => self.registers.adc(data),
                    0b010 => self.registers.sub(data),
                    0b011 => self.registers.sbc(data),
                    0b100 => self.registers.and(data),
                    0b101 => self.registers.xor(data),
                    0b110 => self.registers.or(data),
                    0b111 => self.registers.cp(data),
                    _ => panic!()
                }
            },

            // 16-bit arithmetic
            "00xx_1001" => {
                let source = match x {
                    0b00 => self.registers.get_bc(),
                    0b01 => self.registers.get_de(),
                    0b10 => self.registers.get_hl(),
                    0b11 => self.registers.sp,
                    _ => panic!()
                };

                self.registers.do_hl(|hl| hl + source);
            },

            // pop the stack
            "11xx_0001" => if let Arg::None = arg {

            },

            // push on the stack
            "11xx_0101" => if let Arg::None = arg {

            },

            // Call a reset
            "11xx_x111" => if let Arg::None = arg {
//                self.call_reset(memory, x * 8);
            },

            // relative jumps
            "0001_1000" => if let &Arg::Offset8(offset) = arg {
                if offset < 0 {
                    self.registers.pc -= (-offset) as u16;
                } else {
                    self.registers.pc += offset as u16;
                }
            },

            "001x_x000" => if let &Arg::Offset8(offset) = arg {
                let cond = match x {
                    0b00 => !self.registers.zero(),
                    0b01 => self.registers.zero(),
                    0b10 => !self.registers.carry(),
                    0b11 => self.registers.carry(),
                    _ => panic!()
                };

                if cond {
                    if offset < 0 {
                        self.registers.pc -= (-offset) as u16;
                    } else {
                        self.registers.pc += offset as u16;
                    }
                }
            },

            // absolute jumps
            "1100_0011" => if let &Arg::Addr16(addr) = arg {
                self.registers.pc = addr;
            },

            "1110_1001" => if let Arg::None = arg {
                //
            },

            "110x_x010" => if let &Arg::Addr16(addr) = arg {
                let cond = match x {
                    0b00 => !self.registers.zero(),
                    0b01 => self.registers.zero(),
                    0b10 => !self.registers.carry(),
                    0b11 => self.registers.carry(),
                    _ => panic!()
                };

                if cond {
                    self.registers.pc = addr;
                }
            },

            // calls
            "1100_1101" => if let &Arg::Addr16(addr) = arg {
                // push the next address onto the stack

                self.registers.pc = addr;
            },

            "110x_x100" => if let &Arg::Addr16(addr) = arg {
                let cond = match x {
                    0b00 => !self.registers.zero(),
                    0b01 => self.registers.zero(),
                    0b10 => !self.registers.carry(),
                    0b11 => self.registers.carry(),
                    _ => panic!()
                };

                if cond {
                    // Push next address onto the stack

                    self.registers.pc = addr;
                }
            },

            // returns
            "110x_1001" => if let Arg::None = arg {
                if x == 1 {
                    // reti
                } else {
                    // ret
                }
            }

            "110x_x000" => if let Arg::None = arg {
                let cond = match x {
                    0b00 => !self.registers.zero(),
                    0b01 => self.registers.zero(),
                    0b10 => !self.registers.carry(),
                    0b11 => self.registers.carry(),
                    _ => panic!()
                };

                if cond {
                    // ret
                }
            },

            // accumulator rotations
            "000x_x111" => match x {
                0b00 => self.registers.a.rot_left(),
                0b01 => self.registers.a.rot_right(),
                0b10 => self.registers.a.rot_left(),
                0b11 => self.registers.a.rot_right(),
                _ => {}
            },

            // immediate address loads
            "111x_0000" => if let &Arg::Addr8(half_addr) = arg {
                let addr = 0xFF00 + (half_addr as usize);

                if x == 0 {
                    memory_controller.write_ram(addr, self.registers.a.0);
                } else {
                    self.registers.a.0 = memory_controller.read_ram(addr).unwrap();
                }
            },

            "111x_0010" => {
                let addr = 0xFF00 + (self.registers.c.0 as usize);

                if x == 0 {
                    memory_controller.write_ram(addr, self.registers.a.0);
                } else {
                    self.registers.a.0 = memory_controller.read_ram(addr).unwrap();
                }
            },

            "111x_1010" => if let &Arg::Addr16(addr) = arg {
                if x == 0 {
                    memory_controller.write_ram(addr as usize, self.registers.a.0);
                } else {
                    self.registers.a.0 = memory_controller.read_ram(addr as usize).unwrap();
                }
            },

            // stack pointer loads
            "0000_1000" => if let &Arg::Addr16(addr) = arg {
                memory_controller.write_ram(addr as usize, (self.registers.sp & 0xF0) as u8);
                memory_controller.write_ram((addr + 1) as usize, (self.registers.sp & 0x0F) as u8);
            },

            "1111_1000" => if let &Arg::Offset8(offset) = arg {
                let data = if offset < 0 {
                    self.registers.sp.wrapping_sub(-offset as u16)
                } else {
                    self.registers.sp.wrapping_add(offset as u16)
                };

                self.registers.set_hl(data);
            },

            "1111_1001" => {
                let hl = self.registers.get_hl();
                self.registers.sp = hl;
            },

            // stack pointer arithmetic
            "1110_1000" => if let &Arg::Offset8(offset) = arg {
                let new_value = if offset < 0 {
                    self.registers.sp.wrapping_sub(-offset as u16)
                } else {
                    self.registers.sp.wrapping_add(offset as u16)
                };

                self.registers.sp = new_value;
            },

            // unused
            "1101_?011" => {},
            "1101_1101" => {},
            "1110_?011" => {},
            "111?_?100" => {},
            "111?_1101" => {}
        }

        Ok(())
    }

    fn execute_prefixed_instruction(&mut self, memory: &mut MBC) -> Result<(), String> {
        Ok(())
    }
}