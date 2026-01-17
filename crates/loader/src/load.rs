//! # Kernel and Graphics Loading Module
//!
//! This module provides functionality for loading ELF kernels from the
//! filesystem and configuring graphics output for the kernel environment.

use crate::Rslt;
use crate::chibi_uefi::required_pages;
use crate::chibi_uefi::table::boot_services;
use crate::elf::Elf;
use crate::elf::program_header::ProgramHeaderType;
use crate::print;
use crate::println;
use crate::raw::protocol::file::FileProtocolV1;
use crate::raw::protocol::file::SimpleFileSystemProtocol;
use crate::raw::protocol::graphic::GraphicsOutputProtocol;
use crate::raw::types::PhysicalAddress;
use crate::raw::types::file::FileAttributes;
use crate::raw::types::file::OpenMode;
use crate::raw::types::memory::AllocateType;
use core::ptr::NonNull;
use oso_no_std_shared::bridge::graphic::FrameBufConf;

/// Loads the kernel ELF file and prepares it for execution
///
/// This function performs the complete kernel loading process:
/// 1. Opens the kernel ELF file from the filesystem
/// 2. Reads and parses the ELF content
/// 3. Calculates memory requirements for all loadable segments
/// 4. Allocates memory at the required virtual addresses
/// 5. Copies loadable segments to their target locations
/// 6. Returns the kernel entry point address
///
/// # Returns
///
/// * `Ok(PhysicalAddress)` - The physical address of the kernel entry point
/// * `Err(_)` - If any step of the loading process fails
///
/// # Errors
///
/// This function can fail if:
/// - The kernel file cannot be opened or read
/// - ELF parsing fails (invalid format, unsupported architecture, etc.)
/// - Memory allocation fails for kernel segments
/// - File I/O operations fail
///
/// # Panics
///
/// Panics if ELF parsing fails with an unrecoverable error, as this indicates
/// a fundamental problem with the kernel file that cannot be resolved.
pub fn kernel() -> Rslt<PhysicalAddress,> {
	// Open and read the kernel ELF file
	let mut kernel_file = open_kernel_file()?;
	let contents = unsafe { kernel_file.as_mut() }.read_as_bytes()?;

	// Parse the ELF file structure
	let elf = match Elf::parse(&contents,) {
		Ok(elf,) => elf,
		Err(e,) => panic!("unrecoverable error: {e:?}"),
	};

	// Calculate memory requirements for all loadable segments
	let (head, tail,) = elf_address_range(&elf,);
	let kernel_size = tail - head;

	// Allocate memory for the kernel at the required address
	let page_count = required_pages(kernel_size,);
	let alloc_head = boot_services().allocate_pages(
		AllocateType::ALLOCATE_ADDRESS,
		crate::raw::types::memory::MemoryType::LOADER_DATA,
		page_count,
		head as u64,
	)?;

	println!("----------------------------");

	// Verify allocation was at the requested address
	assert_eq!(alloc_head as usize, head);

	// Copy all loadable segments to their target locations
	copy_load_segment(&elf, &contents,);

	println!("head: {head:#x}, tail: {tail:#x}");

	Ok(elf.entry_point_address() as u64,)
}

/// Opens the kernel ELF file from the filesystem
///
/// This function locates the simple file system protocol and opens the
/// kernel file named "oso_kernel.elf" from the root directory.
///
/// # Returns
///
/// * `Ok(NonNull<FileProtocolV1>)` - Handle to the opened kernel file
/// * `Err(_)` - If file system access or file opening fails
///
/// # Errors
///
/// This function can fail if:
/// - No simple file system protocol is available
/// - The volume cannot be opened
/// - The kernel file does not exist or cannot be opened
fn open_kernel_file() -> Rslt<NonNull<FileProtocolV1,>,> {
	let open_mode = OpenMode::READ;
	let attrs = FileAttributes(0,);

	let bs = boot_services();

	// Locate the file system protocol
	let sfs_handle =
		unsafe { bs.handle_for_protocol::<SimpleFileSystemProtocol>() }?;

	// Open the root volume
	let volume = unsafe {
		bs.open_protocol_exclusive::<SimpleFileSystemProtocol>(sfs_handle,)?
			.interface()
			.as_mut()
	}
	.open_volume()?;

	// Open the kernel file
	let kernel_file = volume.open("oso_kernel.elf", open_mode, attrs,)?;
	let non_null_kernel_file =
		NonNull::new(kernel_file,).expect("reference can't be null",);
	Ok(non_null_kernel_file,)
}

/// Calculates the memory address range required for all loadable ELF segments
///
/// This function examines all program headers in the ELF file and determines
/// the minimum and maximum addresses needed to load all LOAD-type segments.
///
/// # Arguments
///
/// * `elf` - Reference to the parsed ELF file
///
/// # Returns
///
/// A tuple `(head_address, tail_address)` representing:
/// - `head_address`: The lowest virtual address of any loadable segment
/// - `tail_address`: The highest virtual address + size of any loadable segment
///
/// # Note
///
/// Only program headers with type `ProgramHeaderType::Load` are considered,
/// as these are the segments that need to be loaded into memory.
fn elf_address_range(elf: &Elf,) -> (usize, usize,) {
	let mut pair = (usize::MAX, 0,);

	// Examine each program header
	for ph in &elf.program_headers {
		if ph.ty != ProgramHeaderType::Load {
			continue;
		}

		let segment_head = ph.virtual_address as usize;
		let segment_tail = (ph.virtual_address + ph.memory_size) as usize;

		// Track minimum and maximum addresses
		pair.0 = pair.0.min(segment_head,);
		pair.1 = pair.1.max(segment_tail,);
	}

	pair
}

/// Copies all loadable ELF segments to their target memory locations
///
/// This function processes each LOAD-type program header and:
/// 1. Copies the segment data from the ELF file to the target virtual address
/// 2. Zero-fills any remaining memory (typically for .bss sections)
///
/// # Arguments
///
/// * `elf` - Reference to the parsed ELF file containing program headers
/// * `src` - The raw ELF file content as bytes
///
/// # Memory Layout
///
/// For each loadable segment:
/// - `file_size` bytes are copied from the ELF file
/// - Remaining bytes up to `memory_size` are zero-filled
/// - This handles cases where memory size > file size (e.g., .bss sections)
///
/// # Safety
///
/// This function uses unsafe operations to write directly to virtual memory
/// addresses specified in the ELF program headers. The caller must ensure
/// that the target memory has been properly allocated.
fn copy_load_segment(elf: &Elf, src: &[u8],) {
	for ph in &elf.program_headers {
		if ph.ty != ProgramHeaderType::Load {
			continue;
		}

		// Memory size may be larger than file size due to .bss section
		let mem_size = ph.memory_size as usize;
		let dest = unsafe {
			core::slice::from_raw_parts_mut(
				ph.virtual_address as *mut u8,
				mem_size,
			)
		};

		let offset = ph.offset as usize;
		let file_size = ph.file_size as usize;

		// Copy segment contents from ELF file
		dest[..file_size].copy_from_slice(&src[offset..offset + file_size],);
		// Zero-fill remaining memory (e.g., .bss section)
		dest[file_size..].fill(0,);
	}
}

/// Configures graphics output for the kernel
///
/// This function queries the UEFI Graphics Output Protocol to obtain
/// frame buffer configuration that will be passed to the kernel for
/// graphics operations.
///
/// # Returns
///
/// * `Ok(FrameBufConf)` - Frame buffer configuration containing:
///   - Pixel format information
///   - Frame buffer base address and size
///   - Resolution (width, height)
///   - Stride (bytes per scanline)
/// * `Err(_)` - If graphics protocol cannot be accessed
///
/// # Errors
///
/// This function can fail if:
/// - Graphics Output Protocol is not available
/// - Protocol interface cannot be opened
/// - Graphics mode information is invalid
///
/// # Usage
///
/// The returned configuration is typically passed to the kernel during
/// initialization to enable graphics output capabilities.
pub fn graphic_config() -> Rslt<FrameBufConf,> {
	let bs = boot_services();

	// Open Graphics Output Protocol
	let mut gout =
		bs.open_protocol_with::<GraphicsOutputProtocol>()?.interface();
	let gout = unsafe { gout.as_mut() };

	// Query current graphics mode information
	let info = gout.mode();
	let (width, height,) = info.resolution();
	let stride = info.stride();
	let base = info.frame_buffer_base as *mut u8;
	let size = info.frame_buffer_size;
	let pixel_format = info.pixel_format();

	// Create frame buffer configuration
	let fbc =
		FrameBufConf::new(pixel_format, base, size, width, height, stride,);

	Ok(fbc,)
}
