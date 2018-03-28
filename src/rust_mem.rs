use std::ptr;
use std::sync::Mutex;

use memory::Memory;

lazy_static! {
  pub static ref RUST_MEMORY: Mutex<RustMemory> = Mutex::new(RustMemory{});
}

pub struct RustMemory();

impl Memory<*mut u8> for RustMemory {
    unsafe fn read<T>(&self, ptr: *mut u8) -> T {
        ptr::read(ptr as *mut T)
    }

    unsafe fn write<T>(&mut self, ptr: *mut u8, value: T) {
         *(ptr as *mut T) = value
    }
}