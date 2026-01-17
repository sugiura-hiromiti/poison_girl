//! # OSO Loader
//!
//! A UEFI-based bootloader for the OSO operating system that handles ELF kernel
//! loading and system initialization across multiple architectures (x86_64,
//! aarch64, riscv64).
//!
//! ## Features
//!
//! - **ELF Kernel Loading**: Parses and loads ELF format kernels into memory
//! - **Multi-architecture Support**: Supports x86_64, aarch64, and riscv64
//!   architectures
//! - **UEFI Integration**: Provides a lightweight UEFI interface wrapper
//! - **Device Tree Support**: Handles device tree configuration for kernel
//!   handoff
//! - **Graphics Configuration**: Sets up frame buffer configuration for kernel
//!   graphics
//! - **Memory Management**: Manages memory allocation and MMU configuration
//!
//! ## Architecture-specific Features
//!
//! The loader includes architecture-specific code for:
//! - MMU disabling (aarch64)
//! - Cache management (aarch64)
//! - Calling conventions (different for each architecture)
//!
//! ## Usage
//!
//! This crate is designed to be compiled as a UEFI application. The main entry
//! point is `efi_main` which initializes the system, loads the kernel, and
//! transfers control.

#![no_std]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(ptr_as_ref_unchecked)]
#![feature(iter_next_chunk)]
#![feature(const_trait_impl)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(assert_matches)]
// #![feature(nonzero_internals)]
//#![feature(stdarch_arm_hints)]

extern crate alloc;

use alloc::vec::Vec;
use chibi_uefi::protocol::HandleSearchType;
use chibi_uefi::table::boot_services;
use core::ptr::NonNull;
use oso_error::Rslt;
use oso_error::loader::UefiError;
use oso_error::oso_err;
use oso_no_std_shared::bridge::device_tree::DeviceTreeAddress;
use oso_no_std_shared::wfe;
use oso_no_std_shared::wfi;
use raw::table::SystemTable;
use raw::types::Status;
use raw::types::UnsafeHandle;

use crate::chibi_uefi::table::system_table;
use crate::raw::table::ConfigTable;

/// UEFI interface wrapper providing simplified access to UEFI services
pub mod chibi_uefi;
/// ELF file parsing and loading functionality
pub mod elf;
/// Kernel and graphics loading utilities
pub mod load;
/// Raw UEFI types and protocol definitions
pub mod raw;

/// Custom panic handler for the UEFI environment
///
/// This panic handler prints debug information and enters a wait-for-event loop
/// instead of terminating the program, which is appropriate for a UEFI
/// application.
#[panic_handler]
fn panic(panic: &core::panic::PanicInfo,) -> ! {
	println!("{panic:#?}");
	wfe()
}

/// Macro for handling errors that cannot be processed with the `?` operator
///
/// This macro logs error information when an unrecoverable error occurs.
/// It's particularly useful for debugging loader issues.
///
/// # Arguments
///
/// * `$e` - The error variable to log
/// * `$situation` - A string describing the situation where the error occurred
///
/// # Example
///
/// ```rust,ignore
/// let result = some_operation();
/// if let Err(e,) = result {
/// 	on_error!(e, "during kernel loading");
/// }
/// ```
#[macro_export]
macro_rules! on_error {
	($e:ident, $situation:expr) => {{
		log::error!("error happen {}", $situation);
		log::error!("error msg:");
		log::error!("{}", $e);
	}};
}

/// Initializes the UEFI loader environment
///
/// This function performs essential initialization tasks:
/// - Clears the console output
/// - Sets up the system table and image handle
/// - Connects all available UEFI devices
///
/// Device connection is performed by iterating through all handles and calling
/// `connect_controller` on each one. This ensures that device path protocols
/// are properly installed on connected devices.
///
/// # Arguments
///
/// * `image_handle` - The UEFI image handle for this application
/// * `syst` - Pointer to the UEFI system table
///
/// # Panics
///
/// Panics if:
/// - The system table is null or invalid
/// - Console clearing fails
/// - Handle location fails during device connection
pub fn init(image_handle: UnsafeHandle, syst: *const SystemTable,) {
	// Clear console output for clean startup
	clear_console(syst,);

	// Initialize UEFI table access
	chibi_uefi::table::set_system_table_panicking(syst,);
	chibi_uefi::set_image_handle_panicking(image_handle,);

	// Connect all available devices
	let bs = boot_services();

	// UEFI only installs DevicePathProtocol on devices that are fully connected
	// `AllHandles` is the only way to find unconnected devices
	let handles = unsafe {
		bs.locate_handle_buffer(HandleSearchType::AllHandles,)
			.expect("failed to locate all handles ",)
	};

	// Connect each device, ignoring connection errors
	handles.iter().for_each(|handle| {
		// Ignore errors from connect_controller intentionally
		unsafe {
			bs.connect_controller(
				*handle,
				None,
				None,
				raw::types::Boolean::TRUE,
			)
		};
	},);
}

fn clear_console(syst: *const SystemTable,) {
	unsafe { syst.as_ref().unwrap().stdout.as_mut().unwrap().clear().unwrap() };
}

/// Converts a string to null-terminated UTF-16 format
///
/// This utility function is used for UEFI string operations which require
/// null-terminated UTF-16 strings.
///
/// # Arguments
///
/// * `s` - String-like input to convert
///
/// # Returns
///
/// A vector containing the UTF-16 representation with null terminator
fn into_null_terminated_utf16(s: impl AsRef<str,>,) -> Vec<u16,> {
	let mut utf16_repr: Vec<u16,> = s.as_ref().encode_utf16().collect();
	utf16_repr.push(0,);
	utf16_repr
}

/// Retrieves the device tree configuration table from UEFI
///
/// The device tree is essential for kernel initialization on ARM and RISC-V
/// architectures, providing hardware configuration information.
///
/// # Returns
///
/// * `Ok(NonNull<ConfigTable>)` - Pointer to the device tree configuration
///   table
/// * `Err(UefiError)` - If the device tree cannot be found or accessed
///
/// # Errors
///
/// Returns `UefiError::Custom` if the device tree is not available in the
/// UEFI configuration tables.
pub fn get_device_tree() -> Rslt<NonNull<ConfigTable,>, UefiError,> {
	unsafe { system_table().as_ref() }
		.device_tree()?
		.ok_or(oso_err!(UefiError::Custom("failed to get device tree")),)
}

/// Executes the loaded kernel with proper architecture-specific setup
///
/// This function performs the final handoff to the kernel:
/// 1. Disables MMU (on aarch64)
/// 2. Clears caches (on aarch64)
/// 3. Calls the kernel entry point with the device tree address
/// 4. Falls back to wait-for-interrupt if kernel returns
///
/// # Arguments
///
/// * `kernel_entry` - Physical address of the kernel entry point
/// * `device_tree_ptr` - Address of the device tree for kernel initialization
///
/// # Architecture-specific Behavior
///
/// ## AArch64
/// - Performs data synchronization barrier
/// - Invalidates instruction cache
/// - Disables MMU by clearing SCTLR_EL1 bit 0
/// - Uses ARM calling convention
///
/// ## x86_64
/// - Uses System V AMD64 calling convention
///
/// ## RISC-V 64
/// - Uses standard C calling convention
///
/// # Safety
///
/// This function performs low-level system operations including:
/// - Memory management unit manipulation
/// - Cache operations
/// - Direct kernel execution
///
/// The function never returns under normal circumstances.
pub fn exec_kernel(kernel_entry: u64, device_tree_ptr: DeviceTreeAddress,) {
	// Convert entry point to function pointer
	let kernel_entry = kernel_entry as *const ();

	// Define kernel entry point signature based on architecture
	#[cfg(target_arch = "riscv64")]
	type KernelEntry = extern "C" fn(DeviceTreeAddress,);
	#[cfg(target_arch = "aarch64")]
	type KernelEntry = extern "C" fn(DeviceTreeAddress,);
	#[cfg(target_arch = "x86_64")]
	type KernelEntry = extern "sysv64" fn(DeviceTreeAddress,);

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
	wfi();
}
