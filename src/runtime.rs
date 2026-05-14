//! Syl Native Runtime — Arena Allocator
//!
//! This module provides the memory arena that compiled Syl programs link against.
//! All functions are `#[no_mangle] extern "C"` so Cranelift can resolve them at link time.

use std::alloc::{alloc, Layout};

static mut ARENA_PTR: *mut u8 = std::ptr::null_mut();
static mut ARENA_OFFSET: usize = 0;
const ARENA_CAPACITY: usize = 10 * 1024 * 1024; // 10MB default arena

/// Initializes the global arena. Must be called once before any `syl_alloc`.
#[no_mangle]
pub extern "C" fn syl_arena_init() {
    unsafe {
        let layout = Layout::from_size_align(ARENA_CAPACITY, 8).unwrap();
        ARENA_PTR = alloc(layout);
        if ARENA_PTR.is_null() {
            eprintln!("[ HELIX FATAL ] Failed to allocate native arena.");
            std::process::exit(1);
        }
        ARENA_OFFSET = 0;
    }
}

/// Bump-allocates `size` bytes from the arena. Panics if the arena is exhausted.
#[no_mangle]
pub extern "C" fn syl_alloc(size: usize) -> *mut u8 {
    unsafe {
        if ARENA_OFFSET + size > ARENA_CAPACITY {
            eprintln!("[ HELIX FATAL ] Native Arena Out of Memory ({} bytes requested, {} / {} used).",
                size, ARENA_OFFSET, ARENA_CAPACITY);
            std::process::exit(1);
        }
        let ptr = ARENA_PTR.add(ARENA_OFFSET);
        ARENA_OFFSET += size;
        ptr
    }
}

/// Duplicates a C string into the arena.
#[no_mangle]
pub extern "C" fn syl_strdup(src: *const u8) -> *mut u8 {
    unsafe {
        let len = libc_strlen(src);
        let dst = syl_alloc(len + 1);
        std::ptr::copy_nonoverlapping(src, dst, len + 1);
        dst
    }
}

/// Saves the current arena offset (for scoped memory).
#[no_mangle]
pub extern "C" fn syl_arena_save() -> usize {
    unsafe { ARENA_OFFSET }
}

/// Restores the arena offset, effectively freeing everything allocated after the save point.
#[no_mangle]
pub extern "C" fn syl_arena_restore(saved_offset: usize) {
    unsafe { ARENA_OFFSET = saved_offset; }
}

/// Internal strlen since we can't depend on libc in all targets.
unsafe fn libc_strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}
