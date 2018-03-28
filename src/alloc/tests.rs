use std::cmp::Ordering;
use std::ops::Add;

use super::alloc::Alloc;
use super::bump::BumpAllocator;
use super::layout::Layout;

#[derive(Copy, Clone, Debug, Eq)]
struct Ref16(u16);

impl From<Ref16> for usize {
    fn from(value: Ref16) -> usize { value.0 as usize }
}

impl PartialEq for Ref16 {
    fn eq(&self, other: &Ref16) -> bool { self.0 == other.0 }
}

impl PartialOrd for Ref16 {
    fn partial_cmp(&self, other: &Ref16) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Add for Ref16 {
    type Output = Ref16;
    fn add(self, other: Ref16) -> Self { Ref16(self.0 + other.0) }
}

impl From<usize> for Ref16 {
    fn from(value: usize)-> Self { Ref16(value as u16) }
}



#[test]
fn gives_two_different_pointers() {
    let mut allocator = BumpAllocator::new(Ref16(0), Ref16(4));
    unsafe {
        let u16_layout = Layout::<Ref16>::new_unchecked::<u16>();
        let ptr1 = allocator.alloc(u16_layout.clone());
        let ptr2 = allocator.alloc(u16_layout.clone());
        assert_ne!(ptr1, ptr2);
    }
}

#[test]
fn panics_on_exhaustion() {
    let mut allocator = BumpAllocator::new(Ref16(0), Ref16(4));
    unsafe {
        let u16_layout = Layout::<Ref16>::new_unchecked::<u16>();
        let _ptr1 = allocator.alloc(u16_layout.clone());
        let _ptr2 = allocator.alloc(u16_layout.clone());
        let ptr3 = allocator.alloc(u16_layout.clone());
        assert!(ptr3.is_err());
    }
}
