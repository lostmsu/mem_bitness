use std::cmp::PartialOrd;
use std::alloc::AllocErr;
use std::ops::{Add, Sub};

use alloc::{Alloc, Layout};

use typed_ptr::TypedPtr;
use super::super::Memory;

pub struct FreeList<'a, PTR: Copy + From<usize>, MEM: 'a + Memory<PTR>> where
    PTR: PartialOrd + PartialEq,
    PTR: Add<PTR, Output=PTR>,
    PTR: Sub<PTR, Output=PTR>,
{
    start: PTR,
    free: TypedPtr<Node<PTR>, PTR>,
    max: PTR,
    memory: &'a mut MEM,
}

#[derive(Clone)]
struct Node<PTR: Copy + PartialOrd + PartialEq + Clone>{
    max: PTR,
    next: TypedPtr<Node<PTR>, PTR>,
}

impl<PTR: Copy + Sub<PTR, Output=PTR> + Add<PTR, Output=PTR> + From<usize>> TypedPtr<Node<PTR>, PTR>
where PTR: PartialOrd + PartialEq {
    pub unsafe fn size<MEM: Memory<PTR>>(&self, memory: &MEM) -> PTR {
        let max = self.read(memory).max;
        max - self.address() + 1.into()
    }
}

impl<'a, PTR: Copy + From<usize>, MEM: Memory<PTR>> FreeList<'a, PTR, MEM> where
    PTR: PartialOrd + PartialEq,
    PTR: Add<PTR, Output=PTR>,
    PTR: Sub<PTR, Output=PTR>,
{
    pub unsafe fn new(memory: &'a mut MEM, beginning: PTR, max: PTR) -> Self {
        let free_node_layout = Layout::new_unchecked::<TypedPtr<Node<PTR>, PTR>>();
        if beginning + free_node_layout.size() <= max {
            panic!("memory region is too small")
        };
        let free_node = Node {
            max: max,
            next: FreeList::<'a, PTR, MEM>::invalid_t(beginning, max),
        };
        let head_ptr = TypedPtr::new(beginning);
        head_ptr.write(memory, free_node);
        FreeList{start: beginning, max, free: head_ptr, memory}
    }

    fn is_valid(&self, ptr: PTR) -> bool { ptr >= self.start && ptr <= self.max }
    fn is_valid_t<T>(&self, ptr: &TypedPtr<T, PTR>) -> bool { self.is_valid(ptr.address()) }
    fn invalid_t<T>(_: PTR, max: PTR) -> TypedPtr<T, PTR> {
        unsafe { TypedPtr::new(max + 1.into()) }
    }
    fn invalid<T>(&self) -> TypedPtr<T, PTR>{Self::invalid_t::<T>(self.start, self.max)}

    fn traverse_while<F: FnMut(TypedPtr<Node<PTR>, PTR>) -> bool>(&self, mut f: F) -> bool {
        let mut current = self.free.clone();
        while self.is_valid_t(&current) {
            if !f(current.clone()){
                return true;
            }
            current = unsafe { current.read(self.memory).next }
        }
        return false;
    }
}

unsafe impl<'a, PTR: Copy + From<usize>, MEM: Memory<PTR>> Alloc<PTR>
for FreeList<'a, PTR, MEM> where
    PTR: PartialOrd + PartialEq,
    PTR: Add<PTR, Output=PTR>,
    PTR: Sub<PTR, Output=PTR>,
{
    unsafe fn alloc(&mut self, layout: Layout<PTR>) -> Result<PTR, AllocErr> {
        if layout.size() == 0.into() {
            return Err(AllocErr {})
        }

        let mut prev = self.invalid();
        let mut target = self.invalid();
        if !self.traverse_while(|free| {
            prev = target.clone();
            target = free.clone();
            return free.size(self.memory) < layout.size();
        }){
            return Err(AllocErr {});
        }

        let free_node_layout = Layout::<PTR>::new_unchecked::<TypedPtr<Node<PTR>, PTR>>();
        let mut node = target.read(self.memory);
        let size = target.size(self.memory);
        if free_node_layout.size() + layout.size() <= size {
            node.max = node.max - layout.size();
            let result = node.max + 1.into();
            target.write(self.memory, node);
            return Ok(result)
        } else {
            let result = target.address();
            let next = target.read(self.memory).next;
            if self.is_valid_t(&prev) {
                let mut prev_mut = prev.read(self.memory);
                prev_mut.next = next;
                prev.write(self.memory, prev_mut);
            } else{
                self.free = next;
            }
            return Ok(result);
        }
    }

    unsafe fn dealloc(&mut self, _ptr: PTR, _layout: Layout<PTR>) {
        panic!("Not implemented")
    }
}