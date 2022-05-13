use core::fmt::{self, Debug, Formatter};
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

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

impl From<usize> for PhysicalPageNumber {
    fn from(n: usize) -> Self { Self(n & ( (1 << PPN_WIDTH_SV39) - 1 )) }
}
impl From<PhysicalPageNumber> for usize {
    fn from(ppn: PhysicalPageNumber) -> Self { ppn.0 }
}

impl From<usize> for VirtualAddress {
    fn from(n: usize) -> Self { Self(n & ( (1 << PA_WIDTH_SV39) - 1 )) }
}
impl From<VirtualAddress> for usize {
    fn from(va: VirtualAddress) -> Self { va.0 }
}

impl From<usize> for VirtualPageNumber {
    fn from(n: usize) -> Self { Self(n & ( (1 << PPN_WIDTH_SV39) - 1 )) }
}
impl From<VirtualPageNumber> for usize {
    fn from(vpn: VirtualPageNumber) -> Self { vpn.0 }
}

impl PhysicalAddress {
	pub fn floor(&self) -> PhysicalPageNumber { PhysicalPageNumber(self.0 / PAGE_SIZE) }
	pub fn ceil(&self) -> PhysicalPageNumber { PhysicalPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
	pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
}

impl From<PhysicalAddress> for PhysicalPageNumber {
	fn from(pa: PhysicalAddress) -> Self {
		assert_eq!(pa.page_offset(), 0);
		pa.floor()
	}
}