use {
	super::Handle,
	crate::raw::{
		protocol::device_path::DevicePathProtocol,
		service::BootServices,
		types::{Boolean, Status, UnsafeHandle},
	},
	core::ptr,
};

impl BootServices
{
	/// # Safety
	/// TODO: fill doc comment
	pub unsafe fn connect_controller(
		&self,
		controller_handle: UnsafeHandle,
		driver_image_handle: Option<Handle,>,
		remaining_device_path: Option<DevicePathProtocol,>,
		recursive: Boolean,
	) -> Status
	{
		let driver_image_handle = match driver_image_handle {
			Some(h,) => h.as_ptr(),
			None => ptr::null_mut(),
		};
		let remaining_device_path = match remaining_device_path {
			Some(dpp,) => &dpp as *const _,
			None => ptr::null(),
		};

		unsafe {
			(self.connect_controller)(
				controller_handle,
				driver_image_handle,
				remaining_device_path,
				recursive,
			)
		}
	}
}
