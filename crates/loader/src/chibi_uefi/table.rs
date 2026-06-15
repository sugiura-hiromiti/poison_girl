use {
	crate::raw::{
		service::{BootServices, RuntimeServices},
		table::SystemTable,
	},
	core::{
		ptr::NonNull,
		sync::atomic::{AtomicPtr, Ordering},
	},
	poison_girl_no_std_error::{PoisonGirlB, UefiError, X, Y, poison_girl_err},
};

static SYSTEM_TABLE: AtomicPtr<SystemTable,> =
	AtomicPtr::new(core::ptr::null_mut(),);

unsafe fn set_system_table(ptr: *const SystemTable,)
{
	SYSTEM_TABLE.store(ptr.cast_mut(), Ordering::Release,);
}

/// # Panic
///
/// if SYSTEM_TABLE is null after set, this fn panics
pub(crate) fn set_system_table_panicking(ptr: *const SystemTable,)
{
	assert!(!ptr.is_null());
	unsafe { set_system_table(ptr,) };
	assert!(!SYSTEM_TABLE.load(Ordering::Acquire).is_null());
}

pub fn system_table() -> PoisonGirlB<NonNull<SystemTable,>,>
{
	let p = SYSTEM_TABLE.load(Ordering::Acquire,);
	match NonNull::new(p,) {
		Some(table,) => X(table,),
		None => Y(poison_girl_err!(UefiError::Custom(
			"set_system_table has not been called",
		)),),
	}
}

/// # Panics
///
/// if boot_services is null, then panics
pub fn boot_services<'a,>() -> PoisonGirlB<&'a BootServices,>
{
	let syst = system_table()?;
	match unsafe { syst.as_ref().boot_services.as_ref() } {
		Some(services,) => X(services,),
		None => Y(poison_girl_err!(UefiError::Custom(
			"boot services table is null",
		)),),
	}
}

pub fn runtime_services<'a,>() -> PoisonGirlB<&'a RuntimeServices,>
{
	let syst = system_table()?;
	match unsafe { syst.as_ref().runtime_services.as_ref() } {
		Some(services,) => X(services,),
		None => Y(poison_girl_err!(UefiError::Custom(
			"runtime services table is null",
		)),),
	}
}
