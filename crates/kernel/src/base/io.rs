use {
	super::graphic::FRAME_BUFFER,
	crate::base::graphic::position::Coordinal,
	core::{
		fmt::Write,
		ops::{Add, Div, Mul, Sub},
	},
	poison_girl_macro::{font, impl_int},
	poison_girl_no_std_error::{PoisonGirlB, X, Y},
};

pub const SINONOME: &[u128; 256] = font!("resource/sinonome_font.dat");
/// Maximum number of digits that can be represented in a u128
///
/// This constant is used for buffer sizing when converting integers to strings.
/// A u128 can have at most 39 decimal digits (2^128 - 1 =
/// 340282366920938463463374607431768211455).
pub const MAX_DIGIT: usize = 39;
/// Global console text buffer for kernel output
///
/// This static instance provides the primary console interface for the kernel.
/// It's initialized with a position of (0, 0) and uses 8x16 pixel characters.
///
/// # Safety
///
/// This static is accessed through unsafe operations in the `print` function
/// to provide interior mutability. Proper synchronization should be added
/// for multi-threaded environments.
static CONSOLE: TextBuf<(usize, usize,),> = TextBuf::new((0, 0,), 8, 16,);

/// Text buffer for managing character display and positioning
///
/// This struct handles the layout and rendering of text characters on the
/// screen. It maintains the current cursor position, handles line wrapping, and
/// manages the conversion from characters to pixel operations.
///
/// # Type Parameters
///
/// * `C` - Coordinate type that implements the [`Coordinal`] trait
///
/// # Fields
///
/// * `init_pos` - Initial position where text rendering begins
/// * `row` - Current row position (in character units)
/// * `col` - Current column position (in character units)
/// * `font_width` - Width of each character in pixels
/// * `font_height` - Height of each character in pixels
///
/// # Examples
///
/// ```rust,ignore
/// use oso_kernel::base::io::TextBuf;
///
/// // Create a text buffer at position (100, 50) with 8x16 characters
/// let mut text_buf = TextBuf::new((100, 50), 8, 16);
///
/// // Render some text
/// text_buf.put_char(b'H')?;
/// text_buf.put_char(b'i')?;
/// text_buf.put_char(b'\n')?; // New line
/// ```
pub struct TextBuf<C: Coordinal,>
{
	/// Initial position for text rendering
	init_pos:        C,
	/// Current row position (in character units)
	row:             usize,
	/// Current column position (in character units)
	col:             usize,
	/// Width of each character in pixels
	pub font_width:  usize,
	/// Height of each character in pixels
	pub font_height: usize,
}

impl<C: Coordinal,> TextBuf<C,>
{
	/// Creates a new text buffer with the specified parameters
	///
	/// This constructor initializes a text buffer at the given position with
	/// the specified font dimensions. The cursor starts at position (0, 0)
	/// relative to the initial position.
	///
	/// # Arguments
	///
	/// * `init_pos` - Initial position where text rendering begins
	/// * `font_width` - Width of each character in pixels
	/// * `font_height` - Height of each character in pixels
	///
	/// # Returns
	///
	/// A new `TextBuf` instance ready for text rendering
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// // Create a text buffer at the top-left corner
	/// let text_buf = TextBuf::new((0, 0), 8, 16);
	///
	/// // Create a text buffer at a specific position
	/// let text_buf = TextBuf::new((100, 200), 12, 20);
	/// ```
	pub const fn new(
		init_pos: C, font_width: usize, font_height: usize,
	) -> Self
	{
		Self { init_pos, row: 0, col: 0, font_width, font_height, }
	}

	/// Calculates the current row position in pixels
	///
	/// This method converts the current row position from character units
	/// to pixel coordinates by multiplying by the font height and adding
	/// the initial Y position.
	///
	/// # Returns
	///
	/// The current row position in pixels from the top of the screen
	///
	/// # Formula
	///
	/// ```text
	/// pixel_row = init_pos.y + (font_height * current_row)
	/// ```
	fn row_pixel(&self,) -> usize
	{
		self.init_pos.y() + self.font_height * self.row
	}

	/// Calculates the current column position in pixels
	///
	/// This method converts the current column position from character units
	/// to pixel coordinates by multiplying by the font width and adding
	/// the initial X position.
	///
	/// # Returns
	///
	/// The current column position in pixels from the left of the screen
	///
	/// # Formula
	///
	/// ```text
	/// pixel_col = init_pos.x + (font_width * current_col)
	/// ```
	fn col_pixel(&self,) -> usize
	{
		self.init_pos.x() + self.font_width * self.col
	}

	/// Resets the text buffer cursor to the initial position
	///
	/// This method clears the current cursor position, moving it back to
	/// row 0, column 0. This is typically used when the screen needs to
	/// be cleared or when text has reached the bottom of the display.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// text_buf.clear(); // Reset cursor to (0, 0)
	/// ```
	pub fn clear(&mut self,)
	{
		self.row = 0;
		self.col = 0;
	}

	/// Renders a single character at the current cursor position
	///
	/// This method handles the rendering of individual characters, including
	/// special characters like newline. It performs automatic line wrapping
	/// and screen clearing when necessary.
	///
	/// # Arguments
	///
	/// * `char` - The character to render (as a byte value)
	///
	/// # Returns
	///
	/// * `Ok(())` - Character was successfully rendered
	/// * `Err(...)` - Rendering error occurred
	///
	/// # Special Characters
	///
	/// - `\n` (0x0A): Moves cursor to the beginning of the next line
	/// - Other characters: Rendered using the bitmap font
	///
	/// # Automatic Behaviors
	///
	/// - **Line Wrapping**: When text reaches the right edge, cursor moves to
	///   next line
	/// - **Screen Clearing**: When text reaches the bottom, screen is cleared
	///   and cursor resets
	/// - **Cursor Advancement**: After each character, cursor moves to the next
	///   position
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// text_buf.put_char(b'A')?;     // Render 'A'
	/// text_buf.put_char(b'\n')?;    // New line
	/// text_buf.put_char(b'B')?;     // Render 'B' on next line
	/// ```
	///
	/// # TODO
	///
	/// - Re-enable pixel rendering (currently commented out)
	/// - Add color support for characters
	/// - Implement proper error handling for rendering failures
	fn put_char(&mut self, char: u8,) -> PoisonGirlB<(),>
	{
		// Handle newline character
		if char == b'\n' {
			self.row += 1;
			self.col = 0;
			return X((),);
		}

		// Check if we've reached the bottom of the screen
		if self.row * self.font_height >= FRAME_BUFFER.height {
			self.clear();
		}

		// Get the bitmap data for this character
		let font_data = SINONOME[char as usize];
		let col_pos = self.col_pixel();
		let row_pos = self.row_pixel();

		// Render each pixel of the character
		for i in 0..self.font_width {
			for j in 0..self.font_height {
				let flag = i + j * self.font_width;
				// Determine whether pixel at position (i, j) should be drawn
				let bit = font_data & (0b1 << flag);
				if bit != 0 {
					let _coord = (col_pos + i, row_pos + j,);
					// TODO: Re-enable pixel rendering
					// put_pixel(&coord, &"#000000",)?;
				}
			}
		}

		// Advance cursor position
		self.col += 1;

		// Check for line wrapping
		if self.col_pixel() + self.font_width >= FRAME_BUFFER.width {
			self.col = 0;
			self.row += 1;
		}

		X((),)
	}
}

impl<C: Coordinal,> Write for TextBuf<C,>
{
	/// Implements the `Write` trait for formatted output support
	///
	/// This implementation allows the text buffer to be used with Rust's
	/// formatting macros like `write!`, `writeln!`, etc. It processes
	/// each byte in the string and renders it as a character.
	///
	/// # Arguments
	///
	/// * `s` - String slice to write to the text buffer
	///
	/// # Returns
	///
	/// * `Ok(())` - String was successfully written
	/// * `Err(core::fmt::Error)` - Writing failed
	///
	/// # Implementation Details
	///
	/// The method iterates through each byte in the string and calls
	/// `put_char` for each one. This handles UTF-8 strings by processing
	/// them byte-by-byte, which works correctly for ASCII characters.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use core::fmt::Write;
	///
	/// write!(text_buf, "Hello, World!")?;
	/// writeln!(text_buf, "Value: {}", 42)?;
	/// ```
	fn write_str(&mut self, s: &str,) -> core::fmt::Result
	{
		for c in s.as_bytes() {
			if let Y(_,) = self.put_char(*c,) {
				return Err(core::fmt::Error,);
			}
		}
		Ok((),)
	}
}

/// Prints formatted text to the console with a newline
///
/// This macro provides a convenient way to output formatted text to the kernel
/// console. It automatically appends a newline character after the formatted
/// output.
///
/// # Arguments
///
/// * `$($arg:tt)*` - Format string and arguments (same as `std::println!`)
///
/// # Examples
///
/// ```rust,ignore
/// println!(); // Print just a newline
/// println!("Hello, World!"); // Print with newline
/// println!("Value: {}, Address: 0x{:x}", 42, 0x1000); // Formatted output
/// ```
///
/// # Implementation
///
/// The macro expands to a call to the `print!` macro with a newline appended:
///
/// ```rust,ignore
/// println!("Hello") // expands to:
/// print!("{}\n", format_args!("Hello"))
/// ```
#[macro_export]
macro_rules! println {
	() => {
		$crate::print!("\n");
	};
	($($arg:tt)*) => {
		$crate::print!("{}\n", format_args!($($arg)*));
	};
}

/// Prints formatted text to the console
///
/// This macro provides the primary interface for outputting text from the
/// kernel. It supports all of Rust's standard formatting options and directs
/// output to the global console text buffer.
///
/// # Arguments
///
/// * `$($arg:tt)*` - Format string and arguments (same as `std::print!`)
///
/// # Examples
///
/// ```rust,ignore
/// print!("Hello, World!");
/// print!("Number: {}", 42);
/// print!("Hex: 0x{:x}, Binary: 0b{:b}", 255, 255);
/// ```
///
/// # Implementation
///
/// The macro expands to a call to the `print` function with formatted
/// arguments:
///
/// ```rust,ignore
/// print!("Hello") // expands to:
/// crate::base::io::print(format_args!("Hello"))
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::base::io::print(format_args!($($arg)*));
    };
}

/// Low-level print function for console output
///
/// This function provides the underlying implementation for the `print!` and
/// `println!` macros. It writes formatted arguments to the global console
/// text buffer using unsafe operations to achieve interior mutability.
///
/// # Arguments
///
/// * `args` - Formatted arguments created by `format_args!`
///
/// # Safety
///
/// This function uses unsafe operations to obtain a mutable reference to the
/// static `CONSOLE` text buffer. This is necessary to provide interior
/// mutability for the global console state.
///
/// # Panics
///
/// This function will panic if:
/// - The console text buffer cannot be accessed
/// - Writing to the console fails
///
/// # Implementation Details
///
/// The function:
/// 1. Casts the static `CONSOLE` to a mutable pointer
/// 2. Converts the pointer back to a mutable reference
/// 3. Uses the `Write` trait to output the formatted arguments
/// 4. Panics if any step fails
///
/// # Examples
///
/// This function is typically not called directly, but through the macros:
///
/// ```rust,ignore
/// // These macros call print() internally:
/// print!("Hello");
/// println!("World");
/// ```
pub fn print(args: core::fmt::Arguments,)
{
	use core::fmt::Write;
	unsafe {
		// SAFETY: We're obtaining a mutable reference to the static CONSOLE
		// This is safe because:
		// 1. The CONSOLE is a valid static with a stable address
		// 2. We're not creating multiple mutable references simultaneously
		// 3. The operation is atomic (single-threaded kernel context)
		// TODO: Add proper synchronization for multi-threaded environments
		(&CONSOLE as *const TextBuf<(usize, usize,),>
			as *mut TextBuf<(usize, usize,),>)
			.as_mut()
			.unwrap()
			.write_fmt(args,)
	}
	.expect("unable to write to console",)
}

// TODO: Implement integer to string conversion macro
// This macro would provide efficient integer to string conversion
// for use in formatting operations.
//
// macro_rules! to_txt {
// 	(let $rslt:ident = $exp:expr) => {
// 		let mut ___original = $exp.clone();
// 		let mut ___num = [0; oso_kernel::base::text::MAX_DIGIT];
// 		let mut ___digits = $exp.digit_count();
//
// 		/// Handle negative numbers by adding '-' prefix
// 		for i in 0..___digits {
// 			___num[i] = ___original.shift_right() + b'0';
// 		}
//
// 		if $exp < 0 {
// 			___num[___digits] = b'-';
// 			___digits += 1;
// 		}
//
// 		let mut rslt = &mut ___num[..___digits];
// 		rslt.reverse();
//
// 		let $rslt = unsafe { core::str::from_utf8_unchecked(rslt,) };
// 	};
// }

/// Trait for integer types that can be converted to text representation
///
/// This trait provides methods for converting integer types to their string
/// representations. It's designed to work in `no_std` environments where
/// standard library formatting may not be available.
///
/// # Required Operations
///
/// The trait requires implementations of basic arithmetic operations:
/// - Addition, subtraction, multiplication, division
/// - Comparison operations (PartialOrd, Ord)
/// - Cloning capability
///
/// # Methods
///
/// - [`digit_count`]: Returns the number of digits in the integer
/// - [`nth_digit`]: Returns the nth digit of the integer
/// - [`shift_right`]: Removes and returns the rightmost digit
///
/// # Examples
///
/// ```rust,ignore
/// let num = 12345u32;
/// assert_eq!(num.digit_count(), 5);
/// assert_eq!(num.nth_digit(0), 1); // First digit
/// assert_eq!(num.nth_digit(4), 5); // Last digit
/// ```
pub trait Integer:
	Add<Output = Self,>
	+ Sub<Output = Self,>
	+ Mul<Output = Self,>
	+ Div<Output = Self,>
	+ PartialOrd
	+ Ord
	+ Clone
	+ Sized
{
	/// Returns the number of decimal digits in this integer
	///
	/// # Returns
	///
	/// The count of digits (e.g., 123 returns 3, 0 returns 1)
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// assert_eq!(0u32.digit_count(), 1);
	/// assert_eq!(123u32.digit_count(), 3);
	/// assert_eq!((-456i32).digit_count(), 3); // Sign not counted
	/// ```
	fn digit_count(&self,) -> usize;

	/// Returns the nth digit of this integer (0-indexed from left)
	///
	/// # Arguments
	///
	/// * `n` - Index of the digit to retrieve (0 = leftmost/most significant)
	///
	/// # Returns
	///
	/// The digit at position n as a byte value (b'0' to b'9')
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let num = 12345u32;
	/// assert_eq!(num.nth_digit(0), b'1');
	/// assert_eq!(num.nth_digit(2), b'3');
	/// assert_eq!(num.nth_digit(4), b'5');
	/// ```
	fn nth_digit(&self, n: usize,) -> u8;

	/// Removes and returns the rightmost (least significant) digit
	///
	/// This method modifies the integer by removing its rightmost digit
	/// and returns that digit as a byte value.
	///
	/// # Returns
	///
	/// The rightmost digit as a byte value (b'0' to b'9')
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let mut num = 12345u32;
	/// assert_eq!(num.shift_right(), b'5');
	/// assert_eq!(num, 1234); // Number is modified
	/// assert_eq!(num.shift_right(), b'4');
	/// assert_eq!(num, 123);
	/// ```
	fn shift_right(&mut self,) -> u8;
}

// Implements the Integer trait for common integer types
//
// This macro invocation generates `Integer` trait implementations for all
// standard integer types (both signed and unsigned, various sizes).
//
// # Supported Types
//
// - Unsigned: u8, u16, u32, u64, u128, usize
// - Signed: i8, i16, i32, i64, i128, isize
//
// The implementations are generated by the `impl_int!` procedural macro
// from the `oso_proc_macro` crate.
impl_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
