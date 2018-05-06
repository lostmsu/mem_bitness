use std::heap::AllocErr;

use alloc::Layout;

pub unsafe trait Alloc<PTR: Copy> {
    unsafe fn alloc(&mut self, layout: Layout<PTR>) -> Result<PTR, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: PTR, layout: Layout<PTR>);
}

pub trait Allocated<'a, PTR, T> {

}

pub struct AllocatedPointer<'a, PTR: Copy, T: Sized> {
    originator: Allocator<'a, PTR, T, Pointer=AllocatedPointer<'a, PTR, T>>,
    address: PTR,
}

pub trait Allocator<'a, PTR: Copy, T: Sized> {
    type Pointer: Allocated<'a, PTR, T>;

    fn alloc(&mut self) -> Result<Self::Pointer, AllocErr>;
    fn dealloc(&mut self, pointer: Self::Pointer);
}

impl<'a, T: Sized, PTR: Copy + From<usize>> Allocator<'a, PTR, T> for Alloc<PTR> {
    type Pointer = AllocatedPointer<'a, PTR, T>;
    fn alloc(&mut self) -> Result<Self::Pointer, AllocErr> {
        let layout = Layout::new_unchecked::<T>();
        self.alloc(layout).map(|ptr| -> Self::Pointer{})
    }
}