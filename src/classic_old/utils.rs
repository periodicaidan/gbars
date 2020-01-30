use super::gb_types::Console;
use std::ops::{Div, Rem};

pub fn read_byte(gb: &Console, address: u16) -> u8 {
    if let Some(rom) = &gb.rom {
        *rom.contents.get(address as usize).unwrap()
    } else {
        0
    }
}

pub fn write_byte(gb: &mut Console, address: u16, data: u8) {
    if let Some(rom) = &mut gb.rom {
        rom.contents[address as usize] = data;
    }
}

pub fn vec_slice<T: Clone>(v: &Vec<T>, start: usize, end: usize) -> Vec<T> {
    let mut out_vec = Vec::with_capacity(end - start);
    out_vec.extend_from_slice(&v[start..end]);

    out_vec
}

pub fn make_vec(capacity: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(capacity);
    v.extend([0].iter().cycle().take(capacity));

    v
}