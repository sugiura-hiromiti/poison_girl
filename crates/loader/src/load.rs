//! # Kernel and Graphics Loading Module
//!
//! This module provides functionality for loading ELF kernels from the
//! filesystem and configuring graphics output for the kernel environment.

use {
	crate::{
		chibi_uefi::{required_pages, table::boot_services},
		elf::{
			Elf,
			program_header::{ProgramHeader, ProgramHeaderType},
		},
		print, println,
		raw::{
			protocol::{
				file::{FileProtocolV1, SimpleFileSystemProtocol},
				graphic::GraphicsOutputProtocol,
			},
			types::{
				PhysicalAddress,
				file::{FileAttributes, OpenMode},
				graphic::GraphicsOutputProtocolMode,
				memory::AllocateType,
			},
		},
	},
	core::ptr::NonNull,
	poison_girl_no_std::{KERNEL_FILE_NAME, bridge::graphic::FrameBufConf},
	poison_girl_no_std_error::{
		ElfParseError, PoisonGirlB, X, Y, poison_girl_err,
	},
};

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
pub fn kernel() -> PoisonGirlB<PhysicalAddress,>
{
	// Open and read the kernel ELF file
	let mut kernel_file = open_kernel_file()?;
	let contents = unsafe { kernel_file.as_mut() }.read_as_bytes()?;

	// Parse the ELF file structure
	let elf = match Elf::parse(&contents,) {
		X(elf,) => elf,
		Y(e,) => panic!("unrecoverable error: {e:?}"),
	};

	// Calculate memory requirements for all loadable segments
	let (head, tail,) = elf_address_range(&elf.program_headers,)?;
	let kernel_size = tail - head;

	// Allocate memory for the kernel at the required address
	let page_count = required_pages(kernel_size,);
	let alloc_head = boot_services()?.allocate_pages(
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

	X(elf.entry_point_address() as u64,)
}

/// Opens the kernel ELF file from the filesystem
///
/// This function locates the simple file system protocol and opens the
/// kernel file named "poison_girl_kernel.elf" from the root directory.
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
fn open_kernel_file() -> PoisonGirlB<NonNull<FileProtocolV1,>,>
{
	let open_mode = OpenMode::READ;
	let attrs = FileAttributes(0,);

	let bs = boot_services()?;

	// Locate the file system protocol
	let sfs_handle =
		unsafe { bs.handle_for_protocol::<SimpleFileSystemProtocol>() }?;

	// Open the root volume
	let volume = unsafe {
		bs.open_protocol_exclusive::<SimpleFileSystemProtocol>(sfs_handle,)?
			.interface()?
			.as_mut()
	}
	.open_volume()?;

	// Open the kernel file
	let kernel_file = volume.open(KERNEL_FILE_NAME, open_mode, attrs,)?;
	X(NonNull::from(kernel_file,),)
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
/// On success, a tuple `(head_address, tail_address)` representing:
/// - `head_address`: The lowest virtual address of any loadable segment
/// - `tail_address`: The highest virtual address + size of any loadable segment
///
/// Returns `ElfParseError::NoLoadSegments` when no loadable program headers
/// are present.
///
/// # Note
///
/// Only program headers with type `ProgramHeaderType::Load` are considered,
/// as these are the segments that need to be loaded into memory.
fn elf_address_range(
	program_headers: &[ProgramHeader],
) -> PoisonGirlB<(usize, usize,),>
{
	let mut pair: Option<(usize, usize,),> = None;

	// Examine each program header
	for ph in program_headers {
		if ph.ty != ProgramHeaderType::Load {
			continue;
		}

		let segment_head = ph.virtual_address as usize;
		let segment_tail = (ph.virtual_address + ph.memory_size) as usize;

		// Track minimum and maximum addresses
		pair = Some(match pair {
			Some((head, tail,),) => {
				(head.min(segment_head,), tail.max(segment_tail,),)
			},
			None => (segment_head, segment_tail,),
		},);
	}

	match pair {
		Some(pair,) => X(pair,),
		None => Y(poison_girl_err!(ElfParseError::NoLoadSegments),),
	}
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
fn copy_load_segment(elf: &Elf, src: &[u8],)
{
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

		copy_load_segment_to_slice(ph, src, dest,);
	}
}

fn copy_load_segment_to_slice(ph: &ProgramHeader, src: &[u8], dest: &mut [u8],)
{
	let mem_size = ph.memory_size as usize;
	let file_size = ph.file_size as usize;
	assert!(file_size <= mem_size);
	assert!(dest.len() >= mem_size);

	let offset = ph.offset as usize;

	// Copy segment contents from ELF file
	dest[..file_size].copy_from_slice(&src[offset..offset + file_size],);
	// Zero-fill remaining memory (e.g., .bss section)
	dest[file_size..mem_size].fill(0,);
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
pub fn graphic_config() -> PoisonGirlB<FrameBufConf,>
{
	let bs = boot_services()?;

	// Open Graphics Output Protocol
	let mut gout =
		bs.open_protocol_with::<GraphicsOutputProtocol>()?.interface()?;
	let gout = unsafe { gout.as_mut() };

	// Query current graphics mode information
	let info = gout.mode();

	// Create frame buffer configuration
	let fbc = frame_buffer_config_from_mode(info,);

	X(fbc,)
}

fn frame_buffer_config_from_mode(
	info: &GraphicsOutputProtocolMode,
) -> FrameBufConf
{
	let (width, height,) = info.resolution();
	let base = info.frame_buffer_base as *mut u8;
	let size = info.frame_buffer_size;
	let pixel_format = info.pixel_format();
	let stride = info.stride() * pixel_format.bytes_per_pixel().unwrap_or(1,);

	FrameBufConf::new(pixel_format, base, size, width, height, stride,)
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::raw::types::graphic::{
			GraphicsOutputModeInfo, GraphicsOutputProtocolMode,
			GraphicsPixelFormat, PixelBitMask,
		},
		poison_girl_no_std::bridge::graphic::PixelFormatConf,
	};

	fn ph(
		ty: ProgramHeaderType,
		offset: u64,
		virtual_address: u64,
		file_size: u64,
		memory_size: u64,
	) -> ProgramHeader
	{
		ProgramHeader {
			ty,
			flags: 0,
			offset,
			virtual_address,
			physical_address: virtual_address,
			file_size,
			memory_size,
			align: 0,
		}
	}

	#[test]
	fn address_range_without_load_segments_returns_error()
	{
		let headers = [
			ph(ProgramHeaderType::Null, 0, 0x1000, 0, 0x100,),
			ph(ProgramHeaderType::Interp, 0, 0x2000, 0, 0x100,),
		];

		assert!(matches!(elf_address_range(&headers,), Y(_)));
	}

	#[test]
	fn address_range_uses_only_load_segments_for_min_head_and_max_tail()
	{
		let headers = [
			ph(ProgramHeaderType::Interp, 0, 0x100, 0x20, 0x10000,),
			ph(ProgramHeaderType::Load, 0, 0x3000, 0x20, 0x400,),
			ph(ProgramHeaderType::Load, 0, 0x1000, 0x20, 0x500,),
			ph(ProgramHeaderType::Load, 0, 0x1200, 0x20, 0x100,),
			ph(ProgramHeaderType::Dynamic, 0, 0x4000, 0x20, 0x1000,),
		];

		match elf_address_range(&headers,) {
			X(pair,) => assert_eq!(pair, (0x1000, 0x3400)),
			Y(_,) => panic!("expected a load segment range"),
		}
	}

	#[test]
	fn copy_load_segment_copies_file_bytes_and_zero_fills_tail()
	{
		let src = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9,];
		let ph = ph(ProgramHeaderType::Load, 3, 0x1000, 4, 8,);
		let mut dest = [0xaa; 10];

		copy_load_segment_to_slice(&ph, &src, &mut dest,);

		assert_eq!(&dest[..8], &[3, 4, 5, 6, 0, 0, 0, 0]);
		assert_eq!(&dest[8..], &[0xaa, 0xaa]);
	}

	#[test]
	fn copy_load_segment_without_bss_tail_copies_only_file_bytes()
	{
		let src = [10, 11, 12, 13, 14, 15,];
		let ph = ph(ProgramHeaderType::Load, 1, 0x1000, 3, 3,);
		let mut dest = [0xaa; 3];

		copy_load_segment_to_slice(&ph, &src, &mut dest,);

		assert_eq!(dest, [11, 12, 13]);
	}

	#[test]
	fn frame_buffer_config_converts_raw_graphics_mode()
	{
		let mut info = GraphicsOutputModeInfo {
			version:               0,
			horizontal_resolution: 100,
			vertical_resolution:   50,
			pixel_format:
				GraphicsPixelFormat::RGB_RESERVED_8_BIT_PER_COLOR,
			pixel_info:            PixelBitMask::default(),
			pixels_per_scal_line:  128,
		};
		let mode = GraphicsOutputProtocolMode {
			max_mode:          3,
			mode:              1,
			info:              &mut info,
			frame_buffer_base: 0x1000,
			frame_buffer_size: 128 * 50 * 4,
		};

		let config = frame_buffer_config_from_mode(&mode,);

		assert_eq!(config.pixel_format, PixelFormatConf::Rgb);
		assert_eq!(config.base, 0x1000 as *mut u8);
		assert_eq!(config.size, 128 * 50 * 4);
		assert_eq!(config.width, 100);
		assert_eq!(config.height, 50);
		assert_eq!(config.stride, 128 * 4);
	}
}
