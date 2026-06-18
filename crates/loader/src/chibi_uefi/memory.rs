use {
	super::{drop_uefi_cleanup_result, table::boot_services},
	crate::raw::{
		service::BootServices,
		types::{
			PhysicalAddress, Status,
			memory::{
				AllocateType, MemoryDescriptor, MemoryMapInfo, MemoryType,
			},
		},
	},
	core::{
		alloc::{GlobalAlloc, Layout},
		ptr::NonNull,
	},
	poison_girl_macro::cfg_if,
	poison_girl_no_std_error::{PoisonGirlB, X},
};

/**
TODO:
P1 crates/loader/src/chibi_uefi/memory.rs:19: the UEFI allocator is still installed as the global
   allocator for loader tests. cargo test -p poison_girl_loader --lib now builds, but aborts at runtime with
   memory allocation of 4 bytes failed because LoaderAllocator depends on UEFI boot services that are not
   initialized in host tests.
 */
#[cfg(target_os = "uefi")]
#[global_allocator]
static LOADER_ALLOCATOR: LoaderAllocator = LoaderAllocator;

pub struct LoaderAllocator;

unsafe impl GlobalAlloc for LoaderAllocator
{
	unsafe fn alloc(&self, layout: Layout,) -> *mut u8
	{
		if layout.align() > 8 {
			return core::ptr::null_mut();
		}
		let mem_ty = MemoryType::LOADER_DATA;
		let X(bs,) = boot_services() else {
			return core::ptr::null_mut();
		};
		match bs.allocate_pool(mem_ty, layout.size(),) {
			X(ptr,) => ptr.as_ptr(),
			poison_girl_no_std_error::Y(_,) => core::ptr::null_mut(),
		}
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout,)
	{
		if layout.align() > 8 {
			return;
		}
		if let X(bs,) = boot_services()
			&& let Some(ptr,) = unsafe { ptr.as_mut() }
		{
			drop_uefi_cleanup_result(bs.free_pool(ptr,),);
		}
	}
}

cfg_if!(
	if #[cfg(all(not(test), target_os = "uefi"))] {
		/// TODO:
		/// P1 crates/loader/src/chibi_uefi/memory.rs:54: #[cfg_attr(not(test),
		/// alloc_error_handler)] still emits #[alloc_error_handler] when the loader
		/// library is compiled as a normal dependency of the loader bin test.
		/// cfg(test) is only true for the crate currently being built as a test
		/// target, so cargo test -p poison_girl_loader --all-targets --no-run still
		/// fails with the std allocation handler conflict. This  needs a gate tied to
		/// the real boot target or a feature, not only cfg(test).
		/// #[cfg_attr(not(test), alloc_error_handler)]
		#[alloc_error_handler]
		fn alloc_error(layout: Layout,) -> !
		{
			panic!("system run out of memory: {layout:#?}")
		}
	}
);

impl BootServices
{
	pub fn allocate_pool(
		&self,
		mem_ty: MemoryType,
		size: usize,
	) -> PoisonGirlB<NonNull<u8,>,>
	{
		let mut buf = core::ptr::null_mut();
		unsafe { (self.allocate_pool)(mem_ty, size, &mut buf,) }.x_or()?;
		X(unsafe {
			// "allocate_pool must not return a null pointer if successful
			NonNull::new_unchecked(buf,)
		},)
	}

	pub fn free_pool(&self, ptr: &mut u8,) -> PoisonGirlB<Status,>
	{
		unsafe { (self.free_pool)(ptr,).x_or() }
	}

	pub fn allocate_pages(
		&self,
		allocation_type: AllocateType,
		mem_ty: MemoryType,
		page_count: usize,
		mut alloc_head: PhysicalAddress,
	) -> PoisonGirlB<PhysicalAddress,>
	{
		unsafe {
			(self.allocate_pages)(
				allocation_type,
				mem_ty,
				page_count,
				&mut alloc_head,
			)
		}
		.x_or_with(|_| alloc_head,)
	}

	pub fn memory_map_size(&self,) -> (usize, usize,)
	{
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

	pub fn get_memory_map(&self, buf: &mut [u8],)
	-> PoisonGirlB<MemoryMapInfo,>
	{
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
		.x_or_with(|_| MemoryMapInfo {
			map_size,
			desc_size,
			map_key,
			desc_ver,
		},)
	}
}
