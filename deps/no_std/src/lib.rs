// #![feature(min_specialization)]
// #![feature(specialization)]

//! # OSO No-Std Shared Library
//!
//! This crate provides shared utilities and data structures for the OSO
//! operating system that work in `no_std` environments. It serves as a
//! foundational library containing common functionality used across different
//! components of the OSO ecosystem.
//!
//! ## Features
//!
//! - **Bridge Module**: Low-level hardware interfaces and CPU control functions
//! - **Data Module**: Generic data structures like trees for system data
//!   management
//! - **Parser Module**: Parsing utilities for binary data, HTML, and code
//!   generation
//! - **CPU Control**: Platform-specific CPU power management functions
//!
//! ## Architecture
//!
//! This crate is designed to work in bare-metal environments and uses several
//! unstable Rust features to provide zero-cost abstractions and compile-time
//! optimizations.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use oso_no_std_shared::bridge::graphic::FrameBufConf;
//! use oso_no_std_shared::bridge::graphic::PixelFormatConf;
//! use oso_no_std_shared::wfi;
//!
//! // Configure graphics
//! let framebuf = FrameBufConf::new(
//! 	PixelFormatConf::Rgb,
//! 	0x1000_0000 as *mut u8,
//! 	1024 * 768 * 4,
//! 	1024,
//! 	768,
//! 	1024 * 4,
//! );
//!
//! // Enter low-power state
//! wfi(); // Never returns
//! ```

#![no_std]
// Enable unstable features required for advanced type system usage
#![feature(unboxed_closures)]
#![feature(associated_type_defaults)]
#![feature(impl_trait_in_assoc_type)]
#![feature(const_trait_impl)]

// Public modules
pub mod bridge;
pub mod data;
pub mod parser;

use core::arch::asm;

/// Puts the CPU into a low-power state until an interrupt occurs.
///
/// This function enters an infinite loop where the CPU is repeatedly put into a
/// wait-for-interrupt state. This is commonly used in bare-metal environments
/// to conserve power when there's no work to be done.
///
/// # Platform-specific behavior
///
/// - On AArch64 (ARM): Uses the `wfi` (Wait For Interrupt) instruction
/// - On x86_64: Uses the `hlt` (Halt) instruction
///
/// # Examples
///
/// ```rust,no_run
/// use oso_no_std_shared::wfi;
///
/// // After completing all necessary work:
/// wfi(); // CPU will enter low-power state until an interrupt occurs
/// ```
///
/// # Safety
///
/// This function never returns and contains inline assembly.
#[inline(always)]
pub fn wfi() -> ! {
	loop {
		unsafe {
			if cfg!(target_arch = "aarch64") {
				asm!("wfi"); // ARM64: Wait For Interrupt
			} else if cfg!(target_arch = "riscv64") {
				todo!()
			} else if cfg!(target_arch = "x86_64") {
				asm!("hlt"); // x86_64: Halt until interrupt
			} else {
				unimplemented!("Architecture not supported")
			}
		}
	}
}

/// Puts the CPU into a low-power state until an event occurs.
///
/// This function enters an infinite loop where the CPU is repeatedly put into a
/// wait-for-event state. This is similar to `wfi()` but responds to events
/// rather than just interrupts, which can be useful in certain synchronization
/// scenarios.
///
/// # Platform-specific behavior
///
/// - On AArch64 (ARM): Uses the `wfe` (Wait For Event) instruction
/// - On x86_64: Uses the `hlt` (Halt) instruction as a fallback
///
/// # Examples
///
/// ```rust,no_run
/// use oso_no_std_shared::wfe;
///
/// // After setting up event monitoring:
/// wfe(); // CPU will enter low-power state until an event occurs
/// ```
///
/// # Safety
///
/// This function never returns and contains inline assembly.
#[inline(always)]
pub fn wfe() -> ! {
	loop {
		unsafe {
			if cfg!(target_arch = "aarch64") {
				asm!("wfe"); // ARM64: Wait For Event
			} else if cfg!(target_arch = "riscv64") {
				todo!()
			} else if cfg!(target_arch = "x86_64") {
				asm!("hlt"); // x86_64: Halt until interrupt
			} else {
				unimplemented!("Architecture not supported")
			}
		}
	}
}

/// Puts the CPU into an infinite loop of no-operation instructions.
///
/// This function enters an infinite loop where the CPU repeatedly executes
/// no-operation instructions. This can be useful for debugging or in situations
/// where you want to keep the CPU busy without doing meaningful work.
///
/// # Platform-specific behavior
///
/// - On AArch64 (ARM): Uses the `nop` (No Operation) instruction
/// - On x86_64: Uses the `hlt` (Halt) instruction as a fallback
///
/// # Examples
///
/// ```rust,no_run
/// use oso_no_std_shared::nop;
///
/// // When you want to keep the CPU busy without doing work:
/// nop(); // CPU will continuously execute no-operation instructions
/// ```
///
/// # Safety
///
/// This function never returns and contains inline assembly.
#[inline(always)]
pub fn nop() -> ! {
	loop {
		unsafe {
			// Platform-specific no-operation implementation
			if cfg!(target_arch = "aarch64") {
				asm!("nop"); // ARM64: Wait For Event
			} else if cfg!(target_arch = "riscv64") {
				todo!()
			} else if cfg!(target_arch = "x86_64") {
				asm!("hlt"); // x86_64: Halt until interrupt
			} else {
				unimplemented!("Architecture not supported")
			}
		}
	}
}
