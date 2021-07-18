#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate binread;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate anyhow;
extern crate thiserror;

mod apu;
mod bus;
mod controller;
mod cpu;
mod data_types;
mod emulator;
mod interrupt;
mod mapper;
mod nes;
mod ppu;
mod prelude;

pub use emulator::Emulator;
pub use mapper::Cartridge;

pub use controller::StandardController;
