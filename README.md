# mem_bitness
This Rust library enables arbitrary pointer bitness.

For example, you can define a memory region, that uses u16 as pointer type. Then you can read and write memory like this (unsafe):

```rust
let mut backend = MemoryRegion::new(36_u16);
let data: u32 = memory.read(12_u16);
memory.write(24_u16, data);
```

This is just an approximate example. In the current version of the library you'd have to define a custom pointer type, that will implement a set of traits.

# allocators
The library will also provide some simple allocators you can use in your custom regions. See [src/alloc/tests.rs](src/alloc/tests.rs) for usage examples.

There's a `BumpAllocator`, that can't deallocate, and a `FreeList` allocator,
that has very small number of safeguards, and has unpredictable behavior when its `max` is close to max value of pointer type.
E.g. don't set max to 65535 for u16-sized pointers.

Pointers for `FreeList` must implement a number of arithmetic traits, including Add, Sub, Not, and BitAnd among others.
