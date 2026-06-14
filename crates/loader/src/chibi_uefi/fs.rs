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
