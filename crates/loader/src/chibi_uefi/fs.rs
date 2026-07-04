use {
	crate::{
		into_null_terminated_utf16,
		raw::{
			protocol::file::{FileProtocolV1, SimpleFileSystemProtocol},
			types::{
				Status,
				file::{FileAttributes, FileInfo, FileInformation, OpenMode},
			},
		},
	},
	alloc::vec::Vec,
	core::ptr::{self, NonNull},
	poison_girl_no_std_error::{PoisonGirlB, UefiError, X, Y, poison_girl_err},
};

impl SimpleFileSystemProtocol
{
	pub fn open_volume(&mut self,) -> PoisonGirlB<&mut FileProtocolV1,>
	{
		let mut root = ptr::null_mut();
		unsafe { (self.open_volume)(self, &mut root,) }.x_or()?;
		match unsafe { root.as_mut() } {
			Some(root,) => X(root,),
			None => Y(poison_girl_err!(UefiError::Custom(
				"root directory handle is null",
			)),),
		}
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		core::{ffi::c_void, ptr},
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	#[repr(C)]
	struct FakeSimpleFileSystem
	{
		protocol: SimpleFileSystemProtocol,
		root:     *mut FileProtocolV1,
	}

	impl FakeSimpleFileSystem
	{
		fn with_root(root: &mut FileProtocolV1,) -> Self
		{
			Self {
				protocol: SimpleFileSystemProtocol {
					revision:    0,
					open_volume: open_volume_from_fake,
				},
				root:     ptr::from_mut(root,),
			}
		}

		fn with_null_root() -> Self
		{
			Self {
				protocol: SimpleFileSystemProtocol {
					revision:    0,
					open_volume: open_volume_from_fake,
				},
				root:     ptr::null_mut(),
			}
		}

		fn protocol_mut(&mut self,) -> &mut SimpleFileSystemProtocol
		{
			&mut self.protocol
		}
	}

	unsafe extern "efiapi" fn open_volume_from_fake(
		this: *mut SimpleFileSystemProtocol,
		root: *mut *mut FileProtocolV1,
	) -> Status
	{
		let fs = this.cast::<FakeSimpleFileSystem>();
		unsafe {
			*root = (*fs).root;
		}
		Status::EFI_SUCCESS
	}

	unsafe extern "efiapi" fn open_volume_status_error(
		_this: *mut SimpleFileSystemProtocol,
		_root: *mut *mut FileProtocolV1,
	) -> Status
	{
		Status::EFI_DEVICE_ERROR
	}

	unsafe extern "efiapi" fn open_returns_self(
		this: *mut FileProtocolV1,
		new_handle: *mut *mut FileProtocolV1,
		_file_name: *const crate::raw::types::Char16,
		_open_mode: OpenMode,
		_attr: FileAttributes,
	) -> Status
	{
		unsafe {
			*new_handle = this;
		}
		Status::EFI_SUCCESS
	}

	unsafe extern "efiapi" fn open_returns_null(
		_this: *mut FileProtocolV1,
		new_handle: *mut *mut FileProtocolV1,
		_file_name: *const crate::raw::types::Char16,
		_open_mode: OpenMode,
		_attr: FileAttributes,
	) -> Status
	{
		unsafe {
			*new_handle = ptr::null_mut();
		}
		Status::EFI_SUCCESS
	}

	unsafe extern "efiapi" fn open_status_error(
		_this: *mut FileProtocolV1,
		_new_handle: *mut *mut FileProtocolV1,
		_file_name: *const crate::raw::types::Char16,
		_open_mode: OpenMode,
		_attr: FileAttributes,
	) -> Status
	{
		Status::EFI_NOT_FOUND
	}

	unsafe extern "efiapi" fn file_status(_this: *mut FileProtocolV1,)
	-> Status
	{
		Status::EFI_SUCCESS
	}

	unsafe extern "efiapi" fn file_read_write(
		_this: *mut FileProtocolV1,
		_buf_size: *mut usize,
		_buf: *mut c_void,
	) -> Status
	{
		Status::EFI_SUCCESS
	}

	unsafe extern "efiapi" fn file_get_position(
		_this: *const FileProtocolV1,
		position: *mut u64,
	) -> Status
	{
		unsafe {
			*position = 0;
		}
		Status::EFI_SUCCESS
	}

	unsafe extern "efiapi" fn file_set_position(
		_this: *mut FileProtocolV1,
		_position: u64,
	) -> Status
	{
		Status::EFI_SUCCESS
	}

	unsafe extern "efiapi" fn get_info_buffer_too_small(
		_this: *mut FileProtocolV1,
		_info_type: *const crate::raw::types::Guid,
		buf_size: *mut usize,
		_buf: *mut c_void,
	) -> Status
	{
		unsafe {
			*buf_size = 128;
		}
		Status::EFI_BUFFER_TOO_SMALL
	}

	unsafe extern "efiapi" fn get_info_status_error(
		_this: *mut FileProtocolV1,
		_info_type: *const crate::raw::types::Guid,
		_buf_size: *mut usize,
		_buf: *mut c_void,
	) -> Status
	{
		Status::EFI_DEVICE_ERROR
	}

	unsafe extern "efiapi" fn file_set_info(
		_this: *mut FileProtocolV1,
		_info_type: *const crate::raw::types::Guid,
		_buf_size: usize,
		_buf: *mut c_void,
	) -> Status
	{
		Status::EFI_SUCCESS
	}

	fn file_protocol(
		open: unsafe extern "efiapi" fn(
			*mut FileProtocolV1,
			*mut *mut FileProtocolV1,
			*const crate::raw::types::Char16,
			OpenMode,
			FileAttributes,
		) -> Status,
		get_info: unsafe extern "efiapi" fn(
			*mut FileProtocolV1,
			*const crate::raw::types::Guid,
			*mut usize,
			*mut c_void,
		) -> Status,
	) -> FileProtocolV1
	{
		FileProtocolV1 {
			revision: 0,
			open,
			close: file_status,
			delete: file_status,
			read: file_read_write,
			write: file_read_write,
			get_position: file_get_position,
			set_position: file_set_position,
			get_info,
			set_info: file_set_info,
			flush: file_status,
		}
	}

	#[test]
	fn open_volume_returns_non_null_root() -> PoisonGirlTestB
	{
		let mut root =
			file_protocol(open_returns_self, get_info_buffer_too_small,);
		let root_ptr = ptr::from_mut(&mut root,);
		let mut fs = FakeSimpleFileSystem::with_root(&mut root,);

		let opened = fs.protocol_mut().open_volume()?;

		assert_eq!(ptr::from_mut(opened,), root_ptr);
		success!()
	}

	#[test]
	fn open_volume_success_with_null_root_returns_error()
	{
		let mut fs = FakeSimpleFileSystem::with_null_root();

		assert!(matches!(fs.protocol_mut().open_volume(), Y(_)));
	}

	#[test]
	fn open_volume_status_error_propagates()
	{
		let mut fs = SimpleFileSystemProtocol {
			revision:    0,
			open_volume: open_volume_status_error,
		};

		assert!(matches!(fs.open_volume(), Y(_)));
	}

	#[test]
	fn file_open_returns_non_null_handle() -> PoisonGirlTestB
	{
		let mut file =
			file_protocol(open_returns_self, get_info_buffer_too_small,);
		let file_ptr = ptr::from_mut(&mut file,);

		let opened =
			file.open("kernel", OpenMode::READ, FileAttributes::ARCHIVE,)?;

		assert_eq!(ptr::from_mut(opened,), file_ptr);
		success!()
	}

	#[test]
	fn file_open_success_with_null_handle_returns_error()
	{
		let mut file =
			file_protocol(open_returns_null, get_info_buffer_too_small,);

		assert!(matches!(
			file.open("kernel", OpenMode::READ, FileAttributes::ARCHIVE,),
			Y(_)
		));
	}

	#[test]
	fn file_open_status_error_propagates()
	{
		let mut file =
			file_protocol(open_status_error, get_info_buffer_too_small,);

		assert!(matches!(
			file.open("kernel", OpenMode::READ, FileAttributes::ARCHIVE,),
			Y(_)
		));
	}

	#[test]
	fn info_size_accepts_buffer_too_small_probe() -> PoisonGirlTestB
	{
		let mut file =
			file_protocol(open_returns_self, get_info_buffer_too_small,);

		assert_eq!(file.info_size::<FileInfo>()?, 128);
		success!()
	}

	#[test]
	fn info_size_propagates_unexpected_status()
	{
		let mut file = file_protocol(open_returns_self, get_info_status_error,);

		assert!(matches!(file.info_size::<FileInfo>(), Y(_)));
	}
}

impl FileProtocolV1
{
	/// opens a new file relative to the source directory's location
	pub fn open(
		&mut self,
		path: impl AsRef<str,>,
		mode: OpenMode,
		attrs: FileAttributes,
	) -> PoisonGirlB<&mut FileProtocolV1,>
	{
		let path = into_null_terminated_utf16(path,);
		let path = path.as_ptr();

		let mut file = ptr::null_mut();

		unsafe { (self.open)(self, &mut file, path, mode, attrs,) }.x_or()?;
		match unsafe { file.as_mut() } {
			Some(file,) => X(file,),
			None => {
				Y(poison_girl_err!(UefiError::Custom("file handle is null",)),)
			},
		}
	}

	/// reads file content to buf
	///
	/// # Return
	///
	/// returns bytes amount of read data
	pub unsafe fn read(&mut self, buf: &mut [u8],) -> PoisonGirlB<usize,>
	{
		let mut len = buf.len();
		unsafe { (self.read)(self, &mut len, buf.as_mut_ptr().cast(),) }
			.x_or_with(|_| len,)
	}

	pub fn read_as_bytes(&mut self,) -> PoisonGirlB<Vec<u8,>,>
	{
		let file_info = self.get_file_info()?;
		let mut buf = alloc::vec![0; file_info.file_size as usize];
		let read_len = unsafe { self.read(buf.as_mut_slice(),) }?;
		assert_eq!(read_len, file_info.file_size as usize);
		X(buf,)
	}

	pub fn get_info<F: FileInformation,>(
		&mut self,
		buf: &mut [u8],
	) -> PoisonGirlB<*mut F,>
	{
		let mut len = buf.len();
		unsafe {
			(self.get_info)(self, &F::GUID, &mut len, buf.as_mut_ptr().cast(),)
		}
		.x_or()?;

		let file = NonNull::new(buf,)
			.ok_or(poison_girl_err!(UefiError::Custom(
				"file information is null",
			)),)
			.map(|s| s.as_ptr().cast(),)?;
		X(file,)
	}

	pub fn get_file_info(&mut self,) -> PoisonGirlB<FileInfo,>
	{
		let info_size = self.info_size::<FileInfo>()?;
		let mut buf = alloc::vec![0u8; info_size];
		let buf: &mut [u8] = buf.as_mut();
		let file_info = self.get_info(buf,)?;
		X(unsafe { *file_info },)
	}

	pub fn info_size<F: FileInformation,>(&mut self,) -> PoisonGirlB<usize,>
	{
		let mut len = 0;
		let status = unsafe {
			(self.get_info)(self, &F::GUID, &mut len, ptr::null_mut(),)
		};
		match status {
			Status::EFI_BUFFER_TOO_SMALL => X(len,),
			Status::EFI_SUCCESS => unreachable!(),
			_ => status.x_or_with(|_| 0,),
		}
	}
}
