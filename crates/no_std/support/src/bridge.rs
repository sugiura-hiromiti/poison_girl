//! # poison_girl Bridge
//!
//! `poison_girl_bridge` is a no_std crate that provides low-level interfaces
//! and utilities for operating system development, particularly focused on
//! bare-metal environments.
//!
//! This crate serves as a bridge between hardware and the operating system
//! kernel, providing essential primitives for CPU control, framebuffer
//! management, and device tree access.
//!
//! ## Features
//!
//! - CPU control functions (wait for interrupt, wait for event, no-operation)
//! - Framebuffer configuration for graphics output
//! - Device tree address handling
//!
//! ## Usage
//!
//! This crate is designed to be used in no_std environments, typically in
//! kernel or bootloader code:
//!
//! ```rust,no_run,ignore
//! use poison_girl_no_std_shared::{
//! 	bridge::graphic::{FrameBufConf, PixelFormatConf},
//! 	wfi,
//! };
//!
//! // Configure a framebuffer
//! let framebuf = FrameBufConf::new(
//! 	PixelFormatConf::Rgb,
//! 	0x1000_0000 as *mut u8, // Base address
//! 	1024 * 768 * 4,         // Size (bytes)
//! 	1024,                   // Width (pixels)
//! 	768,                    // Height (pixels)
//! 	1024 * 4,               // Stride (bytes per row)
//! );
//!
//! // Put the CPU into a low-power state until an interrupt occurs
//! wfi(); // This function never returns
//! ```

use crate::bridge::{device_tree::DeviceTreeAddress, graphic::FrameBufConf};

pub mod device_tree;
pub mod graphic;

/// loaderからkernelに渡される情報を一つの構造体にまとめる
/// 必要に応じて拡張していこう!
#[repr(C)]
pub struct BootInfo
{
	frame_buf:           FrameBufConf,
	device_tree_address: DeviceTreeAddress,
}
