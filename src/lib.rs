#![allow(dead_code)]
use std::{alloc::alloc, alloc::Layout, ptr::NonNull};

pub struct BumpUp<const MIN_ALIGN: usize> {
    start: *mut u8,
    end: *mut u8,
    ptr: *mut u8,
}

#[inline(never)]
pub fn foo(x: &[usize]) -> usize {
    Layout::for_value(x).align()
}

impl<const MIN_ALIGN: usize> BumpUp<MIN_ALIGN> {
    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::from_size_align(cap, 8).unwrap();
        let ptr = unsafe { alloc(layout) };
        Self {
            start: ptr,
            end: ptr.wrapping_add(cap),
            ptr,
        }
    }

    pub fn clear(&mut self) {
        self.ptr = self.start;
    }

    #[inline(always)]
    const fn align_offset(size: usize, align: usize) -> usize {
        align.wrapping_sub(size) & (align - 1)
    }

    #[inline]
    pub fn alloc_orig(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();
        debug_assert!(align > 0);
        debug_assert!(align.is_power_of_two());

        let ptr = self.ptr as usize;

        // Round the bump pointer up to the requested
        // alignment.
        let aligned = (ptr.checked_add(align - 1))? & !(align - 1);
        let new_ptr = aligned.checked_add(size)?;

        let end = self.end as usize;
        if new_ptr > end {
            // Didn't have enough capacity!
            return None;
        }

        self.ptr = new_ptr as *mut u8;
        unsafe { Some(NonNull::new_unchecked(aligned as *mut u8)) }
    }

    #[inline]
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let ptr = self.ptr;
        let align = layout.align();
        let align_offset = if align > MIN_ALIGN {
            Self::align_offset(ptr as usize, align)
        } else {
            0
        };

        let size = layout.size() + align_offset;
        let available = self.end as usize - ptr as usize ;
        if available >= size {
            let end_offset = Self::align_offset(layout.size(), MIN_ALIGN);
            let aligned_size = size + end_offset;
            let result = ptr.wrapping_add(align_offset);
            self.ptr = ptr.wrapping_add(aligned_size);
            unsafe { Some(NonNull::new_unchecked(result)) }
        } else {
            None
        }
    }
}

pub struct BumpDown<const MIN_ALIGN: usize> {
    start: *mut u8,
    ptr: *mut u8,
    end: *mut u8,
}

impl<const MIN_ALIGN: usize> BumpDown<MIN_ALIGN> {
    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::from_size_align(cap, 8).unwrap();
        let ptr = unsafe { alloc(layout) };
        let end = ptr.wrapping_add(cap);
        Self {
            start: ptr,
            end,
            ptr: end,
        }
    }

    pub fn clear(&mut self) {
        self.ptr = self.end;
    }

    #[inline]
    fn reserve_space_for(layout: Layout, ptr: *mut u8) -> *mut u8 {
        let size = (layout.size() + MIN_ALIGN - 1) & !(MIN_ALIGN - 1);
        let ptr = ptr.wrapping_sub(size);
        if layout.align() > MIN_ALIGN {
            round_mut_ptr_down_to(ptr, layout.align())
        } else {
            ptr
        }
    }

    #[inline]
    pub fn alloc_orig(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let ptr = self.ptr;
        let start = self.start;
        if (ptr as usize) < layout.size() {
            return None;
        }

        let ptr = ptr.wrapping_sub(layout.size());
        let aligned_ptr = round_mut_ptr_down_to(ptr, layout.align());

        if aligned_ptr >= start {
            self.ptr = aligned_ptr;
            let aligned_ptr = unsafe { NonNull::new_unchecked(aligned_ptr) };
            Some(aligned_ptr)
        } else {
            None
        }
    }

    #[inline]
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let ptr = self.ptr;
        let start = self.start;
        if (ptr as usize) < layout.size() {
            return None;
        }

        let aligned_ptr = Self::reserve_space_for(layout, ptr);

        if aligned_ptr >= start {
            self.ptr = aligned_ptr;
            let aligned_ptr = unsafe { NonNull::new_unchecked(aligned_ptr) };
            Some(aligned_ptr)
        } else {
            None
        }
    }
}

#[inline]
pub(crate) fn round_mut_ptr_down_to(ptr: *mut u8, divisor: usize) -> *mut u8 {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    ptr.wrapping_sub(ptr as usize & (divisor - 1))
}

#[no_mangle]
fn align_std(ptr: *mut u8) -> usize {
    ptr.align_offset(8)
}

#[no_mangle]
fn align_manual(ptr: *mut u8) -> usize {
    8usize.wrapping_sub(ptr as usize) & (8 - 1)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bump_up() {
        let mut bump = BumpUp::<8>::with_capacity(100);
        let layout = Layout::new::<u8>();
        let ptr = bump.alloc(layout).unwrap();
        assert_eq!(ptr.as_ptr() as usize % 8, 0);
    }
}
