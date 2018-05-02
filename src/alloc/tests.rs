use std::cmp::Ordering;
use std::ops::{Add, Sub, BitAnd, Not};

use super::alloc::Alloc;
use super::bump::BumpAllocator;
use super::freelist::FreeList;
use super::layout::Layout;
use super::super::MemoryRegion;

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

impl Sub for Ref16 {
    type Output = Ref16;
    fn sub(self, other: Ref16) -> Self { Ref16(self.0 - other.0) }
}

impl BitAnd for Ref16 {
    type Output = Ref16;
    fn bitand(self, other: Ref16) -> Self { Ref16(self.0 & other.0) }
}

impl Not for Ref16 {
    type Output = Ref16;
    fn not(self) -> Self { Ref16(!self.0) }
}


impl From<usize> for Ref16 {
    fn from(value: usize)-> Self { Ref16(value as u16) }
}

struct UnevenObject{
    _byte: u8,
    _word: u16,
}

fn ensure_can_alloc<A: Alloc<PTR>, PTR: Copy>(allocator: &mut A, layout: Layout<PTR>) {
    unsafe {
        let ptr = allocator.alloc(layout.clone()).unwrap();
        allocator.dealloc(ptr, layout);
    }
}

unsafe fn fill<A: Alloc<PTR>, PTR: Copy>(allocator: &mut A, allocated: &mut Vec<PTR>, layout: Layout<PTR>) {
    loop {
        match allocator.alloc(layout.clone()) {
            Result::Ok(ptr) => allocated.push(ptr),
            Result::Err(_) => break,
        }
        assert!((allocated.len() < 70000) & (allocated.len() > 0));
    }

    assert_ne!(0, allocated.len());
    println!("allocated: {}", allocated.len());
}

unsafe fn allocator_sanity_test<A: Alloc<PTR>, PTR: Copy + From<usize>>(allocator: &mut A) {
    let layout = Layout::new_unchecked::<UnevenObject>();
    let mut allocated: Vec<PTR> = Vec::new();

    fill(allocator, &mut allocated, layout);

    for i in 0 .. allocated.len() - 1 {
        if i & 1 == 0 {
            allocator.dealloc(allocated[i], layout.clone());
        }
    }

    ensure_can_alloc(allocator, layout.clone());

    for i in 0 .. allocated.len() - 1{
        if i & 1 != 0 {
            allocator.dealloc(allocated[i], layout.clone());
        }
    }

    allocated.clear();

    ensure_can_alloc(allocator, layout.clone());

    fill(allocator, &mut allocated, layout.clone());

    for p in allocated.iter() {
        allocator.dealloc(*p, layout.clone());
    }
    
    allocated.clear();

    ensure_can_alloc(allocator, layout.clone());

    fill(allocator, &mut allocated, layout.clone());

    for p in allocated.iter().rev() {
        allocator.dealloc(*p, layout.clone());
    }

    allocated.clear();

    ensure_can_alloc(allocator, layout.clone());
}


#[test]
fn freelist_is_sane(){
    let mut backend = MemoryRegion::new(Ref16(36));
    unsafe {
        let mut allocator = FreeList::new(&mut backend, Ref16(0), Ref16(35));
        allocator_sanity_test(&mut allocator);
    }
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
