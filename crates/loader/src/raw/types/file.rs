use super::Boolean;
use super::Char16;
use super::Event;
use super::Status;
use super::time::Time;
use crate::c_style_enum;
use crate::chibi_uefi::protocol::Protocol;
use core::ffi::c_void;

c_style_enum! {
	pub enum OpenMode : u64 => {
		CREATE = 0x8000000000000000,
		READ = 0x1,
		WRITE = 0x2,
	}
}

c_style_enum! {
	pub enum FileAttributes: u64 => {
		ARCHIVE = 0x0000000000000020,
		DIRECTORY = 0x0000000000000010,
		HIDDEN = 0x0000000000000002,
		READ_ONLY = 0x0000000000000001,
		RESERVED = 0x0000000000000008,
		SYSTEM = 0x0000000000000004,
		VALID_ATTR = 0x0000000000000037,
	}
}

#[repr(C)]
pub struct FileIoToken {
	event:    Event,
	status:   Status,
	buf_size: usize,
	buf:      *mut c_void,
}

pub trait FileInformation: Protocol + Copy {}
impl FileInformation for FileInfo {}
impl FileInformation for FileSystemInfo {}

#[repr(C)]
#[derive(Clone, Copy, Debug,)]
pub struct FileInfo {
	pub size:             u64,
	pub file_size:        u64,
	pub physical_size:    u64,
	pub created_at:       Time,
	pub last_accessed_at: Time,
	pub modified_at:      Time,
	pub attr:             FileAttributes,
	/// must be null terminated
	pub file_name:        [Char16; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug,)]
pub struct FileSystemInfo {
	size:         u64,
	read_only:    Boolean,
	volume_size:  u64,
	free_space:   u64,
	block_size:   u32,
	volume_label: [Char16; 0],
}

#[repr(C)]
pub struct FileSystemVolumeLabel {
	volume_label: [Char16; 0],
}
