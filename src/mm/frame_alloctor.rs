use core::mem;

use super::*;
use alloc::boxed::Box;
use mem::forget;
use mem::ManuallyDrop;
use once_cell::race::OnceBox;
use spin::Mutex;
use xalloc::{SysTlsf, SysTlsfRegion};

pub struct Frame {
    pub ppn: PhysicalPageNumber,
    r: Option<SysTlsfRegion>,
}
pub struct FrameRegion {
    pub ppn: PhysicalPageNumber,
    r: Option<SysTlsfRegion>,
    pub size: usize,
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            frame_dealloc(self.r.take().unwrap());
        }
        log!("{:?} dropped", self.ppn);
    }
}
impl Drop for FrameRegion {
    fn drop(&mut self) {
        unsafe {
            frame_dealloc(self.r.take().unwrap());
        }
        log!("dropped");
    }
}

impl From<Frame> for FrameRegion {
    fn from(f: Frame) -> Self {
        let mut x = ManuallyDrop::new(f);
        FrameRegion {
            ppn: x.ppn,
            r: x.r.take(),
            size: 1,
        }
    }
}
impl From<FrameRegion> for Frame {
    fn from(fr: FrameRegion) -> Self {
        assert_eq!(fr.size, 1);
        let mut x = ManuallyDrop::new(fr);
        Frame {
            ppn: x.ppn,
            r: x.r.take(),
        }
    }
}

struct FrameAllocator {
    start: PhysicalPageNumber,
    size: usize,
    allocator: SysTlsf<usize>,
}

impl FrameAllocator {
    pub fn new(start: PhysicalPageNumber, size: usize) -> Self {
        Self {
            start,
            size,
            allocator: SysTlsf::new(size),
        }
    }
}

impl FrameAllocator {
    fn alloc(&mut self, size: usize) -> Option<(SysTlsfRegion, PhysicalPageNumber)> {
        let (region, offset) = self.allocator.alloc(size)?;
        log!("allocated: {:?}", &region);
        Some((region, PhysicalPageNumber(usize::from(self.start) + offset)))
    }
    fn dealloc(&mut self, r: SysTlsfRegion) -> Result<(), SysTlsfRegion> {
        self.allocator.dealloc(r)
    }
}

static ALLOCATOR: OnceBox<Mutex<FrameAllocator>> = OnceBox::new();

pub fn frame_alloc_multiple(size: usize) -> Option<FrameRegion> {
    let (region, ppn) = ALLOCATOR.get()?.lock().alloc(size)?;
    Some(FrameRegion {
        ppn,
        r: Some(region),
        size,
    })
}

pub fn frame_alloc() -> Option<Frame> {
    let (region, ppn) = ALLOCATOR.get()?.lock().alloc(1)?;
    Some(Frame {
        ppn,
        r: Some(region),
    })
}

unsafe fn frame_dealloc(r: SysTlsfRegion) -> Option<()> {
    ALLOCATOR.get()?.lock().dealloc(r).ok()
}

pub fn init_allocator(start: PhysicalPageNumber, size: usize) {
    let allocator = FrameAllocator::new(start, size);
    let allocator = Mutex::new(allocator);
    ALLOCATOR.set(Box::new(allocator)).ok();
}
