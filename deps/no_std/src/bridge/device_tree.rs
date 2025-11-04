//! # Device Tree Module
//!
//! This module provides types and utilities for working with Device Trees,
//! which are data structures that describe hardware components in a system.
//!
//! Device Trees are commonly used in embedded systems and operating systems
//! to provide a hardware description that the kernel can use to configure
//! drivers and manage hardware resources.

/// Represents a pointer to a Device Tree Blob (DTB) in memory.
///
/// This type alias provides a convenient way to pass around and work with
/// device tree addresses in a type-safe manner. The device tree is typically
/// passed to the kernel by the bootloader.
///
/// # Examples
///
/// ```rust,no_run
/// use oso_no_std_shared::bridge::device_tree::DeviceTreeAddress;
///
/// fn process_device_tree(dtb_addr: DeviceTreeAddress,) {
/// 	// Parse and process the device tree at the given address
/// 	// ...
/// }
///
/// // In kernel entry point:
/// let dtb_addr: DeviceTreeAddress = 0x4000_0000 as *const u8;
/// process_device_tree(dtb_addr,);
/// ```
///
/// # Safety
///
/// This is a raw pointer and should be used with care. The caller must ensure
/// that the address points to a valid Device Tree Blob in memory.
pub type DeviceTreeAddress = *const u8;
