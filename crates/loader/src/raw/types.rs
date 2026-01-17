//! uefi implementation

use crate::Rslt;
use crate::c_style_enum;
use core::ffi::c_void;
use oso_error::loader::UefiError;

pub mod capsule;
pub mod event;
pub mod file;
pub mod graphic;
pub mod memory;
pub mod misc;
pub mod protocol;
pub mod text;
pub mod time;
pub mod util;
pub mod variable;

pub type UnsafeHandle = *mut c_void;
pub type Event = *mut c_void;
pub type Char8 = u8;
pub type Char16 = u16;
pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

#[repr(C)]
pub struct Header {
	signature: u64,
	revision:  u32,
	size:      u32,
	crc32:     u32,
	reserved:  u32,
}

c_style_enum! {
	/// task priority level
	pub enum Tpl: usize => {
		APPLICATION = 4,
		CALLBACK    = 8,
		NOTIFY      = 16,
		HIGH_LEVEL  = 31,
	}
}

/// abi compatible uefi boolean
/// 0 is false,
/// others are true
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
pub struct Boolean(pub u8,);

impl Boolean {
	pub const FALSE: Self = Self(0,);
	pub const TRUE: Self = Self(1,);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(C)]
pub struct Guid {
	pub time_low:                    u32,
	pub time_mid:                    [u8; 2],
	pub time_high_and_version:       [u8; 2],
	pub clock_seq_high_and_reserved: u8,
	pub clock_seq_low:               u8,
	pub node:                        [u8; 6],
}

impl Guid {
	pub const fn new(
		time_low: [u8; 4],
		time_mid: [u8; 2],
		time_high_and_version: [u8; 2],
		clock_seq_high_and_reserved: u8,
		clock_seq_low: u8,
		node: [u8; 6],
	) -> Self {
		let time_low = u32::from_ne_bytes([
			time_low[0],
			time_low[1],
			time_low[2],
			time_low[3],
		],);
		Self {
			time_low,
			time_mid: [time_mid[0], time_mid[1],],
			time_high_and_version: [
				time_high_and_version[0],
				time_high_and_version[1],
			],
			clock_seq_high_and_reserved,
			clock_seq_low,
			node,
		}
	}
}

oso_proc_macro::status!(2.11);

// #[repr(usize)]
// #[derive(Eq, PartialEq, Clone, Debug,)]
// pub enum Status {}
