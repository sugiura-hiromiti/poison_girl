use oso_error::loader::UefiError;

use super::service::BootServices;
use super::service::RuntimeServices;
use super::types::Char16;
use super::types::Guid;
use super::types::Header;
use super::types::UnsafeHandle;
use crate::Rslt;
use crate::guid;
use crate::raw::protocol::text::TextInputProtocol;
use crate::raw::protocol::text::TextOutputProtocol;
use core::ffi::c_void;
use core::ptr::NonNull;

#[repr(C)]
pub struct SystemTable {
	pub header: Header,

	pub firmware_vendor:   *const Char16,
	pub firmware_revision: u32,

	pub stdin_handle: UnsafeHandle,
	pub stdin:        *mut TextInputProtocol,

	pub stdout_handle: UnsafeHandle,
	pub stdout:        *mut TextOutputProtocol,

	pub stderr_handle: UnsafeHandle,
	pub stderr:        *mut TextOutputProtocol,

	pub runtime_services: *mut RuntimeServices,
	pub boot_services:    *mut BootServices,

	pub config_table_count: usize,
	pub config_tables:      *mut ConfigTable,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy,)]
#[repr(C)]
pub struct ConfigTable {
	vendor_guid:  Guid,
	vendor_table: *mut c_void,
}

pub struct ConfigTableStream {
	max_index:     usize,
	config_tables: Option<NonNull<ConfigTable,>,>,
}

impl ConfigTableStream {
	fn config_table_with(&self, guid: Guid,) -> Option<NonNull<ConfigTable,>,> {
		let config_tables = self.config_tables?;

		for i in 0..self.max_index {
			let target_config_table = unsafe { config_tables.as_ptr().add(i,) };

			let config_table = unsafe {
				// `config_tables.as_ptr().
				// add(i)`は本来nullではないはずなのでunsafeしても良さげ
				NonNull::new_unchecked(target_config_table,)
			};

			if unsafe { config_table.as_ref() }.vendor_guid == guid {
				return Some(config_table,);
			}
		}

		None
	}
}

pub const DEVICE_TREE_TABLE_GUID: Guid =
	guid!("b1b621d5-f19c-41a5-830b-d9152c69aae0");

impl SystemTable {
	pub fn get_config_tables(&self,) -> Rslt<ConfigTableStream, UefiError,> {
		let config_tables = NonNull::new(self.config_tables,);
		Ok(ConfigTableStream {
			max_index: self.config_table_count,
			config_tables,
		},)
	}

	pub fn config_table_with(
		&self,
		guid: Guid,
	) -> Rslt<Option<NonNull<ConfigTable,>,>, UefiError,> {
		Ok(self.get_config_tables()?.config_table_with(guid,),)
	}

	pub fn device_tree(
		&self,
	) -> Rslt<Option<NonNull<ConfigTable,>,>, UefiError,> {
		self.config_table_with(DEVICE_TREE_TABLE_GUID,)
	}
}
