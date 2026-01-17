use super::protocol::OpenProtocolInformationEntry;
use super::types::Boolean;
use super::types::Char16;
use super::types::Event;
use super::types::Guid;
use super::types::Header;
use super::types::PhysicalAddress;
use super::types::Status;
use super::types::Tpl;
use super::types::UnsafeHandle;
use super::types::capsule::CapsuleHeader;
use super::types::event::EventType;
use super::types::memory::AllocateType;
use super::types::memory::MemoryDescriptor;
use super::types::memory::MemoryType;
use super::types::misc::ResetType;
use super::types::time::Time;
use super::types::time::TimeCapabilities;
use super::types::time::TimerDelay;
use super::types::variable::VariableAttributes;
use crate::raw::protocol::device_path::DevicePathProtocol;
use crate::raw::types::protocol::InterfaceType;
use core::ffi::c_void;

#[repr(C)]
pub struct RuntimeServices {
	header: Header,

	// time services
	pub get_time: unsafe extern "efiapi" fn(
		time: *mut Time,
		*mut TimeCapabilities,
	) -> Status,
	pub set_time: unsafe extern "efiapi" fn(time: *const Time,) -> Status,
	pub get_wakeup_time: unsafe extern "efiapi" fn(
		enabled: *mut u8,
		pending: *mut u8,
		time: *mut Time,
	) -> Status,
	pub set_wakeup_time:
		unsafe extern "efiapi" fn(enable: u8, time: *const Time,) -> Status,

	// virtual memory services
	pub set_virtual_address_map: unsafe extern "efiapi" fn(
		memory_map_size: usize,
		descriptor_size: usize,
		descriptor_version: u32,
		virtual_map: *mut MemoryDescriptor,
	) -> Status,
	pub convert_pointer: unsafe extern "efiapi" fn(
		debug_dispositioon: usize,
		address: *mut *const c_void,
	) -> Status,

	// variable services
	pub get_variable: unsafe extern "efiapi" fn(
		variable_name: *const Char16,
		vendor_guid: *const Guid,
		attributes: *mut VariableAttributes,
		data_size: *mut usize,
		data: *mut u8,
	) -> Status,
	pub get_next_variable_name: unsafe extern "efiapi" fn(
		variable_name_size: *mut usize,
		variable_name: *mut u16,
		vendor_guid: *mut Guid,
	) -> Status,
	pub set_variable: unsafe extern "efiapi" fn(
		variable_name: *const Char16,
		vendor_guid: *const Guid,
		attributes: VariableAttributes,
		data_size: usize,
		data: *const u8,
	) -> Status,

	// miscellaneous
	pub get_next_high_monotonic_count:
		unsafe extern "efiapi" fn(count: *mut u32,) -> Status,
	pub reset_system: unsafe extern "efiapi" fn(
		reset_type: ResetType,
		reset_status: Status,
		data_size: usize,
		reset_data: *const u8,
	) -> !,

	// uefi 2.0 capsule services
	pub update_capsule: unsafe extern "efiapi" fn(
		capsule_header_array: *const *const CapsuleHeader,
		capsule_count: usize,
		scatter_gather_list: PhysicalAddress,
	) -> Status,
	pub query_capsule_capabilities: unsafe extern "efiapi" fn(
		capsule_heaader_array: *const *const CapsuleHeader,
		capsule_count: usize,
		maximum_capsule_size: *mut u64,
		reset_type: *mut ResetType,
	) -> Status,

	// miscellaneous uefi 2.0 services
	pub query_variable_info: unsafe extern "efiapi" fn(
		attributes: VariableAttributes,
		maximum_variable_storage_size: *mut u64,
		remaining_variable_storage_size: *mut u64,
		maximum_variable_size: *mut u64,
	) -> Status,
}

#[repr(C)]
pub struct BootServices {
	pub header: Header,

	// task priority services
	pub raise_tpl:   unsafe extern "efiapi" fn(new_tpl: Tpl,) -> Tpl,
	pub restore_tpl: unsafe extern "efiapi" fn(old_tpl: Tpl,),

	// memory services
	pub allocate_pages: unsafe extern "efiapi" fn(
		allocate_type: AllocateType,
		memory_type: MemoryType,
		pages: usize,
		memory: *mut PhysicalAddress,
	) -> Status,
	pub free_pages: unsafe extern "efiapi" fn(
		memory: PhysicalAddress,
		pages: usize,
	) -> Status,
	pub get_memory_map: unsafe extern "efiapi" fn(
		memory_map_size: *mut usize,
		memory_map: *mut MemoryDescriptor,
		map_key: *mut usize,
		descriptor_size: *mut usize,
		descriptor_version: *mut u32,
	) -> Status,
	pub allocate_pool: unsafe extern "efiapi" fn(
		pool_type: MemoryType,
		size: usize,
		buffer: *mut *mut u8,
	) -> Status,
	pub free_pool:      unsafe extern "efiapi" fn(buffer: *mut u8,) -> Status,

	// event & timer services
	pub create_event: unsafe extern "efiapi" fn(
		event_type: EventType,
		notify_tpl: Tpl,
		notify_function: Option<
			unsafe extern "efiapi" fn(event: Event, context: *mut c_void,),
		>,
		notify_context: *mut c_void,
		event: *mut Event,
	) -> Status,
	pub set_timer: unsafe extern "efiapi" fn(
		event: Event,
		time_type: TimerDelay,
		trigger_time: u64,
	) -> Status,
	pub wait_for_event: unsafe extern "efiapi" fn(
		number_of_events: usize,
		event: *mut Event,
		index: *mut usize,
	) -> Status,
	pub signal_event:   unsafe extern "efiapi" fn(event: Event,) -> Status,
	pub close_event:    unsafe extern "efiapi" fn(event: Event,) -> Status,
	pub check_event:    unsafe extern "efiapi" fn(event: Event,) -> Status,

	// protocol handler services
	pub install_protocol_interface: unsafe extern "efiapi" fn(
		handle: *mut UnsafeHandle,
		protocol: *const Guid,
		interface_type: InterfaceType,
		interface: *const c_void,
	) -> Status,
	pub reinstall_protocol_interface: unsafe extern "efiapi" fn(
		handle: *mut UnsafeHandle,
		protocol: *const Guid,
		old_interface: *const c_void,
		new_interface: *const c_void,
	) -> Status,
	pub uninstall_protocol_interface: unsafe extern "efiapi" fn(
		handle: UnsafeHandle,
		protocol: *const Guid,
		interface: *const c_void,
	) -> Status,
	pub handle_protocol: unsafe extern "efiapi" fn(
		handle: UnsafeHandle,
		protocol: *const Guid,
		interface: *mut *mut c_void,
	) -> Status,
	pub reserved:                     *mut c_void,
	pub register_protocol_notify: unsafe extern "efiapi" fn(
		protocol: *const Guid,
		event: Event,
		registration: *mut *const c_void,
	) -> Status,
	pub locate_handle: unsafe extern "efiapi" fn(
		search_type: i32,
		protocol: *const Guid,
		search_key: *const c_void,
		buffer_size: *mut usize,
		buffer: *mut UnsafeHandle,
	) -> Status,
	pub locate_device_path: unsafe extern "efiapi" fn(
		protocol: *const Guid,
		device_path: *mut *const DevicePathProtocol,
		device: *mut *mut c_void,
	) -> Status,
	pub install_configuration_table: unsafe extern "efiapi" fn(
		guid: *const Guid,
		table: *const c_void,
	) -> Status,

	// image services
	pub load_image: unsafe extern "efiapi" fn(
		boot_policy: Boolean,
		parent_image_handle: UnsafeHandle,
		device_path: *const DevicePathProtocol,
		source_buffer: *const u8,
		source_size: usize,
		image_handle: *mut UnsafeHandle,
	) -> Status,
	pub start_image: unsafe extern "efiapi" fn(
		image_handle: UnsafeHandle,
		exit_data_size: *mut usize,
		exit_data: *mut *mut Char16,
	) -> Status,
	pub exit: unsafe extern "efiapi" fn(
		image_handle: UnsafeHandle,
		exit_status: Status,
		exit_data_size: usize,
		exit_data: *mut Char16,
	) -> Status,
	pub unload_image:
		unsafe extern "efiapi" fn(image_handle: UnsafeHandle,) -> Status,
	pub exit_boot_services: unsafe extern "efiapi" fn(
		image_handle: UnsafeHandle,
		map_key: usize,
	) -> Status,

	// miscellaneous services
	pub get_next_monotonic_count:
		unsafe extern "efiapi" fn(count: *mut u64,) -> Status,
	pub stall: unsafe extern "efiapi" fn(micro_second: usize,) -> Status,
	pub set_watchdog_timer: unsafe extern "efiapi" fn(
		timeout: usize,
		watchdog_code: u64,
		data_size: usize,
		watchdog_data: *const u16,
	) -> Status,
	pub connect_controller: unsafe extern "efiapi" fn(
		controller_handle: UnsafeHandle,
		driver_image_handle: UnsafeHandle,
		remaining_device_path: *const DevicePathProtocol,
		recursive: Boolean,
	) -> Status,
	pub disconnect_controller: unsafe extern "efiapi" fn(
		controller_handle: UnsafeHandle,
		driver_image_handle: UnsafeHandle,
		child_handle: UnsafeHandle,
	) -> Status,

	// open and close protocol services
	pub open_protocol: unsafe extern "efiapi" fn(
		handle: UnsafeHandle,
		protocol: *const Guid,
		interface: *mut *mut c_void,
		agent_handle: UnsafeHandle,
		controller_handle: UnsafeHandle,
		attributes: u32,
	) -> Status,
	pub close_protocol: unsafe extern "efiapi" fn(
		handle: UnsafeHandle,
		protocol: *const Guid,
		agent_handle: UnsafeHandle,
		controller_handle: UnsafeHandle,
	) -> Status,
	pub open_protocol_information: unsafe extern "efiapi" fn(
		handle: UnsafeHandle,
		protocol: *const Guid,
		entry_buffer: *mut *const OpenProtocolInformationEntry,
		entry_count: *mut usize,
	) -> Status,

	// library services
	pub protocols_per_handle: unsafe extern "efiapi" fn(
		handle: UnsafeHandle,
		protocol_buffer: *mut *mut *const Guid,
		protocol_buffer_count: *mut usize,
	) -> Status,
	pub locate_handle_buffer: unsafe extern "efiapi" fn(
		search_type: i32,
		protocol: *const Guid,
		search_key: *const c_void,
		handles_count: *mut usize,
		buffer: *mut *mut UnsafeHandle,
	) -> Status,
	pub locate_protocol: unsafe extern "efiapi" fn(
		protocol: *const Guid,
		registration: *mut c_void,
		interface: *mut *mut c_void,
	) -> Status,
	pub install_multiple_protocol_interfaces:
		unsafe extern "C" fn(handle: *mut UnsafeHandle, ...) -> Status,
	pub uninstall_multiple_protocol_interfaces:
		unsafe extern "C" fn(handle: UnsafeHandle, ...) -> Status,

	// crc services
	pub calculate_crc32: unsafe extern "efiapi" fn(
		data: *const c_void,
		data_size: usize,
		crc32: *mut u32,
	) -> Status,

	// misc services
	pub copy_mem: unsafe extern "efiapi" fn(
		dest: *mut u8,
		source: *const u8,
		len: usize,
	),
	pub set_mem:
		unsafe extern "efiapi" fn(buf: *mut u8, size: usize, value: u8,),
	pub create_event_ex: unsafe extern "efiapi" fn(
		event_type: EventType,
		notify_tpl: Tpl,
		notify_function: Option<
			unsafe extern "efiapi" fn(event: Event, context: *mut c_void,),
		>,
		notify_context: *mut c_void,
		event_group: *mut Guid,
		event: *mut Event,
	) -> Status,
}
