use {
	crate::{
		chibi_uefi::{drop_uefi_cleanup_result, table::boot_services},
		raw::types::{
			Status,
			graphic::{
				GraphicsOutputBltOperation, GraphicsOutputBltPixel,
				GraphicsOutputModeInfo, GraphicsOutputProtocolMode,
			},
		},
	},
	poison_girl_no_std_error::{PoisonGirlB, X},
};

#[repr(C)]
pub struct GraphicsOutputProtocol
{
	pub query_mode: unsafe extern "efiapi" fn(
		*const Self,
		mode_number: u32,
		size_of_info: *mut usize,
		info: *mut *const GraphicsOutputModeInfo,
	) -> Status,
	pub set_mode:
		unsafe extern "efiapi" fn(*mut Self, mode_number: u32,) -> Status,
	pub blt: unsafe extern "efiapi" fn(
		*mut Self,
		blt_buffer: *mut GraphicsOutputBltPixel,
		blt_operation: GraphicsOutputBltOperation,
		source_x: usize,
		source_y: usize,
		dest_x: usize,
		dest_y: usize,
		width: usize,
		height: usize,
		delta: usize,
	) -> Status,
	pub mode:       *mut GraphicsOutputProtocolMode,
}

impl GraphicsOutputProtocol
{
	pub fn query_mode(&self, index: u32,) -> PoisonGirlB<(),>
	{
		let mut info_size = 0;
		let mut info_heap_ptr = core::ptr::null();
		unsafe {
			(self.query_mode)(self, index, &mut info_size, &mut info_heap_ptr,)
		}
		.x_or()?;

		if let Some(info_heap_ptr,) =
			unsafe { info_heap_ptr.cast::<u8>().cast_mut().as_mut() }
			&& let X(bs,) = boot_services()
		{
			drop_uefi_cleanup_result(bs.free_pool(info_heap_ptr,),);
		}

		X((),)
	}

	pub fn mode(&self,) -> &GraphicsOutputProtocolMode
	{
		unsafe { &*self.mode }
	}
}
