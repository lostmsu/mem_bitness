use std::cmp::Ordering;
use std::marker::PhantomData;
use memory::Memory;

#[derive(Copy, Clone, Debug)]
pub struct TypedPtr<T, PTR>{
    ptr: PTR,
    phantom: PhantomData<T>,
}

impl<T, PTR: Copy> TypedPtr<T, PTR> {
    pub unsafe fn new(address: PTR) -> Self { TypedPtr { ptr: address, phantom: PhantomData } }

    pub fn address(&self) -> PTR { self.ptr }

    pub unsafe fn read<MEM: Memory<PTR>>(&self, mem: &MEM) -> T { mem.read(self.ptr) }
    pub unsafe fn write<MEM: Memory<PTR>>(&self, mem: &mut MEM, value: T) { mem.write(self.ptr, value) }
}

impl<T, PTR: PartialEq> PartialEq for TypedPtr<T, PTR> {
    fn eq(&self, other: &Self) -> bool { self.ptr == other.ptr }
}

impl<T, PTR: PartialOrd> PartialOrd for TypedPtr<T, PTR> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.ptr.partial_cmp(&other.ptr) }
}
