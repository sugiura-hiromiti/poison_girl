use crate::Rslt;
use crate::into_null_terminated_utf16;
use crate::raw::protocol::file::FileProtocolV1;
use crate::raw::protocol::file::SimpleFileSystemProtocol;
use crate::raw::types::Status;
use crate::raw::types::file::FileAttributes;
use crate::raw::types::file::FileInfo;
use crate::raw::types::file::FileInformation;
use crate::raw::types::file::OpenMode;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr;
use core::ptr::NonNull;
use oso_error::loader::UefiError;
use oso_error::oso_err;

impl SimpleFileSystemProtocol {
	pub fn open_volume(&mut self,) -> Rslt<&mut FileProtocolV1, UefiError,> {
		let mut root = ptr::null_mut();
		unsafe { (self.open_volume)(self, &mut root,) }.ok_or_with(|_| {
			unsafe { root.as_mut() }.expect("root directory handle is null",)
		},)
	}
}

impl FileProtocolV1 {
	/// opens a new file relative to the source directory's location
	pub fn open(
		&mut self,
		path: impl AsRef<str,>,
		mode: OpenMode,
		attrs: FileAttributes,
	) -> Rslt<&mut FileProtocolV1, UefiError,> {
		let path = into_null_terminated_utf16(path,);
		let path = path.as_ptr();

		let mut file = ptr::null_mut();

		let s = unsafe { (self.open)(self, &mut file, path, mode, attrs,) };
		s.ok_or_with(|_| {
			unsafe { file.as_mut() }.expect("file handle is null",)
		},)
	}

	/// reads file content to buf
	///
	/// # Return
	///
	/// returns bytes amount of read data
	/// # Safety
	/// TODO: fill doc comment
	pub unsafe fn read(&mut self, buf: &mut [u8],) -> Rslt<usize, UefiError,> {
		let mut len = buf.len();
		unsafe { (self.read)(self, &mut len, buf.as_mut_ptr().cast(),) }
			.ok_or_with(|_| len,)
	}

	pub fn read_as_bytes(&mut self,) -> Rslt<Vec<u8,>,> {
		let file_info = self.get_file_info()?;
		let mut buf = vec![0; file_info.file_size as usize];
		let read_len = unsafe { self.read(buf.as_mut_slice(),) }?;
		assert_eq!(read_len, file_info.file_size as usize);
		Ok(buf,)
	}

	pub fn get_info<F: FileInformation,>(
		&mut self,
		buf: &mut [u8],
	) -> Rslt<*mut F, UefiError,> {
		let mut len = buf.len();
		unsafe {
			(self.get_info)(self, &F::GUID, &mut len, buf.as_mut_ptr().cast(),)
		}
		.ok_or()?;

		NonNull::new(buf,)
			.ok_or(oso_err!(UefiError::Custom("file information is null")),)
			.map(|s| s.as_ptr().cast(),)
	}

	pub fn get_file_info(&mut self,) -> Rslt<FileInfo,> {
		let info_size = self.info_size::<FileInfo>()?;
		let mut buf = vec![0u8; info_size];
		let buf: &mut [u8] = buf.as_mut();
		let file_info = self.get_info(buf,)?;
		Ok(unsafe { *file_info },)
	}

	pub fn info_size<F: FileInformation,>(
		&mut self,
	) -> Rslt<usize, UefiError,> {
		let mut len = 0;
		let status = unsafe {
			(self.get_info)(self, &F::GUID, &mut len, ptr::null_mut(),)
		};
		match status {
			Status::EFI_BUFFER_TOO_SMALL => Ok(len,),
			Status::EFI_SUCCESS => unreachable!(),
			_ => status.ok_or_with(|_| 0,),
		}
	}
}
