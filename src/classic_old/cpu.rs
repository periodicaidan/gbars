use super::gb_types::{CPU, Memory, CPUStatus, Register};
use std::num::Wrapping;


impl CPU {
    pub fn exec(&mut self, opcode: u8) -> CPUStatus {
        use self::CPUStatus::*;

        println!("{:02X}", opcode);

        match opcode {
            0x00 => self.pc += 1,
            0x10 => return Stop,
            0x3C => self.inc(Register::A),
            0x76 => return Halt,
            0xC3 => {
                // absolute jump
            }
            0xCB => {
                // TODO
            },
            0xF3 => {},
            0xFB => {},
            _ => {
                panic!("UNKNOWN OPCODE: {:02X}", opcode)
            }
        }

        self.pc += 1;

        Continue
    }

    /* 8 BIT ARITHMETIC */

    pub fn inc(&mut self, r: Register) {
        use self::Register::*;
        match r {
            A => {
                let before = self.a;
                self.a.wrapping_add(1);
                let after = self.a;

                self.set_flags(
                    Some(if self.a == 0 { 1 } else { 0 }),
                    Some(0),
                    Some(Self::half_carry_occurred(before, after)),
                    None
                );
            },

            B => {
                let before = self.b;
                self.b += 1;
                let after = self.b;

                self.set_flags(
                    Some(if self.b == 0 { 1 } else { 0 }),
                    Some(0),
                    Some(Self::half_carry_occurred(before, after)),
                    None
                );
            },

            C => {
                let before = self.c;
                self.c += 1;
                let after = self.c;

                self.set_flags(
                    Some(if self.c == 0 { 1 } else { 0 }),
                    Some(0),
                    Some(Self::half_carry_occurred(before, after)),
                    None
                );
            },

            D => {
                let before = self.d;
                self.d += 1;
                let after = self.d;

                self.set_flags(
                    Some(if self.d == 0 { 1 } else { 0 }),
                    Some(0),
                    Some(Self::half_carry_occurred(before, after)),
                    None
                );
            },

            E => {
                let before = self.e;
                self.e += 1;
                let after = self.e;

                self.set_flags(
                    Some(if self.e == 0 { 1 } else { 0 }),
                    Some(0),
                    Some(Self::half_carry_occurred(before, after)),
                    None
                );
            },

            H => {
                let before = self.h;
                self.h += 1;
                let after = self.h;

                self.set_flags(
                    Some(if self.h == 0 { 1 } else { 0 }),
                    Some(0),
                    Some(Self::half_carry_occurred(before, after)),
                    None
                );
            },

            L => {
                let before = self.l;
                self.l += 1;
                let after = self.l;

                self.set_flags(
                    Some(if self.l == 0 { 1 } else { 0 }),
                    Some(0),
                    Some(Self::half_carry_occurred(before, after)),
                    None
                );
            },

            Address(m, addr) => {
                let addr = self.get_hl();
                let mut data = m.get_mut(addr as usize).unwrap();
                *data += 1;
            }

            // 16 bit inc's do not trigger any flags for some reason

            BC => {
                let before = self.get_bc();
                self.set_bc(before + 1);
            },

            DE => {
                let before = self.get_de();
                self.set_de(before + 1);
            },

            HL => {
                let before = self.get_hl();
                self.set_hl(before + 1);
            },

            SP => {
                self.sp += 1
            },

            _ => {}
        }
    }

    pub fn cmp(&mut self, other: u8) {
        let res = self.a.wrapping_sub(other);

        self.set_flags(
            Some(if res == 0 { 1 } else { 0 }),
            Some(1),
            Some(Self::half_borrow_occurred(res, self.a)),
            Some(if res > self.a { 1 } else { 0 })
        )
    }

    /* LOADS */

    pub fn load_8bit_reg(&mut self, target: Register, source: Register) {
        use self::Register::*;
        let s = match source {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            F => self.f,
            H => self.h,
            L => self.l,
            _ => panic!("Invalid source for 8-bit load: {:?}", source)
        };

        match target {
            A => self.a = s,
            B => self.b = s,
            C => self.c = s,
            D => self.d = s,
            E => self.e = s,
            F => self.f = s,
            H => self.h = s,
            L => self.l = s,
            _ => panic!("Invalid target for 8-bit load: {:?}", target)
        };
    }

    pub fn load_8bit_literal(&mut self, target: Register, source: u8) {
        use self::Register::*;
        match target {
            A => self.a = source,
            B => self.b = source,
            C => self.c = source,
            D => self.d = source,
            E => self.e = source,
            H => self.h = source,
            L => self.l = source,
            _ => panic!("Invalid target for 8-bit load: {:?}", target)
        };
    }

    pub fn load_16bit_reg(&mut self, target: Register, source: Register) {
        use self::Register::*;
        let s = match source {
            AF => self.get_af(),
            BC => self.get_bc(),
            DE => self.get_de(),
            HL => self.get_hl(),
            SP => self.sp,
            _ => panic!("Invalid source for 16-bit load: {:?}", source)
        };

        match target {
            AF => self.set_af(s),
            BC => self.set_bc(s),
            DE => self.set_de(s),
            HL => self.set_hl(s),
            SP => self.sp = s,
            _ => panic!("Invalid target for 16-bit load: {:?}", target)
        };
    }

    pub fn load_16bit_literal(&mut self, target: Register, source: u16) {
        use self::Register::*;
        match target {
            BC => self.set_bc(source),
            DE => self.set_de(source),
            HL => self.set_hl(source),
            SP => self.sp = source,
            _ => panic!("Invalid target for 16-bit load: {:?}", target)
        };
    }

    pub fn push(&mut self, source: Register) {
        use self::Register::*;

        self.sp.wrapping_add(1);

        let value = match source {
            BC => self.get_bc(),
            DE => self.get_de(),
            HL => self.get_hl(),
            AF => self.get_af(),
            _ => panic!("Invalid source for push: {:?}", source)
        };

        self.stack[self.sp as usize] = value;
    }

    pub fn pop(&mut self, target: Register) {
        use self::Register::*;
        let value = self.stack[self.sp as usize];
        match target {
            BC => self.set_bc(value),
            DE => self.set_de(value),
            HL => self.set_hl(value),
            AF => self.set_af(value),
            _ => panic!("Invalid target for pop: {:?}", target)
        }
        self.sp.wrapping_sub(1);
    }

    /* PREFIX CB BITWISE OPERATIONS */

    pub fn rlc(&mut self, target: Register) {
        use self::Register::*;

        let carry = match target {
            A => {
                let c = (self.a & 0b1000_0000) >> 7;
                self.a <<= 1;
                self.a |= c;
                c
            },
            B => {
                let c = (self.b & 0b1000_0000) >> 7;
                self.b <<= 1;
                self.b |= c;
                c
            },
            C => {
                let c = (self.c & 0b1000_0000) >> 7;
                self.c <<= 1;
                self.c |= c;
                c
            },
            D => {
                let c = (self.d & 0b1000_0000) >> 7;
                self.d <<= 1;
                self.d |= c;
                c
            },
            E => {
                let c = (self.e & 0b1000_0000) >> 7;
                self.e <<= 1;
                self.e |= c;
                c
            },
            H => {
                let c = (self.h & 0b1000_0000) >> 7;
                self.h <<= 1;
                self.h |= c;
                c
            },
            L => {
                let c = (self.l & 0b1000_0000) >> 7;
                self.l <<= 1;
                self.l |= c;
                c
            },
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                let c = (data & 0b1000_0000) >> 7;
                mem.write_byte(addr, (data << 1) | c);
                c
            },
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        };
    }

    pub fn rrc(&mut self, target: Register) {
        use self::Register::*;

        let carry = match target {
            A => {
                let c = self.a & 0b0000_0001;
                self.a >>= 1;
                self.a |= c << 7;
                c
            },
            B => {
                let c = self.b & 0b0000_0001;
                self.b >>= 1;
                self.b |= c << 7;
                c
            },
            C => {
                let c = self.c & 0b0000_0001;
                self.c >>= 1;
                self.c |= c << 7;
                c
            },
            D => {
                let c = self.d & 0b0000_0001;
                self.d >>= 1;
                self.d |= c << 7;
                c
            },
            E => {
                let c = self.e & 0b0000_0001;
                self.e >>= 1;
                self.e |= c << 7;
                c
            },
            H => {
                let c = self.h & 0b0000_0001;
                self.h >>= 1;
                self.h |= c << 7;
                c
            },
            L => {
                let c = self.l & 0b0000_0001;
                self.l >>= 1;
                self.l |= c << 7;
                c
            },
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                let c = data & 0b0000_0001;
                mem.write_byte(addr, (data >> 1) | c << 7);
                c
            },
            _ => panic!("Invalid targit for 8-bit arithmetic: {:?}", target)
        };

        self.set_flags(
            Some(if carry == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(carry)
        )
    }

    pub fn rl(&mut self, target: Register) {
        use self::Register::*;

        let carry = match target {
            A => {
                let c = (self.a & 0b1000_0000) >> 7;
                self.a <<= 1;
                self.a |= self.get_carry();
                c
            },
            B => {
                let c = (self.b & 0b1000_0000) >> 7;
                self.b <<= 1;
                self.b |= self.get_carry();
                c
            },
            C => {
                let c = (self.c & 0b1000_0000) >> 7;
                self.c <<= 1;
                self.c |= self.get_carry();
                c
            },
            D => {
                let c = (self.d & 0b1000_0000) >> 7;
                self.d <<= 1;
                self.d |= self.get_carry();
                c
            },
            E => {
                let c = (self.e & 0b1000_0000) >> 7;
                self.e <<= 1;
                self.e |= self.get_carry();
                c
            },
            H => {
                let c = (self.h & 0b1000_0000) >> 7;
                self.h <<= 1;
                self.h |= self.get_carry();
                c
            },
            L => {
                let c = (self.l & 0b1000_0000) >> 7;
                self.l <<= 1;
                self.l |= self.get_carry();
                c
            },
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                let c = (data & 0b1000_0000) >> 7;
                mem.write_byte(addr, (data << 1) | self.get_carry());
                c
            },
            _ => panic!("Invalid targit for 8-bit arithmetic: {:?}", target)
        };

        self.set_flags(
            Some(if carry == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(carry)
        )
    }

    pub fn rr(&mut self, target: Register) {
        use self::Register::*;

        let carry = match target {
            A => {
                let c = self.a & 0b0000_0001;
                self.a >>= 1;
                self.a |= self.get_carry() << 7;
                c
            },
            B => {
                let c = self.b & 0b0000_0001;
                self.b >>= 1;
                self.b |= self.get_carry() << 7;
                c
            },
            C => {
                let c = self.c & 0b0000_0001;
                self.c >>= 1;
                self.c |= self.get_carry() << 7;
                c
            },
            D => {
                let c = self.d & 0b0000_0001;
                self.d >>= 1;
                self.d |= self.get_carry() << 7;
                c
            },
            E => {
                let c = self.e & 0b0000_0001;
                self.e >>= 1;
                self.e |= self.get_carry() << 7;
                c
            },
            H => {
                let c = self.h & 0b0000_0001;
                self.h >>= 1;
                self.h |= self.get_carry() << 7;
                c
            },
            L => {
                let c = self.l & 0b0000_0001;
                self.l >>= 1;
                self.l |= self.get_carry() << 7;
                c
            },
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                let c = data & 0b0000_0001;
                mem.write_byte(addr, (data >> 1) | self.get_carry() << 7);
                c
            },
            _ => panic!("Invalid targit for 8-bit arithmetic: {:?}", target)
        };

        self.set_flags(
            Some(if carry == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(carry)
        )
    }

    pub fn sla(&mut self, target: Register) {
        use self::Register::*;

        let carry = match target {
            A => {
                let c = (self.a & 0b1000_0000) >> 7;
                self.a <<= 1;
                c
            },
            B => {
                let c = (self.b & 0b1000_0000) >> 7;
                self.b <<= 1;
                c
            },
            C => {
                let c = (self.c & 0b1000_0000) >> 7;
                self.c <<= 1;
                c
            },
            D => {
                let c = (self.d & 0b1000_0000) >> 7;
                self.d <<= 1;
                c
            },
            E => {
                let c = (self.e & 0b1000_0000) >> 7;
                self.e <<= 1;
                c
            },
            H => {
                let c = (self.h & 0b1000_0000) >> 7;
                self.h <<= 1;
                c
            },
            L => {
                let c = (self.l & 0b1000_0000) >> 7;
                self.l <<= 1;
                c
            },
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                let c = (data & 0b1000_0000) >> 7;
                mem.write_byte(addr, data << 1);
                c
            },
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        };

        self.set_flags(
            Some(if carry == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(carry)
        )
    }

    pub fn sra(&mut self, target: Register) {
        use self::Register::*;

        let (result, carry) = match target {
            A => {
                let c = self.a & 0b0000_0001;
                let leftmost_bit = self.a & 0b1000_0000;
                self.a >>= 1;
                self.a |= leftmost_bit;
                (self.a, c)
            },
            B => {
                let c = self.b & 0b0000_0001;
                let leftmost_bit = self.b & 0b1000_0000;
                self.b >>= 1;
                self.b |= leftmost_bit;
                (self.b, c)
            },
            C => {
                let c = self.c & 0b0000_0001;
                let leftmost_bit = self.c & 0b1000_0000;
                self.c >>= 1;
                self.c |= leftmost_bit;
                (self.c, c)
            },
            D => {
                let c = self.d & 0b0000_0001;
                let leftmost_bit = self.d & 0b1000_0000;
                self.d >>= 1;
                self.d |= leftmost_bit;
                (self.d, c)
            },
            E => {
                let c = self.e & 0b0000_0001;
                let leftmost_bit = self.e & 0b1000_0000;
                self.e >>= 1;
                self.e |= leftmost_bit;
                (self.e, c)
            },
            H => {
                let c = self.h & 0b0000_0001;
                let leftmost_bit = self.h & 0b1000_0000;
                self.h >>= 1;
                self.h |= leftmost_bit;
                (self.h, c)
            },
            L => {
                let c = self.l & 0b0000_0001;
                let leftmost_bit = self.l & 0b1000_0000;
                self.l >>= 1;
                self.l |= leftmost_bit;
                (self.l, c)
            },
            Address(mut mem, addr) => {
                let mut data = mem.read_byte(addr);
                let c = data & 0b0000_0001;
                data = (data >> 1) | (data & 0b1000_0000);
                mem.write_byte(addr, data);
                (data, c)
            },
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        };
    }

    pub fn swap(&mut self, target: Register) {
        use self::Register::*;

        let result = match target {
            A => {
                let tmp = (self.a & 0xF0) >> 4;
                self.a <<= 4;
                self.a |= tmp;
                self.a
            },
            B => {
                let tmp = (self.b & 0xF0) >> 4;
                self.b <<= 4;
                self.b |= tmp;
                self.b
            },
            C => {
                let tmp = (self.c & 0xF0) >> 4;
                self.c <<= 4;
                self.c |= tmp;
                self.c
            },
            D => {
                let tmp = (self.d & 0xF0) >> 4;
                self.d <<= 4;
                self.d |= tmp;
                self.d
            },
            E => {
                let tmp = (self.e & 0xF0) >> 4;
                self.e <<= 4;
                self.e |= tmp;
                self.e
            },
            H => {
                let tmp = (self.h & 0xF0) >> 4;
                self.h <<= 4;
                self.h |= tmp;
                self.h
            },
            L => {
                let tmp = (self.l & 0xF0) >> 4;
                self.l <<= 4;
                self.l |= tmp;
                self.l
            },
            Address(mut mem, addr) => {
                let mut data = mem.read_byte(addr);
                let tmp = (data & 0xF0) >> 4;
                data <<= 4;
                data |= tmp;
                mem.write_byte(addr, data);
                data
            },
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        };

        self.set_flags(
            Some(if result == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(0)
        )
    }

    pub fn srl(&mut self, target: Register) {
        use self::Register::*;

        let (result, carry) = match target {
            A => {
                let c = self.a & 0b0000_0001;
                self.a >>= 1;
                (self.a, c)
            },
            B => {
                let c = self.b & 0b0000_0001;
                self.b >>= 1;
                (self.b, c)
            },
            C => {
                let c = self.c & 0b0000_0001;
                self.c >>= 1;
                (self.c, c)
            },
            D => {
                let c = self.d & 0b0000_0001;
                self.d >>= 1;
                (self.d, c)
            },
            E => {
                let c = self.e & 0b0000_0001;
                self.e >>= 1;
                (self.e, c)
            },
            H => {
                let c = self.h & 0b0000_0001;
                self.h >>= 1;
                (self.h, c)
            },
            L => {
                let c = self.l & 0b0000_0001;
                self.l >>= 1;
                (self.l, c)
            },
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                let c = data & 0b0000_0001;
                mem.write_byte(addr, data >> 1);
                (data, c)
            },
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        };

        self.set_flags(
            Some(if result == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(carry)
        )
    }

    pub fn check_bit(&mut self, bit: u8, target: Register) {
        use self::Register::*;
        let mask = 1 << bit;
        let result = match target {
            A => self.a & mask,
            B => self.b & mask,
            C => self.c & mask,
            D => self.d & mask,
            E => self.e & mask,
            H => self.h & mask,
            L => self.l & mask,
            Address(mem, addr) => mem.read_byte(addr) & mask,
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        };

        self.set_flags(
            Some(if result == 0 { 1 } else { 0 }),
            Some(0),
            Some(1),
            None
        );
    }

    pub fn reset_bit(&mut self, bit: u8, target: Register) {
        use self::Register::*;

        let mask = !(1 << bit);
        match target {
            A => self.a &= mask,
            B => self.b &= mask,
            C => self.c &= mask,
            D => self.d &= mask,
            E => self.e &= mask,
            H => self.h &= mask,
            L => self.l &= mask,
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                mem.write_byte(addr, data & mask);
            },
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        }

        // Weirdly, this doesn't set any flags. Not even Z.
    }

    pub fn set_bit(&mut self, bit: u8, target: Register) {
        use self::Register::*;

        let mask = 1 << bit;
        match target {
            A => self.a |= mask,
            B => self.b |= mask,
            C => self.c |= mask,
            D => self.d |= mask,
            E => self.e |= mask,
            H => self.h |= mask,
            L => self.l |= mask,
            Address(mut mem, addr) => {
                let data = mem.read_byte(addr);
                mem.write_byte(addr, data | mask);
            },
            _ => panic!("Invalid target for 8-bit arithmetic: {:?}", target)
        }
    }

    /* 16 BIT REGISTERS */

    pub fn get_af(&self) -> u16 { (self.a as u16) << 8 | self.f as u16 }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = (value & 0x00FF) as u8;
    }

    pub fn get_bc(&self) -> u16 { (self.b as u16) << 8 | self.c as u16 }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn get_de(&self) -> u16 { (self.d as u16) << 8 | self.e as u16 }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn get_hl(&self) -> u16 { (self.h as u16) << 8 | self.l as u16 }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }

    /* 16-BIT ARITHMETIC */

    pub fn add_to_hl(&mut self, value: u16) {
        let before = self.get_hl();
        let after = before.wrapping_add(value);
        self.set_hl(after);

        self.set_flags(
            None,
            Some(0),
            Some(Self::half_carry_occurred(((before & 0xFF00) >> 8) as u8, ((after & 0xFF00) >> 8) as u8)),
            Some(if after < before { 1 } else { 0 })
        );
    }

    pub fn get_bit(&mut self, reg: Register, n: u8) -> u8 {
        use self::Register::*;
        let bit = match reg {
            A => (self.a << n) & 1,
            B => (self.b << n) & 1,
            C => (self.c << n) & 1,
            D => (self.d << n) & 1,
            E => (self.e << n) & 1,
            H => (self.h << n) & 1,
            L => (self.l << n) & 1,
            _ => panic!("Invalid register for 8-bit operation: {:?}", reg)
        };

        self.set_flags(
            Some(if bit == 0 { 1 } else { 0 }),
            Some(0),
            Some(1),
            None
        );

        bit
    }

    // As it happens, there is no opcode that would trigger a "sub_from_hl" function
    // 16-bit arithmetic is very limited on the GB

    pub fn set_flags(&mut self,
                     z: Option<u8>,
                     n: Option<u8>,
                     h: Option<u8>,
                     c: Option<u8>) {
        let mut flag = 0u8;
        for f in [z, n, h, c].iter() {
            if let Some(b) = f {
                flag |= b;
            }

            flag <<= 1;
        }

        self.f = flag << 3;
    }

    pub fn get_zero(&self) -> u8 { (self.f & 0b1000_0000) >> 7 }

    pub fn get_subtract(&self) -> u8 { (self.f & 0b0100_0000) >> 6 }

    pub fn get_half_carry(&self) -> u8 { (self.f & 0b0010_0000) >> 5 }

    pub fn get_carry(&self) -> u8 { (self.f & 0b0001_0000) >> 4 }

    pub fn half_carry_occurred(b: u8, a: u8) -> u8 {
        // A half-carry is triggered when there's a carry from the 3rd to 4th bit for 8-bit or
        // from the 11th to 12th for 16-bit. The way to check this is if the sum of the 4 least-
        // significant bits of the values before and after the computation carries
        //
        // ex:
        //
        // 0b0000_1101 (13) + 0b0000_0111 (7) = 0b0001_0100 (20)
        // (carries from 3rd to 4th bit => half-carry occurs)
        //
        // 1101 + 0111 = (1) 0100
        //                ^
        //                |
        //                +------ carry from adding lower nibbles => half-carry occurred
        //
        // 0b0101_0110 (86) + 0b0000_1100 (12) = 0b01100010 (98)
        // (carries from 3rd to 4th bit => half-carry occurs)
        //
        // 0110 + 1100 = (1) 0010
        //                ^
        //                |
        //                +------ carry from adding lower nibbles => half-carry occurred
        //
        // 0b0001_0000 (16) + 0b0000_0010 (2) = 0b0001_0010 (18)
        // (no carry from 3rd to 4th bit => half-carry does not occur)
        //
        // 0000 + 0010 = (0) 0010
        //                ^
        //                |
        //                +------ no carry from adding lower nibbles => no half-carry occurred

        if ((b & 0x0F) + (a & 0x0F)) & 0x10 == 0x10 { 1 } else { 0 }
    }

    pub fn half_borrow_occurred(b: u8, a: u8) -> u8 {
        // A half-borrow is the inverse of a half-carry. It's triggered when the 4th bit is borrowed
        // by the 3rd bit in 8-bit arithmetic or the 12th by the 11th in 16-bit. This can be worked
        // out by similar logic as with finding a half-carry, but you must flip the bits of the
        // bottom nibble of the value before subtraction.
        //
        // 0b0001_0110 (22) - 0b0000_1010 (10) = 0b0000_1100 (12)
        // (The 3rd bit borrows from the 4th so a half-borrow occurs)
        //
        // ~0110 + 1100 = 1001 + 1100 = (1) 0101
        //                               ^
        //                               |
        //                               +------- carries from adding the nibbles so half-borrow occurred
        //
        // 0b0001_1100 (28) - 0b0000_1010 (10) = 0b0001_0010 (18)
        // (The 3rd bit does not borrow from the 4th and so a half-borrow doesn't occur)
        //
        // ~1100 + 1010 = 0011 + 1010 = (0) 1101
        //                               ^
        //                               |
        //                               +------- no carry from adding the nibbles so no half-borrow occurred

        if ((!b & 0x0F) + (a & 0x0F)) & 0x10 == 0x10 { 1 } else { 0 }
    }
}