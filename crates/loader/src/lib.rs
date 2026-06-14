#![no_std]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(derive_const)]
#![feature(const_default)]
#![feature(string_from_utf8_lossy_owned)]
#![feature(iterator_try_collect)]
// #![feature(nonzero_internals)]
//#![feature(stdarch_arm_hints)]

extern crate alloc;
#[cfg(test)] extern crate std;

use {
	crate::{chibi_uefi::table::system_table, raw::table::ConfigTable},
	alloc::vec::Vec,
	chibi_uefi::{protocol::HandleSearchType, table::boot_services},
	core::ptr::NonNull,
	poison_girl_macro::cfg_if,
	poison_girl_no_std::{
		bridge::device_tree::DeviceTreeAddress, idle_cpu_forever,
	},
	poison_girl_no_std_error::{PoisonGirlB, UefiError, X, Y, poison_girl_err},
	raw::{
		table::SystemTable,
		types::{Status, UnsafeHandle},
	},
};

/// UEFI interface wrapper providing simplified access to UEFI services
pub mod chibi_uefi;
/// ELF file parsing and loading functionality
pub mod elf;
/// Kernel and graphics loading utilities
pub mod load;
/// Raw UEFI types and protocol definitions
pub mod raw;

#[macro_export]
macro_rules! on_error {
	($e:ident, $situation:expr) => {{
		log::error!("error happen {}", $situation);
		log::error!("error msg:");
		log::error!("{}", $e);
	}};
}

pub fn init(
	image_handle: UnsafeHandle,
	syst: *const SystemTable,
) -> PoisonGirlB<(),>
{
	// Clear console output for clean startup
	clear_console(syst,)?;

	// Initialize UEFI table access
	chibi_uefi::table::set_system_table_panicking(syst,);
	chibi_uefi::set_image_handle_panicking(image_handle,);

	// Connect all available devices
	let bs = boot_services()?;

	// UEFI only installs DevicePathProtocol on devices that are fully connected
	// `AllHandles` is the only way to find unconnected devices
	let handles =
		unsafe { bs.locate_handle_buffer(HandleSearchType::AllHandles,)? };

	for handle in handles {
		unsafe {
			bs.connect_controller(
				*handle,
				None,
				None,
				raw::types::Boolean::TRUE,
			)?
		};
	}

	X((),)
}

fn clear_console(syst: *const SystemTable,) -> PoisonGirlB<(),>
{
	let Some(syst,) = (unsafe { syst.as_ref() }) else {
		return Y(poison_girl_err!(UefiError::Custom("system table is null")),);
	};
	let Some(stdout,) = (unsafe { syst.stdout.as_mut() }) else {
		return Y(poison_girl_err!(UefiError::Custom("stdout is null")),);
	};
	stdout.clear()?;
	X((),)
}

fn into_null_terminated_utf16(s: impl AsRef<str,>,) -> Vec<u16,>
{
	let mut utf16_repr: Vec<u16,> = s.as_ref().encode_utf16().collect();
	utf16_repr.push(0,);
	utf16_repr
}

pub fn get_device_tree() -> PoisonGirlB<NonNull<ConfigTable,>,>
{
	match unsafe { system_table()?.as_ref() }.device_tree() {
		X(Some(dt,),) => X(dt,),
		X(None,) => {
			Y(poison_girl_err!(UefiError::Custom("failed to get device tree")),)
		},
		Y(e,) => Y(e,),
	}
}

pub fn exec_kernel(kernel_entry: u64, device_tree_ptr: DeviceTreeAddress,)
{
	// Convert entry point to function pointer
	let kernel_entry = kernel_entry as *const ();

	// Define kernel entry point signature based on architecture
	cfg_if! {
		if #[cfg(target_arch = "aarch64")] {
			type KernelEntry = extern "C" fn(DeviceTreeAddress,);
		} else if #[cfg(target_arch = "x86_64")] {
			type KernelEntry = extern "sysv64" fn(DeviceTreeAddress,);
		}
	}

	let entry_point = unsafe {
		core::mem::transmute::<*const (), KernelEntry,>(kernel_entry,)
	};

	// Architecture-specific preparation for kernel execution
	#[cfg(target_arch = "aarch64")]
	unsafe {
		use core::arch::asm;

		// Wait for all data accesses to complete
		asm!("dsb sy");

		// Clear all caches as a precaution
		asm!("ic iallu"); // Invalidate all instruction cache
		asm!("dsb ish"); // Wait for invalidation to complete
		asm!("isb"); // Reload instructions after cache clear
		// Cache reload is necessary as instructions may already be cached

		// Disable MMU by modifying SCTLR_EL1
		asm!(
			"mrs x0, sctlr_el1", // Read current MMU state into x0 register (should be enabled)
			"bic x0, x0, #1", // Clear the lowest bit in x0 register
			// This value represents MMU disabled state
			"msr sctlr_el1, x0", // Apply the value, actually disabling MMU
			"isb", // Reload instructions after system state change
			out("x0") _
		);
	}

	// Jump to kernel with MMU disabled
	entry_point(device_tree_ptr,);

	// If we reach here, kernel execution failed
	idle_cpu_forever();
}
