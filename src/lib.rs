#![no_std]
#![feature(panic_info_message, global_asm)]
#![feature(asm, allocator_api, alloc_error_handler, const_raw_ptr_to_usize_cast)]

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////

// ///////////////////////////////////
// / RUST MODULES
// ///////////////////////////////////
pub mod assembly;
pub mod kmem;
pub mod page;
pub mod uart;
