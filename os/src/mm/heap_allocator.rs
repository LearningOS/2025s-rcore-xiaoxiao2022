//! The global allocator
use crate::config::KERNEL_HEAP_SIZE;
// use backtrace::Backtrace;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    // let bt = Backtrace::new();
    println!(
        "[kernel] Heap allocation error, layout = {:?}, heap info = {:?}",
        layout,
        HEAP_ALLOCATOR.lock()
    );
    panic!("Heap allocation error, layout = {:?}", layout);
}
/// heap space ([u8; KERNEL_HEAP_SIZE])
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
/// initiate heap allocator
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!(*val, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}


////! The global allocator with logging
// use crate::config::KERNEL_HEAP_SIZE;
// use buddy_system_allocator::LockedHeap;
// use core::alloc::{GlobalAlloc, Layout};
// use core::ptr::{self, NonNull};

// #[global_allocator]
// /// heap allocator instance with logging
// static HEAP_ALLOCATOR: LoggingHeap = LoggingHeap::new();

// #[alloc_error_handler]
// /// panic when heap allocation error occurs
// pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
//     println!(
//         "[kernel] Heap allocation error, layout = {:?}, heap info = {:?}",
//         layout,
//         HEAP_ALLOCATOR.inner.lock()
//     );
//     panic!("Heap allocation error, layout = {:?}", layout);
// }

// /// heap space ([u8; KERNEL_HEAP_SIZE])
// static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

// /// A wrapper around LockedHeap to add logging
// pub struct LoggingHeap {
//     inner: LockedHeap,
// }

// impl LoggingHeap {
//     /// Create a new LoggingHeap
//     pub const fn new() -> Self {
//         Self {
//             inner: LockedHeap::empty(),
//         }
//     }

//     /// Initialize the heap
//     pub fn init(&self) {
//         unsafe {
//             self.inner
//                 .lock()
//                 .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
//         }
//     }
// }

// unsafe impl GlobalAlloc for LoggingHeap {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         let result = self.inner.lock().alloc(layout);
//         match result {
//             Ok(non_null_ptr) => {
//                 println!(
//                     "[kernel] Allocated: ptr = {:p}, size = {}, align = {}",
//                     non_null_ptr.as_ptr(), layout.size(), layout.align()
//                 );
//                 non_null_ptr.as_ptr()
//             }
//             Err(_) => {
//                 println!(
//                     "[kernel] Allocation failed: size = {}, align = {}",
//                     layout.size(), layout.align()
//                 );
//                 ptr::null_mut()
//             }
//         }
//     }

//     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
//         println!(
//             "[kernel] Deallocated: ptr = {:p}, size = {}, align = {}",
//             ptr, layout.size(), layout.align()
//         );
//         self.inner.lock().dealloc(NonNull::new(ptr).expect("Pointer is null"), layout);
//     }
// }

// /// Initiate heap allocator
// pub fn init_heap() {
//     HEAP_ALLOCATOR.init();
// }