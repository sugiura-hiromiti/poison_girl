use crate::Rslt;
use crate::c_style_enum;
use crate::chibi_uefi::table::boot_services;
use alloc::format;
use core::ops::RangeInclusive;
use core::ptr::NonNull;

pub const PAGE_SIZE: usize = 4096;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
pub struct MemoryDescriptor {
	pub memory_type:    MemoryType,
	pub physical_start: u64,
	pub virtual_start:  u64,
	pub page_count:     u64,
	pub attribute:      MemoryAttribute,
}

c_style_enum! {
	pub enum AllocateType: isize => {
		ALLOCATE_ANY_PAGES   = 0,
		ALLOCATE_MAX_ADDRESS = 1,
		ALLOCATE_ADDRESS     = 2,
		MAX_ALLOCATE_TYPE    = 3,
	}
}

c_style_enum! {
	pub enum MemoryType: u32 => {
		RESERVED              = 0,
		LOADER_CODE           = 1,
		LOADER_DATA           = 2,
		BOOT_SERVICES_CODE    = 3,
		BOOT_SERVICES_DATA    = 4,
		RUNTIME_SERVICES_CODE = 5,
		RUNTIME_SERVICES_DATA = 6,
		CONVENTIONAL          = 7,
		UNUSABLE              = 8,
		ACPI_RECLAIM          = 9,
		ACPI_NON_VOLATILE     = 10,
		MMIO                  = 11,
		MMIO_PORT_SPACE       = 12,
		PAL_CODE              = 13,
		PERSISTENT_MEMORY     = 14,
		UNACCEPTED            = 15,
		MAX                   = 16,
	}
}

impl MemoryType {
	pub const RESERVED_FOR_OEM: RangeInclusive<u32,> =
		0x7000_0000..=0x7fff_ffff;
	pub const RESERVED_FOR_OS_LOADER: RangeInclusive<u32,> =
		0x8000_0000..=0xffff_ffff;

	pub const fn custom(value: u32,) -> Self {
		assert!(value >= 0x8000_0000);
		Self(value,)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(transparent)]
pub struct MemoryAttribute(pub u64,);

impl MemoryAttribute {
	pub const EFI_MEMORY_CPU_CRYPTO: u64 = 0x0000000000080000;
	pub const EFI_MEMORY_HOT_PLUGGABLE: u64 = 0x0000000000100000;
	pub const EFI_MEMORY_ISA_MASK: u64 = 0x0FFFF00000000000;
	pub const EFI_MEMORY_ISA_VALID: u64 = 0x4000000000000000;
	pub const EFI_MEMORY_MORE_RELIABLE: u64 = 0x0000000000010000;
	pub const EFI_MEMORY_NV: u64 = 0x0000000000008000;
	pub const EFI_MEMORY_RO: u64 = 0x0000000000020000;
	pub const EFI_MEMORY_RP: u64 = 0x0000000000002000;
	pub const EFI_MEMORY_RUNTIME: u64 = 0x8000000000000000;
	pub const EFI_MEMORY_SP: u64 = 0x0000000000040000;
	pub const EFI_MEMORY_UC: u64 = 0x0000000000000001;
	pub const EFI_MEMORY_UCE: u64 = 0x0000000000000010;
	pub const EFI_MEMORY_WB: u64 = 0x0000000000000008;
	pub const EFI_MEMORY_WC: u64 = 0x0000000000000002;
	pub const EFI_MEMORY_WP: u64 = 0x0000000000001000;
	pub const EFI_MEMORY_WT: u64 = 0x0000000000000004;
	pub const EFI_MEMORY_XP: u64 = 0x0000000000004000;
}

#[derive(Clone,)]
pub struct MemoryMapInfo {
	pub map_size:  usize,
	pub desc_size: usize,
	pub map_key:   usize,
	pub desc_ver:  u32,
}

impl MemoryMapInfo {
	pub fn assert_sanity_check(&self,) {
		assert!(self.desc_size > 0);
		assert!(self.desc_size >= size_of::<MemoryDescriptor,>());

		const ONE_GB: usize = 1024 * 1024 * 1024;
		assert!(self.map_size > 0);
		assert!(self.map_size <= ONE_GB);
	}

	pub fn entry_count(&self,) -> usize {
		self.map_size / self.desc_size
	}
}

impl core::fmt::Debug for MemoryMapInfo {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_,>,) -> core::fmt::Result {
		f.debug_struct("MemoryMapInfo",)
			.field("map_size", &format!("{:#x}", self.map_size),)
			.field("desc_size", &format!("{:#x}", self.desc_size),)
			.field("map_key", &format!("{:#x}", self.map_key),)
			.field("desc_ver", &format!("{:#x}", self.desc_ver),)
			.finish()
	}
}

#[derive(Clone,)]
pub struct MemoryMapBackingMemory(NonNull<[u8],>,);

impl MemoryMapBackingMemory {
	pub fn new(mem_ty: MemoryType,) -> Rslt<Self,> {
		let bs = boot_services();
		let (map_size, desc_size,) = bs.memory_map_size();
		let len = Self::safe_allocation_size_hint(map_size, desc_size,);
		let alloc_pos = bs.allocate_pool(mem_ty, len,)?.as_ptr();

		assert_eq!(alloc_pos.align_offset(align_of::<MemoryDescriptor,>()), 0);

		assert_eq!(map_size % desc_size, 0);

		unsafe { Ok(Self::from_raw(alloc_pos, len,),) }
	}

	unsafe fn from_raw(alloc_pos: *mut u8, len: usize,) -> Self {
		assert_eq!(alloc_pos.align_offset(align_of::<MemoryDescriptor,>()), 0);

		let ptr = NonNull::new(alloc_pos,)
			.expect("uefi should never return null ptr",);
		let slice = NonNull::slice_from_raw_parts(ptr, len,);

		Self(slice,)
	}

	fn safe_allocation_size_hint(map_size: usize, desc_size: usize,) -> usize {
		const EXTRA_ENTRIES: usize = 8;

		let extra_size = desc_size * EXTRA_ENTRIES;
		map_size + extra_size
	}

	pub fn as_mut_slice(&mut self,) -> &mut [u8] {
		unsafe { self.0.as_mut() }
	}
}

pub struct MemoryMapOwned {
	pub buf:  MemoryMapBackingMemory,
	pub info: MemoryMapInfo,
	pub len:  usize,
}

impl MemoryMapOwned {
	pub fn from_initialized_memory(
		buf: MemoryMapBackingMemory,
		info: MemoryMapInfo,
	) -> Self {
		assert!(info.desc_size >= size_of::<MemoryDescriptor,>());

		let len = info.entry_count();
		Self { buf, info, len, }
	}
}
