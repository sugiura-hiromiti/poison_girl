pub use draw::DisplayDraw;
use {
	super::{
		color::{self, PixFmtNew, PixelFormat},
		position::{Coord, Coordinal},
	},
	core::mem::size_of,
	poison_girl_macro::cfg_if,
	poison_girl_no_std_error::{
		GraphicError, PoisonGirlB, X, Y, poison_girl_err,
	},
};

mod draw;

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
	pub fn slice_mut(&self, pos: usize, len: usize,)
	-> PoisonGirlB<&mut [u8],>
	{
		let pos = pos * size_of::<u8,>();
		let Some(end,) = pos.checked_add(len,) else {
			return Y(poison_girl_err!(
				GraphicError::OverflowingFrameBufferAddress
			),);
		};
		if self.size < end {
			return Y(poison_girl_err!(
				GraphicError::OverflowingFrameBufferAddress
			),);
		}

		let data_at_pos = self.buf + pos;
		let mutable_slice = unsafe {
			core::slice::from_raw_parts_mut(data_at_pos as *mut u8, len,)
		};
		X(mutable_slice,)
	}
}

#[cfg(test)]
mod tests
{
	use {super::*, core::prelude::rust_2024::test};

	const UNCHANGED: u8 = 0xaa;

	fn framebuffer<P: PixelFormat,>(
		drawer: P,
		buf: &mut [u8],
		width: usize,
		height: usize,
		stride: usize,
	) -> FrameBuffer<P,>
	{
		FrameBuffer {
			drawer,
			buf: buf.as_mut_ptr() as usize,
			size: buf.len(),
			width,
			height,
			stride,
		}
	}

	fn pixel_offset(stride: usize, x: usize, y: usize,) -> usize
	{
		(stride * y + x) * 4
	}

	fn write_expected_pixel(
		buf: &mut [u8],
		stride: usize,
		x: usize,
		y: usize,
		color: [u8; 3],
	)
	{
		let offset = pixel_offset(stride, x, y,);
		buf[offset..offset + 3].copy_from_slice(&color,);
	}

	#[test]
	fn slice_mut_returns_slice_inside_bounds()
	{
		let mut buf = [0; 16];
		let fb = framebuffer(color::Rgb, &mut buf, 2, 2, 2,);

		match fb.slice_mut(4, 3,) {
			X(slice,) => {
				assert_eq!(&slice[..], &[0, 0, 0,]);
				slice.copy_from_slice(&[1, 2, 3,],);
			},
			Y(_,) => assert!(false),
		}

		assert_eq!(&buf[4..7], &[1, 2, 3,]);
	}

	#[test]
	fn slice_mut_returns_error_outside_bounds()
	{
		let mut buf = [0; 8];
		let fb = framebuffer(color::Rgb, &mut buf, 1, 2, 1,);

		assert!(matches!(fb.slice_mut(7, 2,), Y(_)));
		assert!(matches!(fb.slice_mut(usize::MAX, 1,), Y(_)));
	}

	#[test]
	fn put_pixel_writes_three_color_bytes_at_expected_offset()
	{
		let mut buf = [UNCHANGED; 32];
		let mut expected = buf;
		let fb = framebuffer(color::Rgb, &mut buf, 4, 2, 4,);

		assert!(matches!(
			fb.put_pixel(&(1, 1,), &(0x10, 0x20, 0x30,),),
			X((),)
		));
		write_expected_pixel(&mut expected, 4, 1, 1, [0x10, 0x20, 0x30,],);

		assert_eq!(buf, expected);
	}

	#[test]
	fn fill_rectangle_writes_inclusive_area()
	{
		let mut buf = [UNCHANGED; 48];
		let mut expected = buf;
		let fb = framebuffer(color::Rgb, &mut buf, 4, 3, 4,);

		assert!(matches!(
			fb.fill_rectangle(&(1, 0,), &(2, 1,), &(0x21, 0x32, 0x43,),),
			X((),)
		));
		for y in 0..=1 {
			for x in 1..=2 {
				write_expected_pixel(
					&mut expected,
					4,
					x,
					y,
					[0x21, 0x32, 0x43,],
				);
			}
		}

		assert_eq!(buf, expected);
	}

	#[test]
	fn fill_rectangle_rejects_invalid_coordinates()
	{
		let mut buf = [UNCHANGED; 48];
		let fb = framebuffer(color::Rgb, &mut buf, 4, 3, 4,);

		assert!(matches!(
			fb.fill_rectangle(&(2, 0,), &(1, 1,), &(1, 2, 3,),),
			Y(_,)
		));
		assert!(matches!(
			fb.fill_rectangle(&(0, 0,), &(5, 1,), &(1, 2, 3,),),
			Y(_,)
		));
		assert!(matches!(
			fb.fill_rectangle(&(0, 0,), &(1, 4,), &(1, 2, 3,),),
			Y(_,)
		));
	}

	#[test]
	fn outline_rectangle_writes_only_border_pixels()
	{
		let mut buf = [UNCHANGED; 100];
		let mut expected = buf;
		let fb = framebuffer(color::Rgb, &mut buf, 5, 5, 5,);

		assert!(matches!(
			fb.outline_rectangle(&(1, 1,), &(4, 4,), &(0x55, 0x66, 0x77,),),
			X((),)
		));
		for y in 1..=3 {
			for x in 1..=3 {
				if x == 1 || x == 3 || y == 1 || y == 3 {
					write_expected_pixel(
						&mut expected,
						5,
						x,
						y,
						[0x55, 0x66, 0x77,],
					);
				}
			}
		}

		assert_eq!(buf, expected);
	}
}
