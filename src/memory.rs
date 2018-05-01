pub trait Memory<PTR: Copy> {
    unsafe fn read<T>(&self, ptr: PTR) -> T;
    unsafe fn write<T>(&mut self, ptr: PTR, value: T);
}