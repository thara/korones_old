#[macro_use]
extern crate bitflags;

mod bus;
mod cpu;
mod data_types;
mod emulator;
mod interrupt;
mod nes;
mod prelude;

pub use emulator::Emulator;
