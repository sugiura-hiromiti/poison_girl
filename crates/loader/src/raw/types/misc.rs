use crate::c_style_enum;

c_style_enum! {
	#[derive(Default)]
	pub enum ResetType: u32 => {
		COLD = 0,
		WARM = 1,
		SHUTDOWN = 2,
		PLATFORM_SPECIFIC = 3,
	}
}
