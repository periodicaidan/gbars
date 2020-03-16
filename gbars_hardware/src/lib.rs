#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)]

#[cfg(feature = "alloc")]
#[macro_use] extern crate alloc;

#[macro_use] extern crate bitmatch;
#[macro_use] extern crate lazy_static;

pub mod classic;