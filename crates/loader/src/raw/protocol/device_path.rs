use crate::raw::types::protocol::DeviceSubType;
use crate::raw::types::protocol::DeviceType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(C)]
pub struct DevicePathProtocol {
	pub major_type: DeviceType,
	pub subtype:    DeviceSubType,
	pub length:     [u8; 2],
}
