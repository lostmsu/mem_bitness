#[macro_use]
extern crate lazy_static;

mod memory;
mod region;
mod rust_mem;

pub use memory::Memory;
pub use rust_mem::RUST_MEMORY;
pub use region::MemoryRegion;

#[cfg(test)]
mod tests;
