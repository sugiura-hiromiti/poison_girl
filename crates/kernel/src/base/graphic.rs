//! # Graphics and Display Management
//!
//! This module provides comprehensive graphics functionality for the OSO
//! kernel, including framebuffer management, pixel manipulation, and drawing
//! operations. It supports multiple pixel formats through compile-time feature
//! flags and provides a safe abstraction over raw framebuffer memory.
//!
//! ## Features
//!
//! - **Multiple Pixel Formats**: Support for RGB, BGR, Bitmask, and BLT-only
//!   formats
//! - **Safe Memory Access**: Memory-safe framebuffer operations with bounds
//!   checking
//! - **Drawing Primitives**: Pixel, rectangle, and outline drawing operations
//! - **Coordinate System**: Flexible coordinate representation and validation
//! - **Static Framebuffer**: Global framebuffer instance for system-wide
//!   graphics
//!
//! ## Pixel Format Support
//!
//! The module supports different pixel formats through feature flags:
//! - `rgb`: Red-Green-Blue pixel format (24-bit color)
//! - `bgr`: Blue-Green-Red pixel format (24-bit color)
//! - `bitmask`: Custom bitmask pixel format
//! - `bltonly`: Block Transfer Only mode (default)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use oso_kernel::base::graphic::{DisplayDraw, FRAME_BUFFER};
//! use oso_kernel::base::graphic::position::Coord;
//! use oso_kernel::base::graphic::color::Rgb;
//!
//! // Draw a single pixel
//! let coord = Coord::new(100, 50);
//! let color = Rgb::new(255, 0, 0); // Red
//! FRAME_BUFFER.put_pixel(&coord, &color)?;
//!
//! // Fill a rectangle
//! let top_left = Coord::new(10, 10);
//! let bottom_right = Coord::new(50, 30);
//! FRAME_BUFFER.fill_rectangle(&top_left, &bottom_right, &color)?;
//! ```

use crate::base::graphic::color::ColorRpr;
use crate::base::graphic::color::PixelFormat;
use crate::base::graphic::position::Coord;
use crate::base::graphic::position::Coordinal;
#[cfg(feature = "bgr")] use color::Bgr;
#[cfg(feature = "bitmask")] use color::Bitmask;
#[cfg(feature = "bltonly")] use color::BltOnly;
#[cfg(feature = "rgb")] use color::Rgb;
use oso_error::Rslt;
use oso_error::kernel::GraphicError;
use oso_error::oso_err;
// use oso_proc_macro::gen_wrapper_fn;

/// Color representation and pixel format implementations
pub mod color;
/// Coordinate system and position management
pub mod position;

/// Global framebuffer instance for RGB pixel format
///
/// This static framebuffer is available when the `rgb` feature is enabled.
/// It provides system-wide access to graphics operations using the RGB color
/// format. The framebuffer is initialized with default values and must be
/// properly configured using the `init()` method before use.
///
/// # Safety
///
/// This static instance uses interior mutability through unsafe operations.
/// Proper synchronization is required in multi-threaded environments.
///
/// # Examples
///
/// ```rust,ignore
/// #[cfg(feature = "rgb")]
/// use oso_kernel::base::graphic::FRAME_BUFFER;
///
/// // Initialize the framebuffer (typically done during kernel boot)
/// unsafe {
///     FrameBuffer::init(
///         &FRAME_BUFFER,
///         framebuffer_base_address,
///         framebuffer_size,
///         screen_width,
///         screen_height,
///         bytes_per_line
///     );
/// }
/// ```
//  TODO: use `MaybeUninit`
//  - support multi thread like using atomic
#[cfg(feature = "rgb")]
pub static FRAME_BUFFER: FrameBuffer<Rgb,> = FrameBuffer {
	drawer: Rgb,
	buf:    0,
	size:   0,
	width:  0,
	height: 0,
	stride: 0,
};

/// Global framebuffer instance for BGR pixel format
///
/// This static framebuffer is available when the `bgr` feature is enabled.
/// It provides system-wide access to graphics operations using the BGR color
/// format.
///
/// # Safety
///
/// This static instance uses interior mutability through unsafe operations.
/// Proper synchronization is required in multi-threaded environments.
#[cfg(feature = "bgr")]
pub static FRAME_BUFFER: FrameBuffer<Bgr,> = FrameBuffer {
	drawer: Bgr,
	buf:    0,
	size:   0,
	width:  0,
	height: 0,
	stride: 0,
};

/// Global framebuffer instance for Bitmask pixel format
///
/// This static framebuffer is available when the `bitmask` feature is enabled.
/// It provides system-wide access to graphics operations using custom bitmask
/// color format.
///
/// # Safety
///
/// This static instance uses interior mutability through unsafe operations.
/// Proper synchronization is required in multi-threaded environments.
#[cfg(feature = "bitmask")]
pub static FRAME_BUFFER: FrameBuffer<Bitmask,> = FrameBuffer {
	drawer: Bitmask,
	buf:    0,
	size:   0,
	width:  0,
	height: 0,
	stride: 0,
};

/// Global framebuffer instance for BLT-only pixel format
///
/// This static framebuffer is available when the `bltonly` feature is enabled
/// (default). It provides system-wide access to graphics operations using Block
/// Transfer Only mode.
///
/// # Safety
///
/// This static instance uses interior mutability through unsafe operations.
/// Proper synchronization is required in multi-threaded environments.
#[cfg(feature = "bltonly")]
pub static FRAME_BUFFER: FrameBuffer<BltOnly,> = FrameBuffer {
	drawer: BltOnly,
	buf:    0,
	size:   0,
	width:  0,
	height: 0,
	stride: 0,
};

/// Trait for drawing operations on display devices
///
/// This trait defines the core drawing operations that can be performed on a
/// display, including pixel manipulation and rectangle drawing. It provides a
/// consistent interface across different pixel formats and display types.
///
/// # Associated Types
///
/// * `Output` - The result type for drawing operations, typically `Result<(),
///   GraphicError>`
///
/// # Examples
///
/// ```rust,ignore
/// use oso_kernel::base::graphic::{DisplayDraw, FrameBuffer};
/// use oso_kernel::base::graphic::color::Rgb;
/// use oso_kernel::base::graphic::position::Coord;
///
/// let framebuffer = FrameBuffer::new(Rgb);
/// let coord = Coord::new(10, 20);
/// let color = Rgb::new(255, 0, 0);
///
/// // Draw a single pixel
/// framebuffer.put_pixel(&coord, &color)?;
///
/// // Fill a rectangle
/// let top_left = Coord::new(0, 0);
/// let bottom_right = Coord::new(100, 50);
/// framebuffer.fill_rectangle(&top_left, &bottom_right, &color)?;
/// ```
// #[gen_wrapper_fn(FRAME_BUFFER)]
pub trait DisplayDraw {
	/// The result type for drawing operations
	type Output = Rslt<(), GraphicError,>;

	/// Draws a single pixel at the specified coordinate with the given color
	///
	/// # Arguments
	///
	/// * `coord` - The coordinate where the pixel should be drawn
	/// * `color` - The color representation for the pixel
	///
	/// # Returns
	///
	/// * `Ok(())` - If the pixel was successfully drawn
	/// * `Err(GraphicError)` - If the coordinate is invalid or drawing fails
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let coord = Coord::new(100, 50);
	/// let color = Rgb::new(255, 0, 0); // Red pixel
	/// framebuffer.put_pixel(&coord, &color)?;
	/// ```
	fn put_pixel(
		&self,
		coord: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output;

	/// Fills a rectangular area with the specified color
	///
	/// This method fills all pixels within the rectangle defined by the
	/// top-left and bottom-right coordinates (inclusive) with the given color.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner coordinate of the rectangle
	/// * `right_bottom` - The bottom-right corner coordinate of the rectangle
	/// * `color` - The color to fill the rectangle with
	///
	/// # Returns
	///
	/// * `Ok(())` - If the rectangle was successfully filled
	/// * `Err(GraphicError::InvalidCoordinate)` - If the coordinates are
	///   invalid
	///
	/// # Coordinate Requirements
	///
	/// The coordinates must satisfy the following conditions:
	/// - `left_top.x < right_bottom.x && left_top.y < right_bottom.y`
	/// - `right_bottom.x <= framebuffer.width && right_bottom.y <=
	///   framebuffer.height`
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(10, 10);
	/// let bottom_right = Coord::new(50, 30);
	/// let color = Rgb::new(0, 255, 0); // Green rectangle
	/// framebuffer.fill_rectangle(&top_left, &bottom_right, &color)?;
	/// ```
	fn fill_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output;

	/// Draws the outline of a rectangle with the specified color
	///
	/// This method draws only the border of the rectangle, leaving the interior
	/// unchanged. The outline is drawn as a single-pixel-wide border.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner coordinate of the rectangle
	/// * `right_bottom` - The bottom-right corner coordinate of the rectangle
	/// * `color` - The color for the rectangle outline
	///
	/// # Returns
	///
	/// * `Ok(())` - If the outline was successfully drawn
	/// * `Err(GraphicError::InvalidCoordinate)` - If the coordinates are
	///   invalid
	///
	/// # Coordinate Requirements
	///
	/// Same requirements as `fill_rectangle()`.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(20, 20);
	/// let bottom_right = Coord::new(80, 60);
	/// let color = Rgb::new(0, 0, 255); // Blue outline
	/// framebuffer.outline_rectangle(&top_left, &bottom_right, &color)?;
	/// ```
	fn outline_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output;
}

/// A framebuffer structure that manages display memory and drawing operations
///
/// The `FrameBuffer` struct encapsulates all the necessary information and
/// functionality for managing a graphics framebuffer, including memory layout,
/// pixel format handling, and drawing operations. It is generic over the pixel
/// format type `P`.
///
/// # Type Parameters
///
/// * `P` - The pixel format type that implements the `PixelFormat` trait
///
/// # Fields
///
/// * `drawer` - The pixel format handler for color conversion and
///   representation
/// * `buf` - The base memory address of the framebuffer
/// * `size` - The total size of the framebuffer in bytes
/// * `width` - The width of the display in pixels
/// * `height` - The height of the display in pixels
/// * `stride` - The number of bytes per scanline (may include padding)
///
/// # Memory Layout
///
/// The framebuffer assumes a linear memory layout where pixels are stored
/// consecutively in memory. Each pixel occupies 4 bytes (32 bits), with the
/// actual color data using the first 3 bytes according to the pixel format.
///
/// # Examples
///
/// ```rust,ignore
/// use oso_kernel::base::graphic::{FrameBuffer, DisplayDraw};
/// use oso_kernel::base::graphic::color::Rgb;
///
/// // Create a new framebuffer with RGB pixel format
/// let framebuffer = FrameBuffer::new(Rgb);
///
/// // Initialize with actual hardware parameters
/// unsafe {
///     FrameBuffer::init(
///         &framebuffer,
///         0x1000_0000,  // Base address
///         1920 * 1080 * 4,  // Size in bytes
///         1920,  // Width
///         1080,  // Height
///         1920 * 4,  // Stride
///     );
/// }
/// ```
pub struct FrameBuffer<P: PixelFormat,> {
	/// The pixel format handler for color operations
	pub drawer: P,
	/// Base address of the framebuffer memory (as usize for arithmetic)
	pub buf:    usize,
	/// Total size of the framebuffer in bytes
	pub size:   usize,
	/// Display width in pixels
	pub width:  usize,
	/// Display height in pixels
	pub height: usize,
	/// Number of bytes per scanline (including any padding)
	pub stride: usize,
}

impl<P: PixelFormat,> FrameBuffer<P,> {
	/// Creates a new framebuffer instance with the specified pixel format
	///
	/// This constructor creates a framebuffer with default (zero) values for
	/// all memory-related fields. The framebuffer must be initialized with
	/// actual hardware parameters using the `init()` method before use.
	///
	/// # Arguments
	///
	/// * `pxl_fmt` - The pixel format handler to use for color operations
	///
	/// # Returns
	///
	/// A new `FrameBuffer` instance with default values
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use oso_kernel::base::graphic::FrameBuffer;
	/// use oso_kernel::base::graphic::color::Rgb;
	///
	/// let framebuffer = FrameBuffer::new(Rgb);
	/// // framebuffer now needs to be initialized with init() before use
	/// ```
	///
	/// # TODO
	///
	/// - Replace hardcoded configuration with actual hardware detection
	/// - Implement proper configuration structure
	pub fn new(/* conf: FrameBufConf, */ pxl_fmt: P,) -> Self {
		// TODO: Replace this placeholder with actual configuration
		struct A {
			base:   usize,
			width:  usize,
			height: usize,
			stride: usize,
			size:   usize,
		}

		let conf = A { base: 0, width: 0, height: 0, stride: 0, size: 0, };

		let buf = conf.base;
		let width = conf.width;
		let height = conf.height;
		let stride = conf.stride;
		let size = conf.size;

		Self { drawer: pxl_fmt, buf, width, height, stride, size, }
	}

	/// Initializes a framebuffer instance with hardware-specific parameters
	///
	/// This method provides interior mutability for static framebuffer
	/// instances by allowing modification of the framebuffer parameters after
	/// creation. It's typically called during kernel initialization when
	/// hardware parameters become available.
	///
	/// # Arguments
	///
	/// * `this` - Pointer to the framebuffer instance to initialize
	/// * `buf` - Base address of the framebuffer memory
	/// * `size` - Total size of the framebuffer in bytes
	/// * `width` - Display width in pixels
	/// * `height` - Display height in pixels
	/// * `stride` - Number of bytes per scanline
	///
	/// # Safety
	///
	/// This method is unsafe because:
	/// - It performs raw pointer manipulation to achieve interior mutability
	/// - It assumes the provided pointer is valid and properly aligned
	/// - It doesn't provide synchronization for concurrent access
	/// - The caller must ensure the memory parameters are valid
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use oso_kernel::base::graphic::{FrameBuffer, FRAME_BUFFER};
	///
	/// // Initialize the global framebuffer during kernel boot
	/// unsafe {
	///     FrameBuffer::init(
	///         &FRAME_BUFFER,
	///         0x1000_0000,      // Framebuffer base address from firmware
	///         1920 * 1080 * 4,  // Total framebuffer size
	///         1920,             // Screen width
	///         1080,             // Screen height
	///         1920 * 4,         // Bytes per scanline
	///     );
	/// }
	/// ```
	///
	/// # Panics
	///
	/// This method may panic if the provided parameters are inconsistent
	/// (e.g., size doesn't match width * height * bytes_per_pixel).
	pub unsafe fn init(
		this: *const Self,
		buf: usize,
		size: usize,
		width: usize,
		height: usize,
		stride: usize,
	) {
		unsafe {
			let this = this as *mut Self;
			(*this).buf = buf;
			(*this).size = size;
			(*this).width = width;
			(*this).height = height;
			(*this).stride = stride;
		}
	}

	/// Calculates the byte offset for a pixel at the given coordinate
	///
	/// This method converts 2D pixel coordinates to a linear byte offset
	/// within the framebuffer memory. It accounts for the stride (bytes per
	/// line) and assumes 4 bytes per pixel.
	///
	/// # Arguments
	///
	/// * `coord` - The coordinate to calculate the position for
	///
	/// # Returns
	///
	/// The byte offset from the framebuffer base address
	///
	/// # Formula
	///
	/// ```text
	/// offset = (stride * y + x) * 4
	/// ```
	///
	/// Where:
	/// - `stride` is the number of pixels per scanline (including padding)
	/// - `y` and `x` are the pixel coordinates
	/// - `4` is the number of bytes per pixel (32-bit pixels)
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let coord = Coord::new(100, 50);
	/// let offset = framebuffer.pos(&coord);
	/// // offset = (stride * 50 + 100) * 4
	/// ```
	fn pos(&self, coord: &impl Coordinal,) -> usize {
		// Each pixel is 4 bytes (32 bits), so multiply by 4
		(self.stride * coord.y() + coord.x()) * 4
	}

	/// Returns the coordinate of the bottom-right corner of the display
	///
	/// This utility method calculates the coordinate of the last valid pixel
	/// in the framebuffer, which is useful for bounds checking and drawing
	/// operations that need to know the display limits.
	///
	/// # Returns
	///
	/// A `Coord` representing the bottom-right corner pixel
	///
	/// # Formula
	///
	/// ```text
	/// Coord { x: width - 1, y: height - 1 }
	/// ```
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let bottom_right = framebuffer.right_bottom();
	/// // For a 1920x1080 display: Coord { x: 1919, y: 1079 }
	///
	/// // Useful for bounds checking
	/// if coord.x() <= bottom_right.x && coord.y() <= bottom_right.y {
	///     // Coordinate is within bounds
	/// }
	/// ```
	pub fn right_bottom(&self,) -> Coord {
		Coord { x: self.width - 1, y: self.height - 1, }
	}

	/// Creates a mutable slice to framebuffer memory at the specified position
	///
	/// This method provides safe access to framebuffer memory by creating a
	/// mutable slice at the given byte position with the specified length.
	/// It includes bounds checking to prevent buffer overruns.
	///
	/// # Arguments
	///
	/// * `pos` - The byte position within the framebuffer (will be multiplied
	///   by sizeof(u8))
	/// * `len` - The length of the slice in bytes
	///
	/// # Returns
	///
	/// A mutable slice to the framebuffer memory
	///
	/// # Panics
	///
	/// This method panics if:
	/// - `pos` is greater than or equal to `self.size`
	/// - The requested slice would extend beyond the framebuffer bounds
	///
	/// # Safety
	///
	/// While this method performs bounds checking, it still creates a raw slice
	/// from a memory address. The caller must ensure:
	/// - The framebuffer has been properly initialized
	/// - The memory region is valid and accessible
	/// - No other code is concurrently accessing the same memory region
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// // Get a 3-byte slice for RGB pixel data
	/// let pixel_data = framebuffer.slice_mut(pixel_offset, 3);
	/// pixel_data[0] = red_value;
	/// pixel_data[1] = green_value;
	/// pixel_data[2] = blue_value;
	/// ```
	pub fn slice_mut(&self, pos: usize, len: usize,) -> &mut [u8] {
		let pos = pos * size_of::<u8,>();
		assert!(self.size - pos > 0);

		let data_at_pos = self.buf + pos;
		unsafe { core::slice::from_raw_parts_mut(data_at_pos as *mut u8, len,) }
	}
}

impl<P: PixelFormat,> DisplayDraw for FrameBuffer<P,> {
	/// Draws a single pixel at the specified coordinate
	///
	/// This implementation writes the color data directly to the framebuffer
	/// memory at the calculated position. It converts the color to the
	/// appropriate pixel format and writes the RGB components.
	///
	/// # Arguments
	///
	/// * `coord` - The coordinate where the pixel should be drawn
	/// * `color` - The color representation to draw
	///
	/// # Returns
	///
	/// * `Ok(())` - If the pixel was successfully drawn
	/// * `Err(GraphicError)` - If an error occurs during drawing
	///
	/// # Implementation Details
	///
	/// 1. Calculates the byte position using the coordinate
	/// 2. Gets a mutable slice to the pixel memory (3 bytes for RGB)
	/// 3. Converts the color using the pixel format's color representation
	/// 4. Writes the RGB components to memory
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use oso_kernel::base::graphic::position::Coord;
	/// use oso_kernel::base::graphic::color::Rgb;
	///
	/// let coord = Coord::new(100, 50);
	/// let red_color = Rgb::new(255, 0, 0);
	/// framebuffer.put_pixel(&coord, &red_color)?;
	/// ```
	fn put_pixel(
		&self,
		coord: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output {
		let pos = self.pos(coord,);
		let pxl = self.slice_mut(pos, 3,);
		let color = self.drawer.color_repr(color,);
		pxl[0] = color[0];
		pxl[1] = color[1];
		pxl[2] = color[2];

		Ok((),)
	}

	/// Fills a rectangular area with the specified color
	///
	/// This implementation validates the coordinates and then iterates through
	/// all pixels within the rectangle, setting each one to the specified
	/// color. The color conversion is performed once before the loop for
	/// efficiency.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner of the rectangle
	/// * `right_bottom` - The bottom-right corner of the rectangle
	/// * `color` - The color to fill the rectangle with
	///
	/// # Returns
	///
	/// * `Ok(())` - If the rectangle was successfully filled
	/// * `Err(GraphicError::InvalidCoordinate)` - If coordinates are invalid
	///
	/// # Coordinate Validation
	///
	/// The method validates that:
	/// - `left_top.x() <= right_bottom.x()`
	/// - `left_top.y() <= right_bottom.y()`
	/// - `right_bottom.x() < self.width`
	/// - `right_bottom.y() < self.height`
	///
	/// # Performance Optimization
	///
	/// The color is converted to the pixel format representation once before
	/// the drawing loop to avoid repeated conversions for each pixel.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(10, 10);
	/// let bottom_right = Coord::new(50, 30);
	/// let blue_color = Rgb::new(0, 0, 255);
	/// framebuffer.fill_rectangle(&top_left, &bottom_right, &blue_color)?;
	/// ```
	fn fill_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output {
		// Validate coordinate bounds
		if left_top.x() > right_bottom.x()
			|| left_top.y() > right_bottom.y()
			|| right_bottom.x() > self.width
			|| right_bottom.y() > self.height
		{
			return Err(oso_err!(GraphicError::InvalidCoordinate),);
		}

		// Convert color once for performance optimization
		// This reduces pixel format determination to just once per rectangle
		let color = self.drawer.color_repr(color,);
		let mut coord = (left_top.x(), left_top.y(),);

		// Fill rectangle row by row
		for _ in left_top.y()..=right_bottom.y() {
			for _ in left_top.x()..=right_bottom.x() {
				let pos = self.pos(&coord,);
				let pxl = self.slice_mut(pos, 3,);
				pxl[0] = color[0];
				pxl[1] = color[1];
				pxl[2] = color[2];
				coord.0 += 1;
			}
			coord.1 += 1;
			coord.0 = left_top.x();
		}

		Ok((),)
	}

	/// Draws the outline of a rectangle with the specified color
	///
	/// This implementation draws a single-pixel-wide border around the
	/// rectangle defined by the coordinates. It draws four lines: top, right,
	/// bottom, and left.
	///
	/// # Arguments
	///
	/// * `left_top` - The top-left corner of the rectangle
	/// * `right_bottom` - The bottom-right corner of the rectangle
	/// * `color` - The color for the rectangle outline
	///
	/// # Returns
	///
	/// * `Ok(())` - If the outline was successfully drawn
	/// * `Err(GraphicError::InvalidCoordinate)` - If coordinates are invalid
	///
	/// # Drawing Algorithm
	///
	/// The outline is drawn in four phases:
	/// 1. **Top line**: From left_top to (right_bottom.x, left_top.y)
	/// 2. **Right line**: From (right_bottom.x, left_top.y) to right_bottom
	/// 3. **Bottom line**: From right_bottom to (left_top.x, right_bottom.y)
	/// 4. **Left line**: From (left_top.x, right_bottom.y) to left_top
	///
	/// # Performance Optimization
	///
	/// Like `fill_rectangle`, the color conversion is performed once before
	/// drawing to avoid repeated conversions.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let top_left = Coord::new(20, 20);
	/// let bottom_right = Coord::new(80, 60);
	/// let green_color = Rgb::new(0, 255, 0);
	/// framebuffer.outline_rectangle(&top_left, &bottom_right, &green_color)?;
	/// ```
	fn outline_rectangle(
		&self,
		left_top: &impl Coordinal,
		right_bottom: &impl Coordinal,
		color: &impl ColorRpr,
	) -> Self::Output {
		// Validate coordinate bounds
		if left_top.x() > right_bottom.x()
			|| left_top.y() > right_bottom.y()
			|| right_bottom.x() > self.width
			|| right_bottom.y() > self.height
		{
			return Err(oso_err!(GraphicError::InvalidCoordinate),);
		}

		let width = right_bottom.x() - left_top.x() - 1;
		let height = right_bottom.y() - left_top.y() - 1;

		// Convert color once for performance
		let color = self.drawer.color_repr(color,);
		let mut coord = (left_top.x(), left_top.y(),);

		// Draw top horizontal line
		for _ in 0..width {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.0 += 1;
		}

		// Draw right vertical line
		for _ in 0..height {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.1 += 1;
		}

		// Draw bottom horizontal line
		for _ in 0..width {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.0 -= 1;
		}

		// Draw left vertical line
		for _ in 0..height {
			let pos = self.pos(&coord,);
			let pxl = self.slice_mut(pos, 3,);
			pxl[0] = color[0];
			pxl[1] = color[1];
			pxl[2] = color[2];
			coord.1 -= 1;
		}

		Ok((),)
	}
}
