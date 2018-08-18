pub fn reset() {
    self::optional_impl::reset()
}

pub fn alloc_count() -> usize {
    self::optional_impl::alloc_count()
}

pub fn dealloc_count() -> usize {
    self::optional_impl::dealloc_count()
}

#[cfg(feature = "alloc_debug")]
mod optional_impl {
    static ALLOC_COUNT: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);
    static DEALLOC_COUNT: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);

    pub fn reset() {
        ALLOC_COUNT.store(0, ::std::sync::atomic::Ordering::SeqCst);
        DEALLOC_COUNT.store(0, ::std::sync::atomic::Ordering::SeqCst);
    }

    pub fn alloc_count() -> usize {
        ALLOC_COUNT.load(::std::sync::atomic::Ordering::SeqCst)
    }

    pub (super) fn alloc_inc() {
        ALLOC_COUNT.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
    }

    pub fn dealloc_count() -> usize {
        DEALLOC_COUNT.load(::std::sync::atomic::Ordering::SeqCst)
    }

    pub (super) fn dealloc_inc() {
        DEALLOC_COUNT.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(not(feature = "alloc_debug"))]
mod optional_impl {
    pub fn reset() {
    }

    pub fn alloc_count() -> usize {
        0
    }

    fn alloc_inc() {
    }

    pub fn dealloc_count() -> usize {
        0
    }

    fn dealloc_inc() {
    }
}

use std::alloc::{GlobalAlloc, System, Layout};

pub struct ProfilingAlloc;

unsafe impl GlobalAlloc for ProfilingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        optional_impl::alloc_inc();
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        optional_impl::dealloc_inc();
        System.dealloc(ptr, layout)
    }
}