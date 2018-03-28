use memory::Memory;
use region::MemoryRegion;

#[derive(Copy, Clone)]
struct Ref16(u16);

impl From<Ref16> for usize {
    fn from(value: Ref16) -> usize { value.0 as usize }
}

#[test]
fn it_works() {
    let mut region = MemoryRegion::<Ref16>::new(Ref16(1024));
    let ptr = Ref16(32);
    let roundtrip_value: u32 = unsafe {
        region.write(ptr, 42 as u32);
        region.read(ptr)
    };
    assert_eq!(roundtrip_value, 42);
}
