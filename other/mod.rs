//! export something test function and variable

// memory space
#[path = "./memory/memory.rs"]
mod memory;
pub use memory::test_space::used_impl_frame_allocator;
