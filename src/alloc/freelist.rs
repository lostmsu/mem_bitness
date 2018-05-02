use std::cmp::PartialOrd;
use std::alloc::AllocErr;
use std::ops::{Add, Sub, BitAnd, Not};

use alloc::{Alloc, Layout};

use typed_ptr::TypedPtr;
use super::super::Memory;

type NodePtr<PTR> = TypedPtr<Node<PTR>,PTR>;
type BlockPtr<PTR> = TypedPtr<Block<PTR>,PTR>;

pub struct FreeList<'a, PTR: Copy + From<usize>, MEM: 'a + Memory<PTR>> where
    PTR: PartialOrd + PartialEq,
    PTR: Add<PTR, Output=PTR>,
    PTR: Sub<PTR, Output=PTR>,
    PTR: BitAnd<PTR, Output=PTR>,
    PTR: Not<Output=PTR>,
{
    start: PTR,
    free: NodePtr<PTR>,
    max: PTR,
    memory: &'a mut MEM,
}

#[derive(Clone)]
struct Node<PTR: Copy + PartialOrd + PartialEq + Clone>{
    max: PTR,
    next: TypedPtr<Node<PTR>, PTR>,
}

impl<PTR: Copy + PartialOrd + PartialEq + Clone + From<usize>> Node<PTR> {
    fn layout() -> Layout<PTR> { unsafe { Layout::new_unchecked::<Node<PTR>>() } }
}

#[derive(Clone)]
struct Block<PTR: Copy + Clone> {
    start: PTR,
    end: PTR,
}

impl <PTR: Copy + Clone + From<usize>> Block<PTR> {
    fn layout() -> Layout<PTR> { unsafe { Layout::new_unchecked::<Block<PTR>>() } }
}

impl<PTR: Copy + Sub<PTR, Output=PTR> + Add<PTR, Output=PTR> + From<usize>> NodePtr<PTR>
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
    PTR: BitAnd<PTR, Output=PTR>,
    PTR: Not<Output=PTR>,
{
    pub unsafe fn new(memory: &'a mut MEM, beginning: PTR, max: PTR) -> Self {
        let free_node_layout = Node::layout();
        if beginning + free_node_layout.size() >= max {
            panic!("memory region is too small")
        };
        let free_node = Node {
            max: max,
            next: FreeList::<'a, PTR, MEM>::invalid_t(beginning, max),
        };
        let head_ptr = NodePtr::new(beginning);
        head_ptr.write(memory, free_node);
        FreeList{start: beginning, max, free: head_ptr, memory}
    }

    fn is_valid(&self, ptr: PTR) -> bool { ptr >= self.start && ptr <= self.max }
    fn is_valid_t<T>(&self, ptr: &TypedPtr<T, PTR>) -> bool { self.is_valid(ptr.address()) }
    fn invalid_t<T>(_: PTR, max: PTR) -> TypedPtr<T, PTR> {
        unsafe { TypedPtr::new(max + 1.into()) }
    }
    fn invalid<T>(&self) -> TypedPtr<T, PTR>{Self::invalid_t::<T>(self.start, self.max)}
    fn minimum_free_block_total_size(&self) -> PTR {
        let node_layout = Node::<PTR>::layout();
        let empty_size = node_layout.align_offset(1.into()) + node_layout.size();
        let allocated_layout = Block::<PTR>::layout();
        let allocated_size = allocated_layout.align_offset(1.into()) + allocated_layout.size();
        if allocated_size > empty_size { allocated_size } else { empty_size }
    }

    fn fits(&self, free_block: NodePtr<PTR>, layout: &Layout<PTR>) -> bool {
        if !self.is_valid_t(&free_block){
            panic!("invalid free block");
        }
        if layout.size() <= 0.into() {
            panic!("invalid layout");
        }

        let block_start = free_block.address();
        let block_end_inclusive = unsafe { free_block.read(self.memory).max };

        let free_node_layout = Node::layout();
        let block_metadata_layout = Block::layout();

        let data_start = layout.align_offset(block_start);
        let metadata_start = block_metadata_layout.align_offset(data_start + layout.size());
        let metadata_end_exclusive = free_node_layout.align_offset(metadata_start + block_metadata_layout.size());
        block_end_inclusive + 1.into() >= metadata_end_exclusive
    }

    fn traverse<F: FnMut(NodePtr<PTR>)>(&self, mut f: F) {
        let mut current = self.free.clone();
        while self.is_valid(current.address()) {
            f(current.clone());
            let next = unsafe { current.read(self.memory).next };
            if next == current {
                panic!("self-loop in traverse")
            }
            current = next;
        }
    }

    fn traverse_while<F: FnMut(NodePtr<PTR>) -> bool>(&self, mut f: F) -> bool {
        let mut current = self.free.clone();
        while self.is_valid_t(&current) {
            if !f(current.clone()){
                return true;
            }
            let next = unsafe { current.read(self.memory).next };
            if next == current {
                panic!("self-loop in traverse")
            }
            current = next;
        }
        return false;
    }

    fn remove(&mut self, node: NodePtr<PTR>, prev: NodePtr<PTR>){
        if !self.is_valid_t(&node){
            panic!("bad node");
        }
        if self.is_valid_t(&prev) {
            let mut prev_node = unsafe { prev.read(self.memory) };
            if prev_node.next != node {
                panic!("bad prev node");
            }
            prev_node.next = unsafe { node.read(self.memory) }.next;
            if prev_node.next == prev {
                panic!("remove self-loop");
            }
        } else {
            if self.free != node {
                panic!("prev is missing, but the node is not the first");
            }
            self.free = unsafe { node.read(self.memory) }.next;
        }
    }

    unsafe fn set_next(&mut self, node_or_invalid: NodePtr<PTR>, next: NodePtr<PTR>){
        if self.is_valid_t(&node_or_invalid){
            let mut node_value = node_or_invalid.read(self.memory);
            node_value.next = next.clone();
            if node_or_invalid == next {
                panic!("set_next self-loop");
            }
            self.write_node(&node_or_invalid, node_value);
        } else {
            self.free = next;
        }
    }

    unsafe fn write_node(&mut self, to: &NodePtr<PTR>, node: Node<PTR>) {
        if node.next == *to {
            panic!("making a loop!")
        }
        if self.is_valid_t(&node.next) & (node.next >= *to) & (node.next.address() <= node.max) {
            panic!("next node can't be within this node!")
        }
        to.write(self.memory, node);
    }
}

unsafe impl<'a, PTR: Copy + From<usize>, MEM: Memory<PTR>> Alloc<PTR>
for FreeList<'a, PTR, MEM> where
    PTR: PartialOrd + PartialEq,
    PTR: Add<PTR, Output=PTR>,
    PTR: Sub<PTR, Output=PTR>,
    PTR: BitAnd<PTR, Output=PTR>,
    PTR: Not<Output=PTR>,
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
            let continue_search = !self.fits(free, &layout);
            continue_search
        }){
            return Err(AllocErr {});
        }

        let free_node_layout = Layout::<PTR>::new_unchecked::<NodePtr<PTR>>();
        let node = target.read(self.memory);

        let block_start = target.address();
        let block_end_exclusive = target.read(self.memory).max + 1.into();
        let block_metadata_layout = Block::layout();

        let data_start = layout.align_offset(block_start);
        let metadata_start = block_metadata_layout.align_offset(data_start + layout.size());
        let metadata_end_exclusive = free_node_layout.align_offset(metadata_start + block_metadata_layout.size());
        if block_end_exclusive < metadata_end_exclusive {
            panic!("internal miscalculation");
        }
        let metadata;

        if metadata_end_exclusive + self.minimum_free_block_total_size() <= block_end_exclusive {
            let new_free_start = NodePtr::new(metadata_end_exclusive);
            self.write_node(&new_free_start, node);
            metadata = Block { start: block_start, end: block_end_exclusive - 1.into() };
            self.set_next(prev, new_free_start);
        } else {
            metadata = Block { start: block_start, end: block_end_exclusive - 1.into() };
            self.remove(target, prev);
        }

        BlockPtr::new(metadata_start).write(self.memory, metadata);

        Ok(data_start)
    }

    unsafe fn dealloc(&mut self, ptr: PTR, layout: Layout<PTR>) {
        if layout.size() <= 0.into() {
            panic!("bad dealloc layout")
        }
        if !self.is_valid(ptr){
            panic!("bad ptr")
        }
        if ptr + layout.size() > self.max + 1.into() {
            panic!("region past max ptr")
        }

        let block_metadata_layout = Block::layout();
        let metadata_start = block_metadata_layout.align_offset(ptr + layout.size());
        let metadata = BlockPtr::new(metadata_start).read(self.memory);

        let mut preceding = self.invalid();
        let mut prev = self.invalid();
        let mut pre_succeeding = self.invalid();
        let mut succeding = self.invalid();
        self.traverse(|free| {
            let node = free.read(self.memory);
            if node.max + 1.into() == metadata.start {
                preceding = free.clone();
                pre_succeeding = prev.clone();
            }
            if metadata.end + 1.into() == free.address() {
                succeding = free.clone();
            }

            prev = free.clone();
        });

        if self.is_valid_t(&succeding) && self.is_valid_t(&preceding) {
            let new_max = succeding.read(self.memory).max;
            self.remove(succeding, pre_succeeding);
            let mut new_preceding = preceding.read(self.memory);
            new_preceding.max = new_max;
            self.write_node(&preceding, new_preceding);
        } else if self.is_valid_t(&succeding) {
            let succeeding_value = succeding.read(self.memory);
            let new_succeeding_ptr = NodePtr::new(metadata.start);
            self.write_node(&new_succeeding_ptr, succeeding_value);
            self.set_next(pre_succeeding, new_succeeding_ptr);
        } else if self.is_valid_t(&preceding){
            let mut preceding_value = preceding.read(self.memory);
            preceding_value.max = metadata.end;
            self.write_node(&preceding, preceding_value);
        } else {
            let region = Node {
                max: metadata.end,
                next: self.free.clone(),
            };
            let region_ptr = NodePtr::new(metadata.start);
            self.write_node(&region_ptr, region);
            self.free = region_ptr;
        }
    }
}