use oso_error::loader::UefiError;

use super::table::boot_services;
use crate::Rslt;
use crate::raw::service::BootServices;
use crate::raw::types::PhysicalAddress;
use crate::raw::types::Status;
use crate::raw::types::memory::AllocateType;
use crate::raw::types::memory::MemoryDescriptor;
use crate::raw::types::memory::MemoryMapInfo;
use crate::raw::types::memory::MemoryType;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::ptr::NonNull;

type RsltU<T,> = Rslt<T, UefiError,>;

#[global_allocator]
static LOADER_ALLOCATOR: LoaderAllocator = LoaderAllocator;

pub struct LoaderAllocator;

unsafe impl GlobalAlloc for LoaderAllocator {
	unsafe fn alloc(&self, layout: core::alloc::Layout,) -> *mut u8 {
		if layout.align() > 8 {
			panic!()
		}
		let mem_ty = MemoryType::LOADER_DATA;
		let bs = boot_services();
		bs.allocate_pool(mem_ty, layout.size(),)
			.expect("allocation failed",)
			.as_ptr()
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout,) {
		if layout.align() > 8 {
			panic!()
		}
		let bs = boot_services();
		bs.free_pool(unsafe { ptr.as_mut_unchecked() },)
			.expect("deallocation failed",);
	}
}

#[alloc_error_handler]
fn alloc_error(layout: Layout,) -> ! {
	panic!("system run out of memory: {layout:#?}")
}

impl BootServices {
	pub fn allocate_pool(
		&self,
		mem_ty: MemoryType,
		size: usize,
	) -> RsltU<NonNull<u8,>,> {
		let mut buf = core::ptr::null_mut();
		unsafe { (self.allocate_pool)(mem_ty, size, &mut buf,) }.ok_or()?;
		Ok(unsafe {
			// "allocate_pool must not return a null pointer if successful
			NonNull::new_unchecked(buf,)
		},)
	}

	pub fn free_pool(&self, ptr: &mut u8,) -> RsltU<Status,> {
		unsafe { (self.free_pool)(ptr,).ok_or() }
	}

	pub fn allocate_pages(
		&self,
		allocation_type: AllocateType,
		mem_ty: MemoryType,
		page_count: usize,
		mut alloc_head: PhysicalAddress,
	) -> RsltU<PhysicalAddress,> {
		unsafe {
			(self.allocate_pages)(
				allocation_type,
				mem_ty,
				page_count,
				&mut alloc_head,
			)
		}
		.ok_or_with(|_| alloc_head,)
	}

	pub fn memory_map_size(&self,) -> (usize, usize,) {
		let mut map_size = 0;
		let mut map_key = 0;
		let mut descriptor_size = 0;
		let mut desc_version = 0;

		let status = unsafe {
			(self.get_memory_map)(
				&mut map_size,
				core::ptr::null_mut(),
				&mut map_key,
				&mut descriptor_size,
				&mut desc_version,
			)
		};
		assert_eq!(status, Status::EFI_BUFFER_TOO_SMALL);

		assert_eq!(
			map_size % descriptor_size,
			0,
			"memory map size is multiple of descriptor size"
		);

		let memory_map_info = MemoryMapInfo {
			map_size,
			desc_size: descriptor_size,
			map_key,
			desc_ver: desc_version,
		};

		memory_map_info.assert_sanity_check();

		(map_size, descriptor_size,)
	}

	pub fn get_memory_map(&self, buf: &mut [u8],) -> RsltU<MemoryMapInfo,> {
		let mut map_size = buf.len();
		let map_buf = buf.as_mut_ptr().cast::<MemoryDescriptor>();
		let mut map_key = 0;
		let mut desc_size = 0;
		let mut desc_ver = 0;

		assert_eq!(
			(map_buf as usize) % align_of::<MemoryDescriptor,>(),
			0,
			"memory map buffer must be aligned like a memory descriptor"
		);

		unsafe {
			(self.get_memory_map)(
				&mut map_size,
				map_buf,
				&mut map_key,
				&mut desc_size,
				&mut desc_ver,
			)
		}
		.ok_or_with(|_| MemoryMapInfo {
			map_size,
			desc_size,
			map_key,
			desc_ver,
		},)
	}
}
