//! # Graphics Bridge Module
//!
//! This module provides types and utilities for working with framebuffers and
//! graphics output in a bare-metal environment. It serves as a bridge between
//! the bootloader and kernel for graphics configuration.
//!
//! ## Overview
//!
//! The graphics bridge module is designed to facilitate communication between
//! different components of the OSO system regarding graphics configuration.
//! It provides standardized data structures that can be safely passed between
//! the bootloader and kernel to configure display hardware.
//!
//! ## Key Components
//!
//! - [`PixelFormatConf`]: Enum representing different pixel formats
//! - [`FrameBufConf`]: Structure containing framebuffer configuration
//!   parameters
//!
//! ## Design Principles
//!
//! - **ABI Stability**: Uses `#[repr(C)]` for consistent memory layout
//! - **No Standard Library**: Works in `no_std` environments
//! - **Safety**: Provides safe abstractions over raw hardware interfaces
//! - **Flexibility**: Supports multiple pixel formats and display
//!   configurations
//!
//! ## Usage Scenarios
//!
//! ### Bootloader to Kernel Handoff
//!
//! The bootloader discovers display hardware and creates a `FrameBufConf`:
//!
//! ```rust,no_run
//! use oso_no_std_shared::bridge::graphic::FrameBufConf;
//! use oso_no_std_shared::bridge::graphic::PixelFormatConf;
//!
//! // Bootloader code
//! let framebuf_config = FrameBufConf::new(
//! 	PixelFormatConf::Rgb,
//! 	framebuffer_base_address,
//! 	framebuffer_size,
//! 	screen_width,
//! 	screen_height,
//! 	bytes_per_line,
//! );
//!
//! // Pass to kernel...
//! kernel_main(framebuf_config,);
//! ```
//!
//! ### Kernel Graphics Initialization
//!
//! The kernel receives the configuration and initializes its graphics
//! subsystem:
//!
//! ```rust,no_run
//! // Kernel code
//! fn initialize_graphics(config: FrameBufConf,) {
//! 	match config.pixel_format {
//! 		PixelFormatConf::Rgb => {
//! 			// Initialize RGB graphics driver
//! 		},
//! 		PixelFormatConf::Bgr => {
//! 			// Initialize BGR graphics driver
//! 		},
//! 		// ... handle other formats
//! 	}
//! }
//! ```
//!
//! ## Memory Layout Considerations
//!
//! The framebuffer memory layout depends on the pixel format and stride:
//!
//! - **Stride**: May be larger than `width * bytes_per_pixel` due to alignment
//!   requirements
//! - **Padding**: Some hardware requires row padding for optimal performance
//! - **Endianness**: Pixel format determines byte order within each pixel
//!
//! ## Safety Considerations
//!
//! - The `base` pointer must point to valid, writable memory
//! - The `size` must accurately represent the available framebuffer memory
//! - Memory access must respect the stride to avoid buffer overruns
//! - Concurrent access to framebuffer memory should be synchronized

/// Represents the pixel format configuration for a framebuffer
///
/// This enum defines the different pixel formats that can be used when
/// configuring a framebuffer for graphics output. Each format represents a
/// different way of organizing color data within each pixel.
///
/// # Pixel Format Details
///
/// ## RGB Format
/// - **Layout**: Red, Green, Blue (and optionally Alpha)
/// - **Common Variants**: R8G8B8 (24-bit), R8G8B8A8 (32-bit)
/// - **Byte Order**: Red in lowest address, Blue in highest
/// - **Usage**: Most common format on PC graphics hardware
///
/// ## BGR Format
/// - **Layout**: Blue, Green, Red (and optionally Alpha)
/// - **Common Variants**: B8G8R8 (24-bit), B8G8R8A8 (32-bit)
/// - **Byte Order**: Blue in lowest address, Red in highest
/// - **Usage**: Common on some embedded systems and older hardware
///
/// ## Bitmask Format
/// - **Layout**: Custom bit arrangement defined by masks
/// - **Flexibility**: Allows for non-standard color arrangements
/// - **Complexity**: Requires additional mask information for proper handling
/// - **Usage**: Specialized hardware or legacy compatibility
///
/// ## BltOnly Format
/// - **Layout**: No direct pixel access
/// - **Operation**: Only block transfer operations supported
/// - **Performance**: May be optimized for bulk operations
/// - **Usage**: Hardware with limited pixel-level access
///
/// # Examples
///
/// ```rust
/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
///
/// // Configure different pixel formats
/// let rgb_format = PixelFormatConf::Rgb;
/// let bgr_format = PixelFormatConf::Bgr;
/// let bitmask_format = PixelFormatConf::Bitmask;
/// let blt_format = PixelFormatConf::BltOnly;
///
/// // Check format type
/// match rgb_format {
/// 	PixelFormatConf::Rgb => println!("Using RGB format"),
/// 	PixelFormatConf::Bgr => println!("Using BGR format"),
/// 	PixelFormatConf::Bitmask => println!("Using custom bitmask format"),
/// 	PixelFormatConf::BltOnly => println!("Using block transfer only"),
/// }
/// ```
///
/// # Performance Implications
///
/// Different pixel formats may have different performance characteristics:
///
/// - **RGB/BGR**: Direct pixel access, good for individual pixel operations
/// - **Bitmask**: May require additional computation for color conversion
/// - **BltOnly**: Optimized for bulk operations, limited individual pixel
///   access
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy,)]
pub enum PixelFormatConf {
	/// Red, Green, Blue color format
	///
	/// Standard RGB format where red is in the lowest memory address,
	/// followed by green, then blue. May include an alpha channel.
	Rgb,

	/// Blue, Green, Red color format
	///
	/// BGR format where blue is in the lowest memory address,
	/// followed by green, then red. May include an alpha channel.
	Bgr,

	/// Custom color format defined by bit masks
	///
	/// Allows for custom pixel layouts defined by separate bit masks
	/// for each color component. Requires additional mask information.
	Bitmask,

	/// Block transfer only, no direct pixel access
	///
	/// Format that only supports bulk operations like copying rectangular
	/// regions. Individual pixel access may not be available or efficient.
	BltOnly,
}

impl PixelFormatConf {
	/// Returns the typical bytes per pixel for this format
	///
	/// This method provides an estimate of the bytes per pixel for common
	/// implementations of each pixel format. Actual values may vary based
	/// on specific hardware implementations.
	///
	/// # Returns
	///
	/// The typical number of bytes per pixel, or `None` for formats where
	/// this cannot be determined without additional information.
	///
	/// # Examples
	///
	/// ```rust
	/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
	///
	/// assert_eq!(PixelFormatConf::Rgb.bytes_per_pixel(), Some(4));
	/// assert_eq!(PixelFormatConf::Bgr.bytes_per_pixel(), Some(4));
	/// assert_eq!(PixelFormatConf::Bitmask.bytes_per_pixel(), None);
	/// assert_eq!(PixelFormatConf::BltOnly.bytes_per_pixel(), None);
	/// ```
	pub fn bytes_per_pixel(&self,) -> Option<usize,> {
		match self {
			PixelFormatConf::Rgb | PixelFormatConf::Bgr => Some(4,), /* Assuming 32-bit RGBA/BGRA */
			PixelFormatConf::Bitmask | PixelFormatConf::BltOnly => None, /* Variable or unknown */
		}
	}

	/// Checks if this format supports direct pixel access
	///
	/// Some formats, particularly BltOnly, may not support efficient
	/// individual pixel access and are optimized for bulk operations.
	///
	/// # Returns
	///
	/// `true` if individual pixel access is supported and efficient,
	/// `false` if only bulk operations are recommended.
	///
	/// # Examples
	///
	/// ```rust
	/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
	///
	/// assert!(PixelFormatConf::Rgb.supports_pixel_access());
	/// assert!(PixelFormatConf::Bgr.supports_pixel_access());
	/// assert!(PixelFormatConf::Bitmask.supports_pixel_access());
	/// assert!(!PixelFormatConf::BltOnly.supports_pixel_access());
	/// ```
	pub fn supports_pixel_access(&self,) -> bool {
		match self {
			PixelFormatConf::Rgb
			| PixelFormatConf::Bgr
			| PixelFormatConf::Bitmask => true,
			PixelFormatConf::BltOnly => false,
		}
	}
}

// TODO: Implement pixel format conversion when allocator is available
// This would allow runtime conversion between different pixel formats
// impl<D: Draw,> PixelFormatConf {
// 	fn convert(self,) -> impl Draw {
// 		match self {
// 			PixelFormatConf::Rgb => Rgb,
// 			PixelFormatConf::Bgr => Bgr,
// 			PixelFormatConf::Bitmask => Bitmask,
// 			PixelFormatConf::BltOnly => BltOnly,
// 		}
// 	}
// }

/// Configuration structure for a framebuffer
///
/// This structure contains all the necessary information to configure and use
/// a framebuffer for graphics output. It is designed to be passed between the
/// bootloader and kernel to provide information about the display hardware.
///
/// ## ABI Stability
///
/// Since Rust doesn't have a stabilized ABI, this structure uses the
/// `#[repr(C)]` attribute to ensure a consistent memory layout when passed
/// between components compiled with different versions of Rust or different
/// compilers.
///
/// ## Memory Layout
///
/// The framebuffer memory is organized as a linear array of pixels, with each
/// row potentially padded to meet alignment requirements:
///
/// ```text
/// Row 0: [Pixel 0][Pixel 1]...[Pixel width-1][Padding]
/// Row 1: [Pixel 0][Pixel 1]...[Pixel width-1][Padding]
/// ...
/// Row height-1: [Pixel 0][Pixel 1]...[Pixel width-1][Padding]
/// ```
///
/// The `stride` field indicates the number of bytes from the start of one row
/// to the start of the next row, which may be larger than `width *
/// bytes_per_pixel` due to alignment requirements.
///
/// ## Fields
///
/// * `pixel_format` - The pixel format used by the framebuffer
/// * `base` - Pointer to the start of the framebuffer memory
/// * `size` - Total size of the framebuffer in bytes
/// * `width` - Width of the display in pixels
/// * `height` - Height of the display in pixels
/// * `stride` - Number of bytes per row (including any padding)
///
/// ## Examples
///
/// ### Basic Configuration
///
/// ```rust,no_run
/// use oso_no_std_shared::bridge::graphic::FrameBufConf;
/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
///
/// // Create a framebuffer configuration for a 1024x768 display
/// let framebuf = FrameBufConf::new(
/// 	PixelFormatConf::Rgb,
/// 	0x1000_0000 as *mut u8, // Base address from firmware
/// 	1024 * 768 * 4,         // Size (4 bytes per pixel)
/// 	1024,                   // Width in pixels
/// 	768,                    // Height in pixels
/// 	1024 * 4,               // Stride (no padding in this example)
/// );
/// ```
///
/// ### With Row Padding
///
/// ```rust,no_run
/// use oso_no_std_shared::bridge::graphic::FrameBufConf;
/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
///
/// // Framebuffer with row padding for alignment
/// let framebuf = FrameBufConf::new(
/// 	PixelFormatConf::Rgb,
/// 	0x1000_0000 as *mut u8,
/// 	1920 * 1080 * 4 + 1080 * 64, // Extra space for padding
/// 	1920,                        // Width in pixels
/// 	1080,                        // Height in pixels
/// 	1920 * 4 + 64,               // Stride includes 64 bytes padding per row
/// );
/// ```
///
/// ## Safety Considerations
///
/// This structure contains a raw pointer to the framebuffer memory.
/// Users must ensure that:
///
/// - The `base` pointer is valid and points to writable memory
/// - The `size` accurately represents the available framebuffer memory
/// - Memory access respects the `stride` to avoid buffer overruns
/// - Concurrent access to framebuffer memory is properly synchronized
/// - The memory region remains valid for the lifetime of the configuration
#[derive(Debug,)]
#[repr(C)]
pub struct FrameBufConf {
	/// The pixel format used by the framebuffer
	pub pixel_format: PixelFormatConf,
	/// Pointer to the start of the framebuffer memory
	pub base:         *mut u8,
	/// Total size of the framebuffer in bytes
	pub size:         usize,
	/// Width of the display in pixels
	pub width:        usize,
	/// Height of the display in pixels
	pub height:       usize,
	/// Number of bytes per row (may include padding)
	pub stride:       usize,
}

impl FrameBufConf {
	/// Creates a new framebuffer configuration with the specified parameters
	///
	/// This constructor validates that the provided parameters are consistent
	/// and creates a new framebuffer configuration.
	///
	/// # Parameters
	///
	/// - `pixel_format` - The pixel format used by the framebuffer
	/// - `base` - Pointer to the start of the framebuffer memory
	/// - `size` - Total size of the framebuffer in bytes
	/// - `width` - Width of the display in pixels
	/// - `height` - Height of the display in pixels
	/// - `stride` - Number of bytes per row (including any padding)
	///
	/// # Returns
	///
	/// A new `FrameBufConf` instance with the specified parameters.
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// use oso_no_std_shared::bridge::graphic::FrameBufConf;
	/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
	///
	/// // Create a configuration for a Full HD display
	/// let framebuf = FrameBufConf::new(
	/// 	PixelFormatConf::Rgb,
	/// 	0x1000_0000 as *mut u8, // Base address
	/// 	1920 * 1080 * 4,        // Size (4 bytes per pixel)
	/// 	1920,                   // Width (pixels)
	/// 	1080,                   // Height (pixels)
	/// 	1920 * 4,               // Stride (bytes per row)
	/// );
	///
	/// assert_eq!(framebuf.width, 1920);
	/// assert_eq!(framebuf.height, 1080);
	/// ```
	///
	/// # Panics
	///
	/// This function may panic in debug builds if the parameters are
	/// inconsistent (e.g., if `stride * height > size`).
	pub fn new(
		pixel_format: PixelFormatConf,
		base: *mut u8,
		size: usize,
		width: usize,
		height: usize,
		stride: usize,
	) -> Self {
		// Debug assertions to catch common configuration errors
		debug_assert!(width > 0, "Width must be greater than 0");
		debug_assert!(height > 0, "Height must be greater than 0");
		debug_assert!(stride > 0, "Stride must be greater than 0");
		debug_assert!(
			stride >= width * pixel_format.bytes_per_pixel().unwrap_or(1),
			"Stride must be at least width * bytes_per_pixel"
		);
		debug_assert!(
			stride * height <= size,
			"Total framebuffer size must accommodate all rows"
		);

		Self { pixel_format, size, base, width, height, stride, }
	}

	/// Calculates the byte offset for a pixel at the given coordinates
	///
	/// This method computes the byte offset from the framebuffer base address
	/// for a pixel at the specified (x, y) coordinates.
	///
	/// # Arguments
	///
	/// * `x` - X coordinate (column) of the pixel
	/// * `y` - Y coordinate (row) of the pixel
	///
	/// # Returns
	///
	/// The byte offset from the base address, or `None` if the coordinates
	/// are out of bounds or the pixel format doesn't support pixel access.
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// use oso_no_std_shared::bridge::graphic::FrameBufConf;
	/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
	///
	/// let framebuf = FrameBufConf::new(
	/// 	PixelFormatConf::Rgb,
	/// 	0x1000_0000 as *mut u8,
	/// 	1024 * 768 * 4,
	/// 	1024,
	/// 	768,
	/// 	1024 * 4,
	/// );
	///
	/// // Calculate offset for pixel at (100, 50)
	/// if let Some(offset,) = framebuf.pixel_offset(100, 50,) {
	/// 	println!("Pixel offset: {}", offset);
	/// }
	/// ```
	pub fn pixel_offset(&self, x: usize, y: usize,) -> Option<usize,> {
		// Check bounds
		if x >= self.width || y >= self.height {
			return None;
		}

		// Check if pixel access is supported
		if !self.pixel_format.supports_pixel_access() {
			return None;
		}

		// Calculate offset
		let bytes_per_pixel = self.pixel_format.bytes_per_pixel()?;
		Some(y * self.stride + x * bytes_per_pixel,)
	}

	/// Returns the total number of pixels in the framebuffer
	///
	/// # Returns
	///
	/// The total number of pixels (width * height)
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// use oso_no_std_shared::bridge::graphic::FrameBufConf;
	/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
	///
	/// let framebuf = FrameBufConf::new(
	/// 	PixelFormatConf::Rgb,
	/// 	0x1000_0000 as *mut u8,
	/// 	1024 * 768 * 4,
	/// 	1024,
	/// 	768,
	/// 	1024 * 4,
	/// );
	///
	/// assert_eq!(framebuf.pixel_count(), 1024 * 768);
	/// ```
	pub fn pixel_count(&self,) -> usize {
		self.width * self.height
	}

	/// Checks if the framebuffer configuration is valid
	///
	/// This method performs basic validation of the framebuffer parameters
	/// to ensure they are consistent and reasonable.
	///
	/// # Returns
	///
	/// `true` if the configuration appears valid, `false` otherwise
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// use oso_no_std_shared::bridge::graphic::FrameBufConf;
	/// use oso_no_std_shared::bridge::graphic::PixelFormatConf;
	///
	/// let framebuf = FrameBufConf::new(
	/// 	PixelFormatConf::Rgb,
	/// 	0x1000_0000 as *mut u8,
	/// 	1024 * 768 * 4,
	/// 	1024,
	/// 	768,
	/// 	1024 * 4,
	/// );
	///
	/// assert!(framebuf.is_valid());
	/// ```
	pub fn is_valid(&self,) -> bool {
		// Basic sanity checks
		if self.width == 0
			|| self.height == 0
			|| self.stride == 0
			|| self.size == 0
		{
			return false;
		}

		// Check if stride is reasonable
		if let Some(bytes_per_pixel,) = self.pixel_format.bytes_per_pixel()
			&& self.stride < self.width * bytes_per_pixel
		{
			return false;
		}

		// Check if size is sufficient
		if self.stride * self.height > self.size {
			return false;
		}

		// Check for null pointer
		if self.base.is_null() {
			return false;
		}

		true
	}
}
