#![feature(allocator_api)]

#[macro_use]
extern crate lazy_static;

pub mod alloc;
mod memory;
mod region;
mod rust_mem;
mod typed_ptr;

pub use self::memory::Memory;
pub use self::rust_mem::RUST_MEMORY;
pub use self::region::MemoryRegion;
pub use self::typed_ptr::TypedPtr;

#[cfg(test)]
mod tests;
