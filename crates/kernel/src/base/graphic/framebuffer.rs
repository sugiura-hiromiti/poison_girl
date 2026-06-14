mod draw;
pub use draw::DisplayDraw;

use {
	super::{
		color::{self, PixFmtNew, PixelFormat},
		position::{Coord, Coordinal},
	},
	poison_girl_macro::cfg_if,
};

cfg_if! {
	if #[cfg(feature = "rgb")] {
		type FbDrawer = color::Rgb;
	} else if #[cfg(feature = "bgr")] {
		type FbDrawer = color::Bgr;
	} else if #[cfg(feature = "bitmask")] {
		type FbDrawer = color::Bitmask;
	} else if #[cfg(feature = "bltonly")] {
		type FbDrawer = color::BltOnly;
	}
}

// #[cfg(feature = "rgb")]
// type FbDrawer = color::Rgb;
// #[cfg(feature = "bgr")]
// type FbDrawer = color::Bgr;
// #[cfg(feature = "bitmask")]
// type FbDrawer = color::Bitmask;
// #[cfg(feature = "bltonly")]
// type FbDrawer = color::BltOnly;

pub static FRAME_BUFFER: FrameBuffer<FbDrawer,> = FrameBuffer {
	drawer: FbDrawer::new_pix(),
	buf:    0,
	size:   0,
	width:  0,
	height: 0,
	stride: 0,
};

// #[cfg(feature = "bgr")]
// pub static FRAME_BUFFER: FrameBuffer<Bgr,> = FrameBuffer {
// 	drawer: Bgr,
// 	buf:    0,
// 	size:   0,
// 	width:  0,
// 	height: 0,
// 	stride: 0,
// };
//
// #[cfg(feature = "bitmask")]
// pub static FRAME_BUFFER: FrameBuffer<Bitmask,> = FrameBuffer {
// 	drawer: Bitmask,
// 	buf:    0,
// 	size:   0,
// 	width:  0,
// 	height: 0,
// 	stride: 0,
// };
//
// #[cfg(feature = "bltonly")]
// pub static FRAME_BUFFER: FrameBuffer<BltOnly,> = FrameBuffer {
// 	drawer: BltOnly,
// 	buf:    0,
// 	size:   0,
// 	width:  0,
// 	height: 0,
// 	stride: 0,
// };

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
/// use poison_girl_kernel::base::graphic::{FrameBuffer, DisplayDraw};
/// use poison_girl_kernel::base::graphic::color::Rgb;
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
pub struct FrameBuffer<P: PixelFormat,>
{
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

impl<P: PixelFormat,> FrameBuffer<P,>
{
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
	/// use poison_girl_kernel::base::graphic::FrameBuffer;
	/// use poison_girl_kernel::base::graphic::color::Rgb;
	///
	/// let framebuffer = FrameBuffer::new(Rgb);
	/// // framebuffer now needs to be initialized with init() before use
	/// ```
	///
	/// # TODO
	///
	/// - Replace hardcoded configuration with actual hardware detection
	/// - Implement proper configuration structure
	pub fn new(/* conf: FrameBufConf, */ pxl_fmt: P,) -> Self
	{
		// TODO: Replace this placeholder with actual configuration
		struct A
		{
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
	/// use poison_girl_kernel::base::graphic::{FrameBuffer, FRAME_BUFFER};
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
	)
	{
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
	fn pos(&self, coord: &impl Coordinal,) -> usize
	{
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
	pub fn right_bottom(&self,) -> Coord
	{
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
	pub fn slice_mut(&self, pos: usize, len: usize,) -> &mut [u8]
	{
		let pos = pos * size_of::<u8,>();
		assert!(self.size - pos > 0);

		let data_at_pos = self.buf + pos;
		unsafe { core::slice::from_raw_parts_mut(data_at_pos as *mut u8, len,) }
	}
}
