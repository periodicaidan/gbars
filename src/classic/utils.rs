pub fn wrapping_inc_8(n: u8) -> u8 { n.wrapping_add(1) }
pub fn wrapping_dec_8(n: u8) -> u8 { n.wrapping_sub(1) }
pub fn wrapping_inc_16(n: u16) -> u16 { n.wrapping_add(1) }
pub fn wrapping_dec_16(n: u16) -> u16 { n.wrapping_sub(1) }

pub fn add_i8_to_u8(n: u8, m: i8) -> u8 {
    if m < 0 {
        n.wrapping_sub(-m as u8)
    } else {
        n.wrapping_add(m as u8)
    }
}

pub fn add_i8_to_u16(n: u16, m: i8) -> u16 {
    if m < 0 {
        n.wrapping_sub(-m as u16)
    } else {
        n.wrapping_add(m as u16)
    }
}