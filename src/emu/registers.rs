#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16
}

impl Registers {
    pub fn init() -> Registers {
        Registers{
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0x100
        }
    }

    /* 8-BIT ARITHMETIC FUNCTIONS */
    // These are to allow for over-/underflow wrapping and flag setting
    // Only the accumulator register (the A register) can undergo 8BA

    pub fn add(&mut self, val: u8) {
        let before = self.a;
        self.a = self.a.wrapping_add(val);
        let after = self.a;

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            Some(0),
            Some(if Registers::half_carry_occurred(before, after) { 1 } else { 0 }),
            Some(if before > after { 1 } else { 0 })
        );
    }

    pub fn addc(&mut self, val: u8) {
        let before = self.a;
        self.a = self.a.wrapping_add(val + self.get_carry());
        let after = self.a;

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            Some(0),
            Some(if Registers::half_carry_occurred(before, after) { 1 } else { 0 }),
            Some(if before > after { 1 } else { 0 })
        );
    }

    pub fn sub(&mut self, val: u8) {
        let before = self.a;
        self.a = self.a.wrapping_sub(val);
        let after = self.a;

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            Some(1),
            Some(if Registers::half_borrow_occurred(before, after) { 1 } else { 0 }),
            Some(if after > before { 1 } else { 0 })
        );
    }

    pub fn subc(&mut self, val: u8) {
        let before = self.a;
        self.a = self.a.wrapping_sub(val + self.get_carry());
        let after = self.a;

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            Some(1),
            Some(if Registers::half_borrow_occurred(before, after) { 1 } else { 0 }),
            Some(if after > before { 1 } else { 0 })
        );
    }

    pub fn and(&mut self, val: u8) {
        self.a &= val;

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            Some(0),
            Some(1),
            Some(0)
        )
    }

    pub fn or(&mut self, val: u8) {
        self.a |= val;

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(0)
        )
    }

    pub fn xor(&mut self, val: u8) {
        self.a ^= val;

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            Some(0),
            Some(0),
            Some(0)
        )
    }

    pub fn comp(&mut self, val: u8) {
        let before = self.a;
        let result = self.a.wrapping_sub(val);

        self.set_flags(
            Some(if result == 0 { 1 } else { 0 }),
            Some(1),
            Some(if Registers::half_carry_occurred(before, result) { 1 } else { 0 }),
            Some(if result > before { 1 } else { 0 })
        );
    }

    pub fn decimal_adjust(&mut self) {
        let mut high_nibble = (0xF0 & self.a) >> 4;
        let mut low_nibble = 0x0F & self.a;
        let mut carry = 0u8;
        let mut correction = 0u8;

        if (self.get_half_carry() == 1) || ((self.get_subtract() == 0) && (low_nibble > 9)) {
            correction |= 0x06;
        }

        if (self.get_carry() == 1) || ((self.get_subtract() == 0) && (high_nibble > 9)) {
            correction |= 0x60;
            carry = 1;
        }

        self.a += if self.get_subtract() == 1 { !correction } else { correction };

        self.set_flags(
            Some(if self.a == 0 { 1 } else { 0 }),
            None,
            Some(0),
            Some(carry)
        );

    }

    /* 16-BIT COMPOUND REGISTERS AND FUNCTIONS */
    // Registers A and F, B and C, D and E, and H and L can be treated as 16-bit registers
    // A, B, D, and H are the high-bytes and F, C, E, and L are their respective low-bytes

    pub fn get_af(&mut self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = ((0xFF00 & val) >> 8) as u8;
        self.f = (0x00FF & val) as u8;
    }

    pub fn add_to_af(&mut self, val: u16) {
//        let mut to_a = (0xFF00 & val) >> 8;
//        let mut to_f = (0x00FF & val);
//
//        let overflow = (self.f as u16) + to_f;
//
//        if overflow > 0x00FF {
//            to_a += (overflow & 0xFF00) >> 8;
//        }
//
//        self.f += to_f as u8;
//        self.a += to_a as u8;

        let before = self.get_af();
        self.set_af(before.wrapping_add(val));
        let after = self.get_af();

        self.set_flags(
            None,
            Some(0),
            Some(if ((before & 0x0F) + (after & 0x0F)) & 0x10 == 0x10 {
                1
            } else {
                0
            }),
            Some(if before > after {
                1
            } else {
                0
            })
        )
    }

    pub fn sub_from_af(&mut self, val: u16) {
        let old = self.get_af();
        self.set_af(old.wrapping_sub(val));
    }

    pub fn get_bc(&mut self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = ((0xFF00 & val) >> 8) as u8;
        self.c = (0x00FF & val) as u8;
    }

    pub fn add_to_bc(&mut self, val: u16) {
//        let mut to_b = (0xFF00 & val) >> 8;
//        let mut to_c = (0x00FF & val);
//
//        let overflow = (self.c as u16) + to_c;
//
//        if overflow > 0x00FF {
//            to_b += (overflow & 0xFF00) >> 8;
//        }
//
//        self.b += to_b as u8;
//        self.c += to_c as u8;

        let before = self.get_bc();
        self.set_bc(before.wrapping_add(val));
        let after = self.get_bc();

        self.set_flags(
            None,
            Some(0),
            Some(if ((before & 0x0F) + (after & 0x0F)) & 0x10 == 0x10 {
                1
            } else {
                0
            }),
            Some(if before > after {
                1
            } else {
                0
            })
        )
    }

    pub fn sub_from_bc(&mut self, val: u16) {
        let old = self.get_bc();
        self.set_bc(old.wrapping_sub(val));
    }

    pub fn get_de(&mut self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = ((0xFF00 & val) >> 8) as u8;
        self.e = (0x00FF & val) as u8;
    }

    pub fn add_to_de(&mut self, val: u16) {
//        let mut to_d = (0xFF00 & val) >> 8;
//        let mut to_e = (0x00FF & val);
//
//        let overflow = (self.e as u16) + to_e;
//
//        if overflow > 0x00FF {
//            to_d += (overflow & 0xFF00) >> 8;
//        }
//
//        self.d += to_d as u8;
//        self.e += to_e as u8;

        let before = self.get_de();
        self.set_de(before.wrapping_add(val));
        let after = self.get_de();

        self.set_flags(
            None,
            Some(0),
            Some(if ((before & 0x0F) + (after & 0x0F)) & 0x10 == 0x10 {
                1
            } else {
                0
            }),
            Some(if before > after {
                1
            } else {
                0
            })
        )
    }

    pub fn sub_from_de(&mut self, val: u16) {
        let old = self.get_de();
        self.set_de(old.wrapping_sub(val));
    }

    pub fn get_hl(&mut self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = ((0xFF00 & val) >> 8) as u8;
        self.l = (0x00FF & val) as u8;
    }

    pub fn add_to_hl(&mut self, val: u16) {
//        let mut to_h = (0xFF00 & val) >> 8;
//        let mut to_l = (0x00FF & val);
//
//        let overflow = (self.l as u16) + to_l;
//
//        if overflow > 0x00FF {
//            to_h += (overflow & 0xFF00) >> 8;
//        }
//
//        self.h += to_h as u8;
//        self.l += to_l as u8;

        let before = self.get_hl();
        self.set_hl(before.wrapping_add(val));
        let after = self.get_hl();

        self.set_flags(
            None,
            Some(0),
            Some(if ((before & 0x0F) + (after & 0x0F)) & 0x10 == 0x10 {
                1
            } else {
                0
            }),
            Some(if before > after {
                1
            } else {
                0
            })
        )
    }

    pub fn sub_from_hl(&mut self, val: u16) {
        let old = self.get_hl();
        self.set_hl(old.wrapping_sub(val));
    }


    /* FLAG FUNCTIONS */

    pub fn set_flags(&mut self, z: Option<u8>, n: Option<u8>, h: Option<u8>, c: Option<u8>) {
        let mut flag = 0u8;
        for f in [z, n, h, c].iter() {
            if let Some(b) = f {
                flag |= b;
            }

            flag <<= 1;
        }

        self.f = flag << 3;
    }

    pub fn get_zero(&mut self) -> u8 {
        (self.f & 0b1000_0000) >> 7
    }

    pub fn get_subtract(&mut self) -> u8 {
        (self.f & 0b0100_0000) >> 6
    }

    pub fn get_half_carry(&mut self) -> u8 {
        (self.f & 0b0010_0000) >> 5
    }

    pub fn half_carry_occurred(b: u8, a: u8) -> bool {
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


        ((b & 0x0F) + (a & 0x0F)) & 0x10 == 0x10
    }

    pub fn half_borrow_occurred(b: u8, a: u8) -> bool {
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

        ((!b & 0x0F) + (a & 0x0F)) & 0x10 == 0x10
    }

    pub fn get_carry(&mut self) -> u8 {
        (self.f & 0b0001_0000) >> 4
    }

    /* DEBUGGING FUNCTIONS */

    pub fn dump_bin(&mut self) {
        println!("\
+--------------+--------------+\n\
|    15 - 8    |     7 - 0    |\n\
+---+----------+---+----------+\n\
| A | {:08b} | F | {:08b} |\n\
+---+----------+---+----------+\n\
| B | {:08b} | C | {:08b} |\n\
+---+----------+---+----------+\n\
| D | {:08b} | E | {:08b} |\n\
+---+----------+---+----------+\n\
| H | {:08b} | L | {:08b} |\n\
+---+----------+---+----------+\n\n\
+----+---------+--------------+\n\
| SP |    {:016b}    |\n\
+----+---------+--------------+\n\
| PC |    {:016b}    |\n\
+----+---------+--------------+",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc
        )
    }

    pub fn dump_hex(&mut self) {
        println!("\
+---------+---------+\n\
| 15 - 8  |  7 - 0  |\n\
+---+-----+---+-----+\n\
| A | ${:02X} | F | ${:02X} |\n\
+---+-----+---+-----+\n\
| B | ${:02X} | C | ${:02X} |\n\
+---+-----+---+-----+\n\
| D | ${:02X} | E | ${:02X} |\n\
+---+-----+---+-----+\n\
| H | ${:02X} | L | ${:02X} |\n\
+---+-----+---+-----+\n\n\
+----+--------------+\n\
| SP |    ${:04X}     |\n\
+----+--------------+\n\
| PC |    ${:04X}     |\n\
+----+--------------+",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc)
    }
}

