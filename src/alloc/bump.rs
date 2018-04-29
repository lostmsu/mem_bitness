use std::cmp::PartialOrd;
use std::alloc::AllocErr;
use std::ops::Add;

use alloc::{Alloc, Layout};

pub struct BumpAllocator<PTR: Copy> where
    PTR: PartialOrd,
    PTR: Add<PTR, Output=PTR>,
{
    current: PTR,
    max: PTR,
}

impl<PTR: Copy> BumpAllocator<PTR> where
    PTR: PartialOrd,
    PTR: Add<PTR, Output=PTR>,
{
    pub fn new(beginning: PTR, max: PTR) -> Self {
        BumpAllocator{current: beginning, max}
    }
}

unsafe impl<PTR: Copy> Alloc<PTR> for BumpAllocator<PTR> where
    PTR: PartialOrd,
    PTR: Add<PTR, Output=PTR>,
{
    unsafe fn alloc(&mut self, layout: Layout<PTR>) -> Result<PTR, AllocErr> {
        if self.current + layout.size() > self.max {
            Err(AllocErr {})
        } else {
            let result = self.current;
            self.current = self.current + layout.size();
            Ok(result)
        }
    }

    unsafe fn dealloc(&mut self, _ptr: PTR, _layout: Layout<PTR>) {
        panic!("BumpAllocator can't deallocate")
    }
}