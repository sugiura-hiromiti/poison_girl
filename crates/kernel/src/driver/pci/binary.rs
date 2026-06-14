use poison_girl_no_std_error::PoisonGirlB;

/// Generic binary data parser framework
///
/// This trait provides a framework for parsing binary data with support for
/// different endianness formats. It's designed to handle the binary nature
/// of device tree data while providing type-safe parsing operations.
///
/// # Type Parameters
///
/// * `IS_LITTLE_ENDIAN` - Compile-time constant indicating endianness
/// * `T` - Target type that implements [`BinaryParserTarget`]
///
/// # Endianness
///
/// Device trees are typically stored in big-endian format, so most
/// implementations will use `IS_LITTLE_ENDIAN = false`.
pub trait BinaryParser<const IS_LITTLE_ENDIAN: bool, T: BinaryParserTarget,>:
	Sized
{
	/// Returns true if the parser uses little-endian byte order
	///
	/// # Returns
	///
	/// `true` for little-endian, `false` for big-endian
	fn is_little_endian() -> bool
	{
		IS_LITTLE_ENDIAN
	}

	/// Returns true if the parser uses big-endian byte order
	///
	/// # Returns
	///
	/// `true` for big-endian, `false` for little-endian
	fn is_big_endian() -> bool
	{
		!IS_LITTLE_ENDIAN
	}

	/// Returns a raw pointer to the binary data being parsed
	///
	/// # Returns
	///
	/// Raw pointer to the start of the binary data
	///
	/// # Safety
	///
	/// The returned pointer must be valid for the lifetime of the parser
	/// and point to readable memory.
	fn raw(&self,) -> *const u8;

	/// Returns the current parsing position as a byte offset
	///
	/// # Returns
	///
	/// Current position in bytes from the start of the data
	fn cur_pos(&self,) -> usize;

	/// Sets the current parsing position to the specified byte offset
	///
	/// # Arguments
	///
	/// * `to` - New position in bytes from the start of the data
	///
	/// # Safety
	///
	/// The caller must ensure that `to` is within the bounds of the data.
	fn set_pos(&mut self, to: usize,);

	/// Advances the current parsing position by the specified number of bytes
	///
	/// # Arguments
	///
	/// * `by` - Number of bytes to advance the position
	///
	/// # Returns
	///
	/// A mutable reference to self for method chaining
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// parser.advance(4).advance(8); // Advance by 12 bytes total
	/// ```
	fn advance(&mut self, by: usize,) -> &mut Self
	{
		let cur_pos = self.cur_pos();
		self.set_pos(cur_pos + by,);
		self
	}

	/// Returns a byte slice at the specified offset and length
	///
	/// # Arguments
	///
	/// * `offset` - Byte offset from the start of the data
	/// * `len` - Length of the slice in bytes
	///
	/// # Returns
	///
	/// A byte slice containing the requested data
	///
	/// # Safety
	///
	/// The caller must ensure that `offset + len` is within the bounds of the
	/// data.
	fn bytes_of(&self, offset: usize, len: usize,) -> &[u8]
	{
		let raw = unsafe { self.raw().add(offset,) };
		unsafe { core::slice::from_raw_parts(raw, len,) }
	}

	/// Reads data at the current position and advances the parser
	///
	/// This method reads `T::DATA_SIZE` bytes from the current position,
	/// advances the parser position, and returns the byte slice.
	///
	/// # Returns
	///
	/// A byte slice containing the data that was read
	fn read_range(&mut self,) -> &[u8]
	{
		let cur_pos = self.cur_pos();
		self.set_pos(cur_pos + T::DATA_SIZE,);
		self.bytes_of(cur_pos, T::DATA_SIZE,)
	}

	/// Parses data at the current position and advances the parser
	///
	/// This method reads and parses data of type `T` from the current position,
	/// then advances the parser position for the next operation.
	///
	/// # Returns
	///
	/// * `Ok(T::Output)` - Successfully parsed data
	/// * `Err(...)` - Parsing error
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let value: u32 = parser.parse()?;
	/// let next_value: u32 = parser.parse()?; // Automatically advanced
	/// ```
	fn parse(&mut self,) -> PoisonGirlB<T::Output,>
	{
		let bytes = self.read_range();
		T::try_interpret(bytes,)
	}

	/// Parses data at the specified offset without advancing the parser
	///
	/// This method allows looking ahead in the data without changing the
	/// current parser position.
	///
	/// # Arguments
	///
	/// * `offset` - Byte offset from the start of the data
	///
	/// # Returns
	///
	/// * `Ok(T::Output)` - Successfully parsed data
	/// * `Err(...)` - Parsing error
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let future_value: u32 = parser.peek(16)?; // Look 16 bytes ahead
	/// let current_value: u32 = parser.parse()?; // Still at original position
	/// ```
	fn peek(&self, offset: usize,) -> PoisonGirlB<T::Output,>
	{
		let bytes = self.bytes_of(offset, T::DATA_SIZE,);
		T::try_interpret(bytes,)
	}
}

/// Target type for binary parsing operations
///
/// This trait defines how to interpret raw bytes as a specific type.
/// It provides the size information and conversion logic needed by
/// the binary parser framework.
pub trait BinaryParserTarget: Sized
{
	/// The output type after parsing (defaults to Self)
	type Output = Self;

	/// The size in bytes of the data to be parsed
	const DATA_SIZE: usize = size_of::<Self::Output,>();

	/// Attempts to interpret the given bytes as the target type
	///
	/// # Arguments
	///
	/// * `bytes` - Raw bytes to interpret (length will be `DATA_SIZE`)
	///
	/// # Returns
	///
	/// * `Ok(Self::Output)` - Successfully parsed value
	/// * `Err(...)` - Parsing error (invalid data, wrong endianness, etc.)
	fn try_interpret(bytes: &[u8],) -> PoisonGirlB<Self::Output,>;
}

/// Implementation of `BinaryParserTarget` for `usize`
///
/// This implementation allows parsing `usize` values from binary data.
/// The actual implementation is currently incomplete and marked as `todo!()`.
///
/// # TODO
///
/// - Implement proper endianness conversion
/// - Add bounds checking for the input bytes
/// - Handle different architectures (32-bit vs 64-bit)
impl BinaryParserTarget for usize
{
	/// Attempts to interpret bytes as a `usize` value
	///
	/// # Arguments
	///
	/// * `bytes` - Raw bytes to interpret (should be 4 or 8 bytes depending on
	///   architecture)
	///
	/// # Returns
	///
	/// * `Ok(usize)` - Successfully parsed value
	/// * `Err(...)` - Parsing error
	///
	/// # TODO
	///
	/// This method needs to be implemented with proper:
	/// - Endianness handling (big-endian for device trees)
	/// - Architecture-specific size handling
	/// - Error handling for invalid input
	fn try_interpret(_bytes: &[u8],) -> PoisonGirlB<Self::Output,>
	{
		todo!("Implement usize parsing with proper endianness conversion")
	}
}
