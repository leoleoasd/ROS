use core::fmt::{self, Debug, Formatter};
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

use super::page_table::PageTableEntry;

const PA_WIDTH_SV39: usize = 56;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddress(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddress(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalPageNumber(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualPageNumber(pub usize);

/// Debug

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtualPageNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysicalPageNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

/// Convert

impl From<usize> for PhysicalAddress {
    fn from(n: usize) -> Self { Self(n & ( (1 << PA_WIDTH_SV39) - 1 )) }
}
impl From<PhysicalAddress> for usize {
    fn from(pa: PhysicalAddress) -> Self { pa.0 }
}

impl PhysicalPageNumber {
    pub unsafe fn from(n: usize) -> Self { Self(n & ( (1 << PPN_WIDTH_SV39) - 1 )) }
}
impl From<PhysicalPageNumber> for usize {
    fn from(ppn: PhysicalPageNumber) -> Self { ppn.0 }
}

impl VirtualAddress {
    pub unsafe fn from(n: usize) -> Self { Self(n & ( (1 << PA_WIDTH_SV39) - 1 )) }
}
impl From<VirtualAddress> for usize {
    fn from(va: VirtualAddress) -> Self { va.0 }
}

impl VirtualPageNumber {
    pub unsafe fn from(n: usize) -> Self { Self(n & ( (1 << PPN_WIDTH_SV39) - 1 )) }
}
impl From<VirtualPageNumber> for usize {
    fn from(vpn: VirtualPageNumber) -> Self { vpn.0 }
}

impl PhysicalAddress {
	pub fn floor(&self) -> PhysicalPageNumber { PhysicalPageNumber(self.0 / PAGE_SIZE) }
	pub fn ceil(&self) -> PhysicalPageNumber { PhysicalPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
	pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
}

impl From<PhysicalPageNumber> for PhysicalAddress {
    fn from(v: PhysicalPageNumber) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<PhysicalAddress> for PhysicalPageNumber {
	fn from(pa: PhysicalAddress) -> Self {
		assert_eq!(pa.page_offset(), 0);
		pa.floor()
	}
}

impl PhysicalPageNumber {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa = PhysicalAddress::from(self.clone());
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)
        }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa = PhysicalAddress::from(self.clone());
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa = PhysicalAddress::from(self.clone());
        unsafe {
            (pa.0 as *mut T).as_mut().unwrap()
        }
    }
}

impl VirtualPageNumber {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}
