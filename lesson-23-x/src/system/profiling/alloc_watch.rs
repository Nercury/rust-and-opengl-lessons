use once_cell::sync::OnceCell;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic;

pub struct PeekAlloc;

#[derive(Debug)]
pub struct PeekedValues {
    pub alloc_num: usize,
    pub alloc_bytes: usize,
    pub dealloc_num: usize,
    pub dealloc_bytes: usize,
}

#[derive(Debug)]
struct PeekableState {
    alloc_num: atomic::AtomicUsize,
    alloc_bytes: atomic::AtomicUsize,
    dealloc_num: atomic::AtomicUsize,
    dealloc_bytes: atomic::AtomicUsize,
}

impl PeekableState {
    pub fn new() -> PeekableState {
        PeekableState {
            alloc_num: atomic::AtomicUsize::new(0),
            alloc_bytes: atomic::AtomicUsize::new(0),
            dealloc_num: atomic::AtomicUsize::new(0),
            dealloc_bytes: atomic::AtomicUsize::new(0),
        }
    }
}

static INSTANCE: OnceCell<PeekableState> = OnceCell::INIT;

impl PeekAlloc {
    pub fn init() {
        INSTANCE.set(PeekableState::new()).unwrap();
    }

    pub fn reset() {
        if let Some(ref instance) = INSTANCE.get() {
            instance.alloc_num.store(0, atomic::Ordering::SeqCst);
            instance.alloc_bytes.store(0, atomic::Ordering::SeqCst);
            instance.dealloc_num.store(0, atomic::Ordering::SeqCst);
            instance.dealloc_bytes.store(0, atomic::Ordering::SeqCst);
        }
    }

    pub fn peek() -> Option<PeekedValues> {
        if let Some(ref instance) = INSTANCE.get() {
            return Some(PeekedValues {
                alloc_num: instance.alloc_num.load(atomic::Ordering::SeqCst),
                alloc_bytes: instance.alloc_bytes.load(atomic::Ordering::SeqCst),
                dealloc_num: instance.dealloc_num.load(atomic::Ordering::SeqCst),
                dealloc_bytes: instance.dealloc_bytes.load(atomic::Ordering::SeqCst),
            });
        }
        None
    }
}

unsafe impl GlobalAlloc for PeekAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(ref instance) = INSTANCE.get() {
            instance.alloc_num.fetch_add(1, atomic::Ordering::SeqCst);
            instance
                .alloc_bytes
                .fetch_add(layout.size(), atomic::Ordering::SeqCst);
        }

        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ref instance) = INSTANCE.get() {
            instance.dealloc_num.fetch_add(1, atomic::Ordering::SeqCst);
            instance
                .dealloc_bytes
                .fetch_add(layout.size(), atomic::Ordering::SeqCst);
        }

        System.dealloc(ptr, layout)
    }
}
