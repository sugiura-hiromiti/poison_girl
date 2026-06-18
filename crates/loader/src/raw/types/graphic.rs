use {
	super::PhysicalAddress, crate::c_style_enum,
	poison_girl_no_std::bridge::graphic::PixelFormatConf,
};

#[repr(C)]
#[derive(Clone, Copy, Default,)]
pub struct GraphicsOutputModeInfo
{
	pub version:               u32,
	pub horizontal_resolution: u32,
	pub vertical_resolution:   u32,
	pub pixel_format:          GraphicsPixelFormat,
	pub pixel_info:            PixelBitMask,
	pub pixels_per_scal_line:  u32,
}

impl GraphicsOutputModeInfo
{
	pub fn resolution(&self,) -> (usize, usize,)
	{
		(self.horizontal_resolution as usize, self.vertical_resolution as usize,)
	}

	pub fn stride(&self,) -> usize
	{
		self.pixels_per_scal_line as usize
	}

	pub fn pixel_format(&self,) -> PixelFormatConf
	{
		use GraphicsPixelFormat as GPF;
		match self.pixel_format {
			GPF::RGB_RESERVED_8_BIT_PER_COLOR => PixelFormatConf::Rgb,
			GPF::BGR_RESERVED_8_BIT_PER_COLOR => PixelFormatConf::Bgr,
			GPF::PIXEL_BIT_MASK => PixelFormatConf::Bitmask,
			GPF::PIXEL_BLT_ONLY => PixelFormatConf::BltOnly,
			_ => todo!(),
		}
	}
}

#[repr(C)]
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash,)]
pub struct GraphicsOutputBltPixel
{
	pub blue:     u8,
	pub green:    u8,
	pub red:      u8,
	pub reserved: u8,
}

c_style_enum! {
	#[derive(Default)]
	pub enum GraphicsOutputBltOperation: u32 => {
		VIDEO_FILL = 0,
		VIDEO_TO_BLT_BUFFER = 1,
		BUFFER_TO_VIDEO = 2,
		VIDEO_TO_VIDEO = 3,
		GRAPHICS_OUTPUT_BLT_OPERATION_MAX = 4,
	}
}

c_style_enum! {
	#[derive(Default)]
	pub enum GraphicsPixelFormat: u32 => {
		RGB_RESERVED_8_BIT_PER_COLOR = 0,
		BGR_RESERVED_8_BIT_PER_COLOR = 1,
		PIXEL_BIT_MASK = 2,
		PIXEL_BLT_ONLY = 3,
		PIXEL_FORMAT_MAX = 4,
	}
}

#[repr(C)]
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash,)]
pub struct GraphicsOutputProtocolMode
{
	pub max_mode:          u32,
	pub mode:              u32,
	pub info:              *mut GraphicsOutputModeInfo,
	pub frame_buffer_base: PhysicalAddress,
	pub frame_buffer_size: usize,
}

impl GraphicsOutputProtocolMode
{
	pub fn info(&self,) -> &GraphicsOutputModeInfo
	{
		unsafe { &*self.info }
	}

	pub fn resolution(&self,) -> (usize, usize,)
	{
		self.info().resolution()
	}

	pub fn stride(&self,) -> usize
	{
		self.info().stride()
	}

	pub fn pixel_format(&self,) -> PixelFormatConf
	{
		self.info().pixel_format()
	}
}

#[repr(C)]
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash,)]
pub struct PixelBitMask
{
	pub red:      u32,
	pub green:    u32,
	pub blue:     u32,
	pub reserved: u32,
}

pub struct GraphicsOutputProtocolModes
{
	pub index:     u32,
	pub info_size: usize,
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		core::mem::{align_of, size_of},
		poison_girl_no_std::bridge::graphic::PixelFormatConf,
	};

	#[test]
	fn graphics_mode_info_layout_matches_uefi_abi()
	{
		assert_eq!(size_of::<GraphicsOutputModeInfo,>(), 36);
		assert_eq!(align_of::<GraphicsOutputModeInfo,>(), 4);
	}

	#[test]
	fn graphics_protocol_mode_layout_matches_uefi_abi()
	{
		assert_eq!(size_of::<GraphicsOutputProtocolMode,>(), 32);
		assert_eq!(align_of::<GraphicsOutputProtocolMode,>(), 8);
	}

	#[test]
	fn mode_info_converts_pixel_formats()
	{
		let mut info = GraphicsOutputModeInfo {
			horizontal_resolution: 800,
			vertical_resolution: 600,
			pixels_per_scal_line: 832,
			..Default::default()
		};

		info.pixel_format = GraphicsPixelFormat::RGB_RESERVED_8_BIT_PER_COLOR;
		assert_eq!(info.pixel_format(), PixelFormatConf::Rgb);
		info.pixel_format = GraphicsPixelFormat::BGR_RESERVED_8_BIT_PER_COLOR;
		assert_eq!(info.pixel_format(), PixelFormatConf::Bgr);
		info.pixel_format = GraphicsPixelFormat::PIXEL_BIT_MASK;
		assert_eq!(info.pixel_format(), PixelFormatConf::Bitmask);
		info.pixel_format = GraphicsPixelFormat::PIXEL_BLT_ONLY;
		assert_eq!(info.pixel_format(), PixelFormatConf::BltOnly);
		assert_eq!(info.resolution(), (800, 600));
		assert_eq!(info.stride(), 832);
	}
}
