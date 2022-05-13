use alloc::vec::Vec;
use alloc::vec;
use bitflags::*;
use bitfield::*;

use super::{PhysicalPageNumber, frame_alloctor::{Frame, FrameRegion, alloc}, addresses::VirtualPageNumber};

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Clone, Copy)]
pub struct PageTableEntry {
	pub bits: u64
}

//bitfield! {
//	pub struct PTE(PTEFlags);
//	u64;
//	get_flags, set_flags: 7, 0;
//}

pub struct PageTable {
	root_ppn: PhysicalPageNumber,
	frames: Vec<Frame>,
}

impl PageTableEntry {
	pub fn new(ppn: PhysicalPageNumber, flags: PTEFlags) -> Self {
		PageTableEntry {
			bits: ((ppn.0 as u64) << 10) | flags.bits as u64
		}
	}
	pub fn empty() -> Self {
		PageTableEntry { bits: 0 }
	}

	pub fn ppn(&self) -> PhysicalPageNumber {
		(((self.bits >> 10) & ((1_u64 << 44) - 1)) as usize).into()
	}
	pub fn flags(&self) -> PTEFlags {
		PTEFlags::from_bits(self.bits as u8).unwrap()
	}

	/// helpers
	pub fn is_valid(&self) -> bool {
		(self.flags() & PTEFlags::V) != PTEFlags::empty()
	}
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

impl PageTable {
	pub fn new() -> Self {
		let frame = alloc().unwrap();
		PageTable {
			root_ppn: frame.ppn,
			frames: vec![frame]
		}
	}
	pub fn map(&mut self, vpn: VirtualPageNumber, ppn: PhysicalPageNumber, flags: PTEFlags) {
		todo!()
	}
    pub fn unmap(&mut self, vpn: VirtualPageNumber) {
		todo!()
	}
}
