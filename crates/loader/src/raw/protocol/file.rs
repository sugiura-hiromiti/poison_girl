use crate::Status;

mod ops;

use ops::{
	FileClose, FileDelete, FileFlush, FileFlushEx, FileGetInfo,
	FileGetPosition, FileOpen, FileOpenEx, FileRead, FileReadEx, FileSetInfo,
	FileSetPosition, FileWrite, FileWriteEx,
};

#[repr(C)]
pub struct SimpleFileSystemProtocol
{
	pub revision:    u64,
	pub open_volume: unsafe extern "efiapi" fn(
		this: *mut Self,
		root: *mut *mut FileProtocolV1,
	) -> Status,
}

#[repr(C)]
pub struct FileProtocolV1
{
	pub revision:     u64,
	pub open:         FileOpen,
	pub close:        FileClose,
	pub delete:       FileDelete,
	pub read:         FileRead,
	pub write:        FileWrite,
	pub get_position: FileGetPosition,
	pub set_position: FileSetPosition,
	pub get_info:     FileGetInfo,
	pub set_info:     FileSetInfo,
	pub flush:        FileFlush,
}

#[repr(C)]
pub struct FileProtocolV2
{
	pub v1:    FileProtocolV1,
	pub open:  FileOpenEx,
	pub read:  FileReadEx,
	pub write: FileWriteEx,
	pub flush: FileFlushEx,
}
