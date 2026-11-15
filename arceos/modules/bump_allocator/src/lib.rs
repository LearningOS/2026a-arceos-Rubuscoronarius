#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::{alloc::Layout, ptr::NonNull};

/// Double-ended allocator: bytes grow upward and pages grow downward.
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    bytes: usize,
    pages: usize,
    allocations: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            bytes: 0,
            pages: 0,
            allocations: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start.checked_add(size).expect("allocator region overflow");
        self.bytes = start;
        self.pages = self.end;
        self.allocations = 0;
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        let end = start.checked_add(size).ok_or(AllocError::InvalidParam)?;
        if start == self.end && self.pages == self.end {
            self.end = end;
            self.pages = end;
            Ok(())
        } else if end == self.start && self.bytes == self.start {
            self.start = start;
            self.bytes = start;
            Ok(())
        } else {
            Err(AllocError::InvalidParam)
        }
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let start = self
            .bytes
            .checked_add(layout.align() - 1)
            .ok_or(AllocError::NoMemory)?
            & !(layout.align() - 1);
        let end = start
            .checked_add(layout.size().max(1))
            .ok_or(AllocError::NoMemory)?;
        if end > self.pages {
            return Err(AllocError::NoMemory);
        }
        let ptr = NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)?;
        self.bytes = end;
        self.allocations += 1;
        Ok(ptr)
    }
    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        assert!(self.allocations > 0);
        self.allocations -= 1;
        if self.allocations == 0 {
            self.bytes = self.start;
        }
    }
    fn total_bytes(&self) -> usize {
        self.end - self.start
    }
    fn used_bytes(&self) -> usize {
        self.bytes - self.start + self.end - self.pages
    }
    fn available_bytes(&self) -> usize {
        self.pages - self.bytes
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if SIZE == 0 || !SIZE.is_power_of_two() || num_pages == 0 || !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let size = num_pages.checked_mul(SIZE).ok_or(AllocError::NoMemory)?;
        let align = align_pow2.max(SIZE);
        let start = self.pages.checked_sub(size).ok_or(AllocError::NoMemory)? & !(align - 1);
        if start < self.bytes {
            return Err(AllocError::NoMemory);
        }
        self.pages = start;
        Ok(start)
    }
    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {}
    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }
    fn used_pages(&self) -> usize {
        (self.end - self.pages) / SIZE
    }
    fn available_pages(&self) -> usize {
        (self.pages - self.bytes) / SIZE
    }
}
