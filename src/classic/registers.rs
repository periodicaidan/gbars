use bitmatch::bitmatch;
use std::ops::{Add, AddAssign, Sub, SubAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Deref, DerefMut};
use super::utils::{wrapping_inc_16, wrapping_dec_16};

/// The Zilog Z80 has an accumulator (A) and flag (F) register, along with 6 general-purpose
/// registers (B, C, D, E, H, and L). All of these are 8-bit but can double up as AF, BC, DE, and
/// HL to act as 16-bit registers, where A, B, D, and H store the high byte and F, C, E, and L
/// store the low byte. (The way I remember this is to consider HL: H for High, L for Low.) There
/// are of course the two pointer registers SP (for the stack pointer) and PC (for the program
/// counter/instruction pointer).
pub struct Registers {
    pub a: Reg8, // accumulator
    pub f: Reg8, // flags: ZNHC0000
    pub b: Reg8,
    pub c: Reg8,
    pub d: Reg8,
    pub e: Reg8,
    pub h: Reg8,
    pub l: Reg8,
    pub sp: u16, // stack pointer
    pub pc: u16, // program counter
}

impl Registers {
    pub fn init() -> Self {
        Self {
            a: Reg8(0),
            f: Reg8(0),
            b: Reg8(0),
            c: Reg8(0),
            d: Reg8(0),
            e: Reg8(0),
            h: Reg8(0),
            l: Reg8(0),
            sp: 0,
            pc: 0
        }
    }
}

pub trait Register<Size> : DerefMut {
    fn load(&mut self, data: Size);
}

#[derive(Copy, Clone)]
pub struct Reg8(pub u8);
pub struct Reg16(u16);

#[derive(Copy, Clone)]
pub struct Accumulator(u8);

impl Deref for Accumulator {
    type Target = u8;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Accumulator {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target { &mut self.0 }
}

impl Register<u8> for Accumulator {
    fn load(&mut self, data: u8) {
        **self = data;
    }
}

macro_rules! impl_16_bit_funcs {
    ($hi:ident , $lo:ident , $getter:ident , $setter:ident , $inc:ident , $dec:ident) => {
        impl Registers {
            #[bitmatch]
            pub fn $getter(&self) -> u16 {
                let (h, l) = (self.$hi.0, self.$lo.0);
                bitpack!("hhhhhhhh_llllllll") as u16
            }

            #[bitmatch]
            pub fn $setter(&mut self, val: u16) {
                #[bitmatch] let "hhhhhhhh_llllllll" = val;
                self.$hi.0 = h;
                self.$lo.0 = l;
            }

            pub fn $inc(&mut self) {
                let $hi$lo = self.$getter();
                self.$setter(wrapping_inc_16($hi$lo));
            }

            pub fn $dec(&mut self) {
                let $hi$lo = self.$getter();
                self.$setter(wrapping_dec_16($hi$lo));
            }
        }
    };
}

macro_rules! impl_16_bit_reg {
    ($hi:ident , $lo:ident) => {
        impl_16_bit_funcs!{
            $hi, $lo,
            get_$hi$lo,
            set_$hi$lo,
            inc_$hi$lo,
            dec_$hi$lo
        }
    };
}

//impl_16_bit_reg!(b, c);

/// 16-bit registers are implemented as getters and setters
impl Registers {
    #[bitmatch]
    pub fn get_bc(&self) -> u16 {
        let (b, c) = (self.b.0, self.c.0);
        bitpack!("bbbbbbbb_cccccccc") as u16
    }

    #[bitmatch]
    pub fn set_bc(&mut self, val: u16) {
        #[bitmatch] let "bbbbbbbb_cccccccc" = val;
        self.b.0 = b as u8;
        self.c.0 = c as u8;
    }

    fn do_bc(&mut self, f: impl FnOnce(u16) -> u16) {
        let bc = self.get_bc();
        self.set_bc(f(bc));
    }

    pub fn inc_bc(&mut self) { self.do_bc(wrapping_inc_16); }

    pub fn dec_bc(&mut self) { self.do_bc(wrapping_dec_16); }

    #[bitmatch]
    pub fn get_de(&self) -> u16 {
        let (d, e) = (self.d.0, self.c.0);
        bitpack!("dddddddd_eeeeeeee") as u16
    }

    #[bitmatch]
    pub fn set_de(&mut self, val: u16) {
        #[bitmatch] let "dddddddd_eeeeeeee" = val;
        self.d.0 = d as u8;
        self.e.0 = e as u8;
    }

    fn do_de(&mut self, f: impl FnOnce(u16) -> u16) {
        let de = self.get_de();
        self.set_de(f(de));
    }

    pub fn inc_de(&mut self) { self.do_de(wrapping_inc_16); }

    pub fn dec_de(&mut self) { self.do_de(wrapping_dec_16); }

    #[bitmatch]
    pub fn get_hl(&self) -> u16 {
        let (h, l) = (self.h.0, self.l.0);
        bitpack!("hhhhhhhh_llllllll") as u16
    }

    #[bitmatch]
    pub fn set_hl(&mut self, val: u16) {
        #[bitmatch] let "hhhhhhhh_llllllll" = val;
        self.h.0 = h as u8;
        self.l.0 = l as u8;
    }

    fn do_hl(&mut self, f: impl FnOnce(u16) -> u16) {
        let hl = self.get_hl();
        self.set_hl(f(hl));
    }

    pub fn inc_hl(&mut self) { self.do_hl(wrapping_inc_16); }

    pub fn dec_hl(&mut self) { self.do_hl(wrapping_dec_16); }

    pub fn add_hl(&mut self, data: u16) { self.do_hl(|hl| hl.wrapping_add(data)); }

    #[bitmatch]
    pub fn get_af(&self) -> u16 {
        let (a, f) = (self.a.0, self.f.0);
        bitpack!("aaaaaaaa_ffffffff") as u16
    }

    #[bitmatch]
    pub fn set_af(&mut self, val: u16) {
        #[bitmatch] let "aaaaaaaa_ffffffff" = val;
        self.a.0 = a as u8;
        self.f.0 = f as u8;
    }
}

impl Registers {
    pub fn add(&mut self, data: u8) {
        let before = self.a.0;
        self.a += data;
        let after = self.a.0;

        self.set_flags(
            Some(self.a.0 == 0),
            Some(false),
            Some(Self::half_carry_occurred(before, after)),
            Some(before > after)
        );
    }

    pub fn adc(&mut self, data: u8) {
        let before = self.a.0;
        self.a += data + self.carry_bit();
        let after = self.a.0;

        self.set_flags(
            Some(self.a.0 == 0),
            Some(false),
            Some(Self::half_carry_occurred(before, after)),
            Some(before > after)
        );
    }

    pub fn sub(&mut self, data: u8) {
        let before = self.a.0;
        self.a -= data;
        let after = self.a.0;

        self.set_flags(
            Some(self.a.0 == 0),
            Some(true),
            Some(Self::half_borrow_occurred(before, after)),
            Some(before < after)
        );
    }

    pub fn sbc(&mut self, data: u8) {
        let before = self.a.0;
        self.a -= data + self.carry_bit();
        let after = self.a.0;

        self.set_flags(
            Some(self.a.0 == 0),
            Some(true),
            Some(Self::half_borrow_occurred(before, after)),
            Some(before < after)
        );
    }

    pub fn and(&mut self, data: u8) {
        self.a &= data;

        self.set_flags(
            Some(self.a.0 == 0),
            Some(false),
            Some(true),
            Some(false)
        );
    }

    pub fn xor(&mut self, data: u8) {
        self.a ^= data;

        self.set_flags(
            Some(self.a.0 == 0),
            Some(false),
            Some(false),
            Some(false)
        );
    }

    pub fn or(&mut self, data: u8) {
        self.a |= data;

        self.set_flags(
            Some(self.a.0 == 0),
            Some(false),
            Some(false),
            Some(false)
        );
    }

    pub fn cp(&mut self, data: u8) {
        let result = self.a.0 - data;

        self.set_flags(
            Some(result == 0),
            Some(true),
            Some(Self::half_carry_occurred(self.a.0, result)),
            Some(result > self.a.0)
        );
    }

    /// This is a weird one. Decimal-Adjust A retroactively turns the previous arithmetic
    /// instruction into a binary-coded decimal operation. It does this by checking the carry, half-
    /// carry, and neg flags (it is, in fact, the only instruction that checks the latter two).
    ///
    /// Essentially, it checks each nibble of A and if that nibble is greater than 9 (the largest
    /// number that can be represented as a single decimal digit) it adds 6 to that nibble and that
    /// turns it into a single decimal digit. The result is a byte whose high and low nibbles
    /// represent the 10's and 1's place of a decimal number, respectively.
    pub fn daa(&mut self) {
        let mut new_carry = false;
        if self.neg() { // previous instruction was a subtraction
            if self.carry() || self.a.0 > 0x99 {
                self.a += 0x60;
                new_carry = true;
            }

            if self.half_carry() || (self.a.0 & 0x0F) > 0x09 {
                self.a.0 += 0x06;
            }
        } else {
            if self.carry() {
                self.a.0 -= 0x60;
            }

            if self.half_carry() {
                self.a.0 -= 0x06;
            }
        }

        self.set_flags(
            Some(self.a.0 == 0),
            None,
            Some(false),
            Some(new_carry)
        );
    }

    pub fn cpl(&mut self) {
        self.a = !self.a;

        self.set_flags(
            None,
            Some(true),
            Some(true),
            None
        );
    }

    pub fn rlca(&mut self) {
        self.a.rot_left();

        self.set_flags(
            Some(false),
            Some(false),
            Some(false),
            Some(self.a.0 & 1 == 1)
        );
    }

    #[bitmatch]
    pub fn rla(&mut self) {
        #[bitmatch] let "xyyyyyyy" = self.a.0;
        let c = self.carry_bit();
        self.a.0 = bitpack!("yyyyyyyc");

        self.set_flags(
            Some(false),
            Some(false),
            Some(false),
            Some(x == 1)
        );
    }

    pub fn rrca(&mut self) {
        self.a.rot_right();

        self.set_flags(
            Some(false),
            Some(false),
            Some(false),
            Some(self.a.0 & 0x80 == 1)
        )
    }

    #[bitmatch]
    pub fn rra(&mut self) {
        #[bitmatch] let "xxxxxxxy" = self.a.0;
        let c = self.carry_bit();
        self.a.0 = bitpack!("cxxxxxxx");

        self.set_flags(
            Some(false),
            Some(false),
            Some(false),
            Some(y == 1)
        );
    }

    pub fn set_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>) {
        let mut f = 0;
        for flag in [z, n, h, c].iter() {
            if let Some(b) = flag {
                f |= if *b { 1 } else { 0 };
            }

            f <<= 1;
        }

        self.f = Reg8(f << 3);
    }

    #[bitmatch]
    pub fn zero(&self) -> bool {
        #[bitmatch] let "zxxx_xxxx" = self.f.0;
        z == 1
    }

    #[bitmatch]
    pub fn neg(&self) -> bool {
        #[bitmatch] let "xnxx_xxxx" = self.f.0;
        n == 1
    }

    #[bitmatch]
    pub fn half_carry(&self) -> bool {
        #[bitmatch] let "xxhx_xxxx" = self.f.0;
        h == 1
    }

    #[bitmatch]
    pub fn carry_bit(&self) -> u8 {
        #[bitmatch] let "xxxc_xxxx" = self.f.0;
        c
    }

    pub fn carry(&self) -> bool { self.carry_bit() == 1 }

    /// A half-carry is triggered when there's a carry from the 3rd to 4th bit for 8-bit or
    /// from the 11th to 12th for 16-bit. The way to check this is if the sum of the 4 least-
    /// significant bits of the values before and after the computation carries.
    ///
    /// ex:
    ///
    /// 0b0000_1101 (13) + 0b0000_0111 (7) = 0b0001_0100 (20)
    /// (carries from 3rd to 4th bit => half-carry occurs)
    ///
    /// 1101 + 0111 = (1) 0100
    ///                ^
    ///                |
    ///                +------ carry from adding lower nibbles => half-carry occurred
    ///
    /// 0b0101_0110 (86) + 0b0000_1100 (12) = 0b01100010 (98)
    /// (carries from 3rd to 4th bit => half-carry occurs)
    ///
    /// 0110 + 1100 = (1) 0010
    ///                ^
    ///                |
    ///                +------ carry from adding lower nibbles => half-carry occurred
    ///
    /// 0b0001_0000 (16) + 0b0000_0010 (2) = 0b0001_0010 (18)
    /// (no carry from 3rd to 4th bit => half-carry does not occur)
    ///
    /// 0000 + 0010 = (0) 0010
    ///                ^
    ///                |
    ///                +------ no carry from adding lower nibbles => no half-carry occurred
    pub fn half_carry_occurred(b: u8, a: u8) -> bool {
        ((b & 0x0F) + (a & 0x0F)) & 0x10 == 0x10
    }

    /// A half-borrow is the inverse of a half-carry. It's triggered when the 4th bit is borrowed
    /// by the 3rd bit in 8-bit arithmetic or the 12th by the 11th in 16-bit. This can be worked
    /// out by similar logic as with finding a half-carry, but you must flip the bits of the
    /// bottom nibble of the value before subtraction.
    ///
    /// 0b0001_0110 (22) - 0b0000_1010 (10) = 0b0000_1100 (12)
    /// (The 3rd bit borrows from the 4th so a half-borrow occurs)
    ///
    /// ~0110 + 1100 = 1001 + 1100 = (1) 0101
    ///                               ^
    ///                               |
    ///                               +------- carries from adding the nibbles so half-borrow occurred
    ///
    /// 0b0001_1100 (28) - 0b0000_1010 (10) = 0b0001_0010 (18)
    /// (The 3rd bit does not borrow from the 4th and so a half-borrow doesn't occur)
    ///
    /// ~1100 + 1010 = 0011 + 1010 = (0) 1101
    ///                               ^
    ///                               |
    ///                               +------- no carry from adding the nibbles so no half-borrow occurred
    pub fn half_borrow_occurred(b: u8, a: u8) -> bool {
        ((!b & 0x0F) + (a & 0x0F)) & 0x10 == 0x10
    }
}

impl Reg8 {
    pub fn load(&mut self, data: u8) {
        self.0 = data;
    }

    pub fn bit(&self, n: u8) -> u8 {
        let mask = 1 << n;
        (self.0 & mask) >> n
    }

    pub fn reset(&mut self, n: u8) {
        let mask = !(1 << n);
        self.0 &= mask;
    }

    pub fn set(&mut self, n: u8) {
        let mask = 1 << n;
        self.0 |= mask;
    }

    #[bitmatch]
    pub fn swap(&mut self) {
        #[bitmatch] let "xxxx_yyyy" = self.0;
        self.0 = bitpack!("yyyy_xxxx");
    }

    #[bitmatch]
    pub fn rot_left(&mut self) {
        #[bitmatch] let "xyyyyyyy" = self.0;
        self.0 = bitpack!("yyyyyyyx");
    }

    #[bitmatch]
    pub fn rot_right(&mut self) {
        #[bitmatch] let "xxxxxxxy" = self.0;
        self.0 = bitpack!("yxxxxxxx");
    }
}

impl Add for Reg8 {
    type Output = Self;

    fn add(self, rhs: Self) -> <Self as Add>::Output { Reg8(self.0.wrapping_add(rhs.0)) }
}

impl Add<u8> for Reg8 {
    type Output = Self;

    fn add(self, rhs: u8) -> <Self as Add>::Output { Reg8(self.0.wrapping_add(rhs)) }
}

impl AddAssign for Reg8 {
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}

impl AddAssign<u8> for Reg8 {
    fn add_assign(&mut self, rhs: u8) { *self = *self + rhs; }
}

impl Sub for Reg8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output { Reg8(self.0.wrapping_sub(rhs.0)) }
}

impl Sub<u8> for Reg8 {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output { Reg8(self.0.wrapping_sub(rhs)) }
}

impl SubAssign for Reg8 {
    fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; }
}

impl SubAssign<u8> for Reg8 {
    fn sub_assign(&mut self, rhs: u8) { *self = *self - rhs; }
}

impl BitAnd for Reg8 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output { Reg8(self.0 & rhs.0) }
}

impl BitAnd<u8> for Reg8 {
    type Output = Self;

    fn bitand(self, rhs: u8) -> Self::Output { Reg8(self.0 & rhs) }
}

impl BitAndAssign for Reg8 {
    fn bitand_assign(&mut self, rhs: Self) { *self = *self & rhs; }
}

impl BitAndAssign<u8> for Reg8 {
    fn bitand_assign(&mut self, rhs: u8) { *self = *self & rhs; }
}

impl BitOr for Reg8 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output { Reg8(self.0 | rhs.0) }
}

impl BitOr<u8> for Reg8 {
    type Output = Self;

    fn bitor(self, rhs: u8) -> Self::Output { Reg8(self.0 | rhs) }
}

impl BitOrAssign for Reg8 {
    fn bitor_assign(&mut self, rhs: Self) { *self = *self | rhs; }
}

impl BitOrAssign<u8> for Reg8 {
    fn bitor_assign(&mut self, rhs: u8) { *self = *self | rhs; }
}

impl BitXor for Reg8 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output { Reg8(self.0 ^ rhs.0) }
}

impl BitXor<u8> for Reg8 {
    type Output = Self;

    fn bitxor(self, rhs: u8) -> Self::Output { Reg8(self.0 ^ rhs) }
}

impl BitXorAssign for Reg8 {
    fn bitxor_assign(&mut self, rhs: Self) { *self = *self ^ rhs; }
}

impl BitXorAssign<u8> for Reg8 {
    fn bitxor_assign(&mut self, rhs: u8) { *self = *self ^ rhs; }
}

impl Not for Reg8 {
    type Output = Reg8;

    fn not(self) -> Self::Output { Reg8(!self.0) }
}