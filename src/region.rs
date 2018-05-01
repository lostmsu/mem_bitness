use std::convert::Into;
use std::marker::PhantomData;
use std::ptr;

use memory::Memory;

pub struct MemoryRegion<PTR: Into<usize> + Copy> {
    data: Vec<u8>,
    phantom: PhantomData<PTR>,
}

impl<PTR: Into<usize> + Copy> MemoryRegion<PTR> {
    pub fn new(max: PTR) -> MemoryRegion<PTR> {
        MemoryRegion {
            data: vec![0; max.into()],
            phantom: PhantomData,
        }
    }
}

impl<PTR: Into<usize> + Copy> Memory<PTR> for MemoryRegion<PTR> {
    unsafe fn read<T>(&self, ptr: PTR) -> T {
        let read_at = self.data.as_ptr().offset(ptr.into() as isize);
        ptr::read(read_at as *const T)
    }

    unsafe fn write<T>(&mut self, ptr: PTR, value: T) {
        let write_to = self.data.as_mut_ptr().offset(ptr.into() as isize);
        *(write_to as *mut T) = value
    }
}