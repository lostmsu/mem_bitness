use std::alloc;
use std::ops::{Add, Sub, BitAnd, Not};

#[derive(Copy)]
pub struct Layout<PTR: Copy>
{
    size: PTR,
    align: PTR,
}

impl<PTR: Copy> Layout<PTR>
{
    pub unsafe fn from_size_align_unchecked(size: PTR, align: PTR) -> Self {
        Layout {  size,  align }
    }

    pub fn size(&self) -> PTR { self.size }
    pub fn align(&self) -> PTR { self.align }
}

impl <PTR: Copy + From<usize>> Layout<PTR>
where
    PTR: Add<PTR, Output=PTR>,
    PTR: Sub<PTR, Output=PTR>,
    PTR: BitAnd<PTR, Output=PTR>,
    PTR: Not<Output=PTR>,
{
    pub fn align_offset(&self, ptr: PTR) -> PTR {
        (ptr + self.align - 1.into()) & !(self.align - 1.into())
    }
}

impl<PTR: Copy> Clone for Layout<PTR>
{
    fn clone(&self) -> Self {
        Layout{ size: self.size, align: self.align }
    }
}

impl<PTR: Copy + From<usize>> Layout<PTR>
{
    pub unsafe fn new_unchecked<T>() -> Self {
        let layout = alloc::Layout::new::<T>();
        Layout::from_size_align_unchecked(layout.size().into(), layout.align().into())
    }
}

impl<PTR: Into<usize> + Copy> From<Layout<PTR>> for alloc::Layout
where
    PTR: Add<PTR, Output=PTR>,
    PTR: Sub<PTR, Output=PTR>,
{
    fn from(layout: Layout<PTR>) -> alloc::Layout {
        alloc::Layout::from_size_align(layout.size.into(), layout.align.into()).unwrap()
    }
}