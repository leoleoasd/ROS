use core::mem;

use super::*;
use once_cell::race::OnceBox;
use alloc::boxed::Box;
use spin::{Mutex};
use xalloc::{SysTlsf, SysTlsfRegion};
use mem::ManuallyDrop;
use mem::forget;

pub struct Frame {
	ppn: PhysicalPageNumber,
	r: Option<SysTlsfRegion>
}
pub struct FrameRegion {
	ppn: PhysicalPageNumber,
	r: Option<SysTlsfRegion>,
	size: usize
}

impl Drop for Frame {
	fn drop(&mut self) {
		unsafe {
			dealloc(self.r.take().unwrap());
		}
	}
}
impl Drop for FrameRegion {
	fn drop(&mut self) {
		unsafe {
			dealloc(self.r.take().unwrap());
		}
	}
}

// impl FrameRegion {
// 	fn into_single(mut self) -> Frame {
// 		assert!(self.size == 1);
// 		let f = Frame {
// 			r: self.r.take(),
// 			ppn: self.ppn
// 		};
// 		forget(self);
// 		f
// 	}
// }

struct FrameAllocator {
    start: PhysicalPageNumber,
	end: PhysicalPageNumber,
	allocator: SysTlsf<usize>
}

impl FrameAllocator {
	pub fn new(start: PhysicalPageNumber, end: PhysicalPageNumber) -> Self {
		Self {
			start,
			end,
			allocator: SysTlsf::new(end.0 - start.0)
		}
	}
}

impl FrameAllocator {
	fn alloc(&mut self, size: usize) -> Option<(SysTlsfRegion, PhysicalPageNumber)> {
		let (region, offset) = self.allocator.alloc(size)?;
		Some((region, PhysicalPageNumber(self.start.0 + offset)))
	}
	fn dealloc(&mut self, r: SysTlsfRegion) -> Result<(), SysTlsfRegion> {
		self.allocator.dealloc(r)
	}
}

static ALLOCATOR: OnceBox<Mutex<FrameAllocator>> = OnceBox::new();

pub fn alloc_multiple(size: usize) -> Option<FrameRegion> {
	let (region, ppn) = ALLOCATOR.get()?.lock().alloc(size)?;
	Some(FrameRegion { ppn, r: Some(region), size })
}

pub fn alloc() -> Option<Frame> {
	let (region, ppn) = ALLOCATOR.get()?.lock().alloc(1)?;
	Some(Frame { ppn, r: Some(region) })
}

/// Safety: This is unsafe because running it twice will deallocate the same frame twice.
/// `Drop` of the `Frame` is generally recommended.
/// Only use this to dealloc frames **not allocated `alloc`**.
//pub unsafe fn dealloc_multiple(r: SysTlsfRegion, size: usize) -> Option<()> {
//	todo!()
//}

/// Safety: This is unsafe because running it twice will deallocate the same frame twice.
/// `Drop` of the `Frame` is generally recommended.
/// Only use this to dealloc frames **not allocated `alloc`**.
unsafe fn dealloc(r: SysTlsfRegion) -> Option<()> {
	ALLOCATOR.get()?.lock().dealloc(r).ok()
}

pub fn init_allocator(start: PhysicalPageNumber, end: PhysicalPageNumber) {
	let allocator = FrameAllocator::new(start, end);
	let allocator =	Mutex::new(allocator);
	ALLOCATOR.set(Box::new(allocator)).ok();
}