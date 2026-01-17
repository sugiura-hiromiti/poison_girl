#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(transparent)]
pub struct EventType(pub u32,);

impl EventType {
	pub const NOTIFY_SIGNAL: u32 = 0x0000_0200;
	pub const NOTIFY_WAIT: u32 = 0x0000_0100;
	pub const RUNTIME: u32 = 0x4000_0000;
	pub const SIGNAL_EXIT_BOOT_SERVICES: u32 = 0x00000201;
	pub const SIGNAL_VIRTUAL_ADDRESS_CHANGE: u32 = 0x6000_0202;
	pub const TIMER: u32 = 0x8000_0000;
}
