#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate binread;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate anyhow;
extern crate thiserror;

mod data_unit;

mod bus;
mod cpu;
mod nes;
pub mod rom;

mod emulator;
