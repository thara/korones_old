#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate binread;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate anyhow;
extern crate thiserror;

pub mod emulator;
pub mod nes;
pub mod rom;

#[macro_use]
mod bus;
mod cpu;
mod data_unit;
mod interrupt;
mod ppu;
mod scanline;
