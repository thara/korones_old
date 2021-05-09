#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate binread;

mod bus;
mod cpu;
mod data_types;
mod emulator;
mod interrupt;
mod mapper;
mod nes;
mod ppu;
mod prelude;

pub use emulator::Emulator;
