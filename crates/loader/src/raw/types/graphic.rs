use oso_no_std_shared::bridge::graphic::PixelFormatConf;

use crate::c_style_enum;

use super::PhysicalAddress;

#[repr(C)]
#[derive(Clone, Copy, Default,)]
pub struct GraphicsOutputModeInfo {
	pub version:               u32,
	pub horizontal_resolution: u32,
	pub vertical_resolution:   u32,
	pub pixel_format:          GraphicsPixelFormat,
	pub pixel_info:            PixelBitMask,
	pub pixels_per_scal_line:  u32,
}

impl GraphicsOutputModeInfo {
	pub fn resolution(&self,) -> (usize, usize,) {
		(self.horizontal_resolution as usize, self.vertical_resolution as usize,)
	}

	pub fn stride(&self,) -> usize {
		self.pixels_per_scal_line as usize
	}

	pub fn pixel_format(&self,) -> PixelFormatConf {
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
pub struct GraphicsOutputBltPixel {
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
pub struct GraphicsOutputProtocolMode {
	pub max_mode:          u32,
	pub mode:              u32,
	pub info:              *mut GraphicsOutputModeInfo,
	pub frame_buffer_base: PhysicalAddress,
	pub frame_buffer_size: usize,
}

impl GraphicsOutputProtocolMode {
	pub fn info(&self,) -> &GraphicsOutputModeInfo {
		unsafe { &*self.info }
	}

	pub fn resolution(&self,) -> (usize, usize,) {
		self.info().resolution()
	}

	pub fn stride(&self,) -> usize {
		self.info().stride()
	}

	pub fn pixel_format(&self,) -> PixelFormatConf {
		self.info().pixel_format()
	}
}

#[repr(C)]
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash,)]
pub struct PixelBitMask {
	pub red:      u32,
	pub green:    u32,
	pub blue:     u32,
	pub reserved: u32,
}

pub struct GraphicsOutputProtocolModes {
	pub index:     u32,
	pub info_size: usize,
}
