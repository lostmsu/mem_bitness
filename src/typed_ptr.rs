use std::marker::PhantomData;
use memory::Memory;

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct TypedPtr<T, PTR: Copy + PartialOrd + Clone + PartialEq>{
    ptr: PTR,
    phantom: PhantomData<T>,
}

impl<T, PTR: Copy + PartialOrd + Clone + PartialEq> TypedPtr<T, PTR> {
    pub unsafe fn new(address: PTR) -> Self { TypedPtr { ptr: address, phantom: PhantomData } }

    pub fn address(&self) -> PTR { self.ptr }

    pub unsafe fn read<MEM: Memory<PTR>>(&self, mem: &MEM) -> T { mem.read(self.ptr) }
    pub unsafe fn write<MEM: Memory<PTR>>(&self, mem: &mut MEM, value: T) { mem.write(self.ptr, value) }
}
