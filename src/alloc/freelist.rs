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

    fn traverse<F: FnMut(TypedPtr<Node<PTR>, PTR>)>(&self, mut f: F) {
        let mut current = self.free.clone();
        while self.is_valid(current.address()) {
            f(current.clone());
            current = unsafe { current.read(self.memory).next }
        }
    }

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

    fn remove(&mut self, node: TypedPtr<Node<PTR>, PTR>, prev: TypedPtr<Node<PTR>, PTR>){
        if !self.is_valid_t(&node){
            panic!("bad node");
        }
        if self.is_valid_t(&prev) {
            let mut prev_node = unsafe { prev.read(self.memory) };
            if prev_node.next != node {
                panic!("bad prev node");
            }
            prev_node.next = unsafe { node.read(self.memory) }.next;
        } else {
            if self.free != node {
                panic!("prev is missing, but the node is not the first");
            }
            self.free = unsafe { node.read(self.memory) }.next;
        }
    }

    unsafe fn set_next(&mut self, node_or_invalid: TypedPtr<Node<PTR>, PTR>, next: TypedPtr<Node<PTR>, PTR>){
        if self.is_valid_t(&node_or_invalid){
            let mut node_value = node_or_invalid.read(self.memory);
            node_value.next = next;
            node_or_invalid.write(self.memory, node_value);
        } else {
            self.free = next;
        }
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
            // this will leak memory, because some of it at the end
            // (< sizeof(Node)) might become untracked
            let result = target.address();
            self.remove(target, prev);
            return Ok(result);
        }
    }

    unsafe fn dealloc(&mut self, ptr: PTR, layout: Layout<PTR>) {
        let free_node_layout = Layout::<PTR>::new_unchecked::<TypedPtr<Node<PTR>, PTR>>();
        if layout.size() < free_node_layout.size() {
            // TODO that's a problem: we can't deallocate anything too small
            panic!("bad dealloc layout")
        }
        if !self.is_valid(ptr){
            panic!("bad ptr")
        }
        if ptr + layout.size() > self.max + 1.into() {
            panic!("region past max ptr")
        }

        let mut preceding = self.invalid();
        let mut prev = self.invalid();
        let mut pre_succeeding = self.invalid();
        let mut succeding = self.invalid();
        self.traverse(|free| {
            let node = free.read(self.memory);
            if node.max + 1.into() == ptr {
                preceding = free.clone();
                pre_succeeding = prev.clone();
            }
            if ptr + layout.size() == free.address() {
                succeding = free.clone();
            }

            prev = free.clone();
        });

        if self.is_valid_t(&succeding) && self.is_valid_t(&preceding) {
            let new_max = succeding.read(self.memory).max;
            self.remove(succeding, pre_succeeding);
            let mut new_preceding = preceding.read(self.memory);
            new_preceding.max = new_max;
            preceding.write(self.memory, new_preceding);
        } else if self.is_valid_t(&succeding) {
            let succeeding_value = succeding.read(self.memory);
            let new_succeeding_ptr = TypedPtr::new(ptr);
            new_succeeding_ptr.write(self.memory, succeeding_value);
            self.set_next(pre_succeeding, new_succeeding_ptr);
        } else if self.is_valid_t(&preceding){
            let mut preceding_value = preceding.read(self.memory);
            preceding_value.max = preceding_value.max + layout.size();
            preceding.write(self.memory, preceding_value);
        } else {
            let region = Node {
                max: ptr + layout.size() - 1.into(),
                next: self.free.clone(),
            };
            let region_ptr = TypedPtr::new(ptr);
            region_ptr.write(self.memory, region);
            self.free = region_ptr;
        }
    }
}