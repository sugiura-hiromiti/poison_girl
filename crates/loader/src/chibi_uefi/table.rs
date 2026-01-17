use crate::raw::service::BootServices;
use crate::raw::service::RuntimeServices;
use crate::raw::table::SystemTable;
use core::ptr::NonNull;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering;

static SYSTEM_TABLE: AtomicPtr<SystemTable,> =
	AtomicPtr::new(core::ptr::null_mut(),);

unsafe fn set_system_table(ptr: *const SystemTable,) {
	SYSTEM_TABLE.store(ptr.cast_mut(), Ordering::Release,);
}

/// # Panic
///
/// if SYSTEM_TABLE is null after set, this fn panics
pub(crate) fn set_system_table_panicking(ptr: *const SystemTable,) {
	assert!(!ptr.is_null());
	unsafe { set_system_table(ptr,) };
	assert!(!SYSTEM_TABLE.load(Ordering::Acquire).is_null());
}

pub fn system_table() -> NonNull<SystemTable,> {
	let p = SYSTEM_TABLE.load(Ordering::Acquire,);
	NonNull::new(p,).expect("set_system_table has not been called",)
}

/// # Panics
///
/// if boot_services is null, then panics
pub fn boot_services<'a,>() -> &'a BootServices {
	let syst = system_table();
	unsafe { syst.as_ref().boot_services.as_ref() }.unwrap()
}

pub fn runtime_services<'a,>() -> &'a RuntimeServices {
	let syst = system_table();
	unsafe { syst.as_ref().runtime_services.as_ref() }.unwrap()
}
