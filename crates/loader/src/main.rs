//! # OSO Loader Main Entry Point
//!
//! This module contains the main UEFI application entry point for the OSO
//! bootloader. It orchestrates the boot process from UEFI initialization
//! through kernel handoff.

#![no_std]
#![no_main]

extern crate alloc;

use oso_error::Rslt;
use oso_loader::chibi_uefi::service::exit_boot_services;
use oso_loader::exec_kernel;
use oso_loader::get_device_tree;
use oso_loader::init;
use oso_loader::load::kernel;
use oso_loader::raw::table::SystemTable;
use oso_loader::raw::types::Status;
use oso_loader::raw::types::UnsafeHandle;
use oso_no_std_shared::bridge::device_tree::DeviceTreeAddress;

/// UEFI application entry point
///
/// This is the main entry point called by the UEFI firmware when the loader
/// is executed. It follows the UEFI application calling convention and performs
/// the complete boot sequence.
///
/// # Boot Sequence
///
/// 1. **Initialization**: Set up UEFI services and connect devices
/// 2. **Kernel Loading**: Load and parse the ELF kernel from filesystem
/// 3. **Device Tree**: Retrieve hardware configuration information
/// 4. **Boot Services Exit**: Transition from boot-time to runtime environment
/// 5. **Kernel Execution**: Transfer control to the loaded kernel
///
/// # Arguments
///
/// * `image_handle` - UEFI handle for this loader application
/// * `system_table` - Pointer to the UEFI system table containing services
///
/// # Returns
///
/// * `Status::EFI_SUCCESS` - Boot completed successfully (should not return)
///
/// # Panics
///
/// Panics if any critical boot operation fails, as recovery is not possible
/// in the bootloader context.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's exported with a specific
/// name for UEFI firmware to call, and it handles raw UEFI pointers.
#[unsafe(export_name = "efi_main")]
pub extern "efiapi" fn efi_image_entry_point(
	image_handle: UnsafeHandle,
	system_table: *const SystemTable,
) -> Status {
	// Initialize UEFI environment and connect devices
	init(image_handle, system_table,);

	// Load kernel and prepare for execution
	let (kernel_entry, device_tree_ptr,) =
		app().expect("error arise while executing application",);

	// Exit UEFI boot services - point of no return
	exit_boot_services();

	// Transfer control to kernel
	exec_kernel(kernel_entry, device_tree_ptr,);

	// Should never reach here
	Status::EFI_SUCCESS
}

/// Main application logic for the bootloader
///
/// This function encapsulates the core bootloader functionality:
/// - Loading the kernel ELF file from the filesystem
/// - Retrieving the device tree configuration
/// - Preparing parameters for kernel execution
///
/// # Returns
///
/// * `Ok((u64, DeviceTreeAddress))` - Tuple containing:
///   - Kernel entry point address
///   - Device tree pointer for kernel initialization
/// * `Err(_)` - If kernel loading or device tree retrieval fails
///
/// # Errors
///
/// This function can fail if:
/// - The kernel file cannot be found or loaded
/// - The ELF parsing fails
/// - Memory allocation for kernel loading fails
/// - Device tree cannot be retrieved from UEFI
fn app() -> Rslt<(u64, DeviceTreeAddress,),> {
	// Load kernel ELF file and get entry point
	let kernel_addr = kernel()?;

	// Get device tree configuration for kernel
	let device_tree = get_device_tree()?;

	// Convert device tree pointer for kernel handoff
	let device_tree_ptr = device_tree.as_ptr().cast_const().cast();

	Ok((kernel_addr, device_tree_ptr,),)
}
