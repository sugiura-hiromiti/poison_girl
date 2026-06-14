#[derive(Debug,)]
pub struct StatusCode
{
	/// Success status codes (high bit clear)
	pub success: Vec<StatusCodeInfo,>,
	/// Error status codes (high bit set)
	pub error:   Vec<StatusCodeInfo,>,
	/// Warning status codes (high bit clear, but indicate warnings)
	pub warn:    Vec<StatusCodeInfo,>,
}

#[derive(Debug,)]
pub struct StatusCodeInfo
{
	/// The mnemonic name of the status code (e.g., "EFI_SUCCESS")
	pub mnemonic: String,
	/// The numeric value of the status code
	pub value:    usize,
	/// Human-readable description of what the status code means
	pub desc:     String,
}

impl StatusCodeInfo
{
	/// Bit mask for error status codes (high bit set)
	///
	/// UEFI error codes have the most significant bit set to 1,
	/// distinguishing them from success and warning codes.
	pub const ERROR_BIT: usize = 1 << (usize::BITS - 1);
}
