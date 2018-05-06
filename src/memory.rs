use std::marker::PhantomData;

pub trait Memory<PTR> {
    unsafe fn read<T>(&self, ptr: PTR) -> T;
    unsafe fn write<T>(&mut self, ptr: PTR, value: T);
}

pub struct MemoryPlace<T, PTR>{
    address: PTR,
    phantom_data: PhantomData<T>,
}

impl<T, PTR> Place<T> for MemoryPlace<T, PTR> {
    fn pointer(&mut self) -> *mut T {
        // This method is only concerned with returning pointer produced in 
        // make_place
        self.0
    }
}