// #![feature(min_specialization)]
// #![feature(specialization)]

//! # poison_girl No-Std Shared Library
//!
//! This crate provides shared utilities and data structures for the poison_girl
//! operating system that work in `no_std` environments. It serves as a
//! foundational library containing common functionality used across different
//! components of the poison_girl ecosystem.
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
//! use poison_girl_no_std_shared::{
//! 	bridge::graphic::{FrameBufConf, PixelFormatConf},
//! 	wfi,
//! };
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

use {core::arch::asm, poison_girl_macro::cfg_if};

// Public modules
pub mod bridge;
pub mod data;
pub mod parser;

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
/// use poison_girl_no_std_shared::wfi;
///
/// // After completing all necessary work:
/// wfi(); // CPU will enter low-power state until an interrupt occurs
/// ```
///
/// # Safety
///
/// This function never returns and contains inline assembly.
#[inline(always)]
pub fn idle_cpu_forever() -> !
{
	loop {
		// unsafe {
		// 	if cfg!(target_arch = "aarch64") {
		// 		asm!("wfi"); // ARM64: Wait For Interrupt
		// 	} else if cfg!(target_arch = "riscv64") {
		// 		todo!()
		// 	} else if cfg!(target_arch = "x86_64") {
		// 		asm!("hlt"); // x86_64: Halt until interrupt
		// 	} else {
		// 		loop {}
		// 		// unimplemented!("Architecture not supported")
		// 	}
		// }
		wfi();
	}
}

#[inline(always)]
pub fn wfi()
{
	unsafe {
		// NOTE: here is explanation of options(..) part in asm! macro.
		// directly, options(nomem, nostack) means
		// > "this assembly instruction does not access normal memory, and it does not use the stack"
		// see [rust reference](https://doc.rust-lang.org/reference/inline-assembly.html#options) page for more detail
		//
		// nomem:
		//for `wfi`/`hlt`, this is usually reasonable becvause the instruction itself waits for an interrupt; it does not directly load/store memory.
		// but be careful: nomem also lets the compiler assume this assembly is not a synchronization point. Rust's reference explicitly says the compiler may assume nomem assembly does not synchronize with other threads, such as through fences.
		// nostack:
		// ensures these instructions do not use the stack(that is expected behavior)
		cfg_if! {
			if #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))] {
				asm!("wfi", options(nomem, nostack));
			} else if #[cfg(target_arch = "x86_64")] {
				asm!("hlt", options(nomem, nostack));
			} else {
				compile_error!("unsupported architecture for wait_for_interrupt");
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
/// use poison_girl_no_std_shared::wfe;
///
/// // After setting up event monitoring:
/// wfe(); // CPU will enter low-power state until an event occurs
/// ```
///
/// # Safety
///
/// This function never returns and contains inline assembly.
#[inline(always)]
pub fn wfe()
{
	unsafe {
		cfg_if! {
			if #[cfg(target_arch = "aarch64")] {
				asm!("wfe", options(nomem, nostack));
			} else {
				compile_error!("only aarch64 provides exact wait for event functionality instruction. we do not support other platform now")
			}
		}
	}
}

/// NOTE: IF YOU JUST WANT TO STOP PROGRAM, use `hinted_loop`
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
/// use poison_girl_no_std_shared::nop;
///
/// // When you want to keep the CPU busy without doing work:
/// nop(); // CPU will continuously execute no-operation instructions
/// ```
///
/// # Safety
///
/// This function never returns and contains inline assembly.
#[inline(always)]
pub fn nop()
{
	unsafe {
		cfg_if! {
			if #[cfg(any(
				target_arch = "aarch64",
				target_arch = "riscv64",
				target_arch = "x86_64",
			))] {
				asm!("nop", options(nomem, nostack, preserves_flags));
			} else {
				compile_error!("unsupported architecture for nop");
			}
		}
	}
}

/// just a wrapper of `core::hint::spin_loop`
#[inline(always)]
pub fn hinted_loop()
{
	core::hint::spin_loop()
}
