use std::heap::AllocErr;

use alloc::Layout;

pub unsafe trait Alloc<PTR: Copy> {
    unsafe fn alloc(&mut self, layout: Layout<PTR>) -> Result<PTR, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: PTR, layout: Layout<PTR>);
}