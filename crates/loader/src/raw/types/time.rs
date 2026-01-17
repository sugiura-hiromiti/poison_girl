use crate::c_style_enum;

use super::Boolean;

#[repr(C)]
#[derive(Clone, Copy, Debug,)]
pub struct Time {
	year:        u16,
	month:       u8,
	day:         u8,
	hour:        u8,
	minute:      u8,
	second:      u8,
	pad1:        u8,
	nano_second: u32,
	time_zone:   u16,
	daylight:    u8,
	pad2:        u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(C)]
pub struct TimeCapabilities {
	resolution:   u32,
	accuracy:     u32,
	sets_to_zero: Boolean,
}

c_style_enum! {
	pub enum TimerDelay: i32 => {
		CANCEL = 0,
		PERIODIC = 1,
		RELATIVE = 2,
	}
}
