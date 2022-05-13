use alloc::vec::Vec;
use alloc::vec;
use bitflags::*;
use bitfield::*;

use super::{PhysicalPageNumber, frame_alloctor::{Frame, FrameRegion, frame_alloc}, addresses::VirtualPageNumber};

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

	pub fn is_dentry(&self) -> bool {
		(self.flags() & PTEFlags::V & (PTEFlags::R | PTEFlags::W | PTEFlags::X))
			!= PTEFlags::empty()
	}
}

impl PageTable {
	pub fn new() -> Self {
		let frame = frame_alloc().unwrap();
		PageTable {
			root_ppn: frame.ppn,
			frames: vec![frame]
		}
	}
	/// Please remember to refresh TLB
	pub fn map(&mut self, vpn: VirtualPageNumber, ppn: PhysicalPageNumber, flags: PTEFlags)
		-> Result<(), VirtualPageNumber> {
		let pte = self.try_find_pte(vpn).ok_or(vpn)?;
        if pte.is_valid() {
			return Err(vpn);		// has been mapped before.
		}
		assert_ne!(flags & (PTEFlags::R | PTEFlags::W | PTEFlags::X), PTEFlags::empty(),
			"invalid flags, should contain at least one of R, W, and X.");
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
		Ok(())
	}
	/// Please remember to refresh TLB
    pub fn unmap(&mut self, vpn: VirtualPageNumber) -> Result<(), VirtualPageNumber> {
		let pte = self.find_pte(vpn).ok_or(vpn)?;
        if !pte.is_valid() {
			return Err(vpn);		// has not been mapped yet.
		}
		*pte = PageTableEntry::empty();
		Ok(())
	}

	/// Find correspond `PageTableEntry` and return.
	/// Returns `None` if not found.
	pub fn find_pte(&self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
		let idxs = vpn.indexes();
		let mut ppn = self.root_ppn;
		//let mut result: Option<&mut PageTableEntry> = None;
		for i in 0..idxs.len() {
			let pte = &mut ppn.get_pte_array()[idxs[i]];
			if !pte.is_valid() {
				return None;
			} else if i == 2 {
				return Some(pte);
			} else if pte.is_dentry() {
				ppn = pte.ppn();
			} else {
				return None;	// should be dentry but is not
			}
		}
		None	// should not be here
	}

	/// Try finding correspond `PageTableEntry` and return.
	/// Recursively creates PDE and PTE if not found.
	/// Note that PTE returned may not be valid.
	fn try_find_pte(&mut self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        //let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..idxs.len() {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                return Some(pte);
            } else if !pte.is_valid() {
                let frame = frame_alloc()?;
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            } else if pte.is_dentry() {
				ppn = pte.ppn();
			} else {
				return None;	// should be dentry but is not
            }
        }
        None	// should not be here
    }
}
