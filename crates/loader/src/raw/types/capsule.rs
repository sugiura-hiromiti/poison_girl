use super::Guid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(C)]
pub struct CapsuleHeader {
	pub capsule_guid:       Guid,
	pub header_size:        u32,
	pub flags:              CapsuleFlags,
	pub capsule_image_size: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(transparent)]
pub struct CapsuleFlags(pub u32,);
impl CapsuleFlags {
	/// Trigger a system reset after passing the capsule to the firmware.
	///
	/// If this flag is set, [`PERSIST_ACROSS_RESET`] must be set as well.
	///
	/// [`PERSIST_ACROSS_RESET`]: Self::PERSIST_ACROSS_RESET
	pub const INITIATE_RESET: u32 = 1 << 18;
	/// Indicates the firmware should process the capsule after system reset.
	pub const PERSIST_ACROSS_RESET: u32 = 1 << 16;
	/// Causes the contents of the capsule to be coalesced from the
	/// scatter-gather list into a contiguous buffer, and then a pointer to
	/// that buffer will be placed in the configuration table after system
	/// reset.
	///
	/// If this flag is set, [`PERSIST_ACROSS_RESET`] must be set as well.
	///
	/// [`PERSIST_ACROSS_RESET`]: Self::PERSIST_ACROSS_RESET
	pub const POPULATE_SYSTEM_TABLE: u32 = 1 << 17;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_0: u32 = 1 << 0;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_1: u32 = 1 << 1;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_10: u32 = 1 << 10;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_11: u32 = 1 << 11;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_12: u32 = 1 << 12;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_13: u32 = 1 << 13;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_14: u32 = 1 << 14;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_15: u32 = 1 << 15;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_2: u32 = 1 << 2;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_3: u32 = 1 << 3;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_4: u32 = 1 << 4;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_5: u32 = 1 << 5;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_6: u32 = 1 << 6;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_7: u32 = 1 << 7;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_8: u32 = 1 << 8;
	/// The meaning of this bit depends on the capsule GUID.
	pub const TYPE_SPECIFIC_BIT_9: u32 = 1 << 9;
}
