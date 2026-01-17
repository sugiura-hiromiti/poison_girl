//! # OSO Kernel Entry Point
//!
//! This module contains the main entry points for the OSO kernel on different
//! architectures. It handles the initial kernel setup, interrupt configuration,
//! and application execution.
//!
//! ## Architecture Support
//!
//! - **AArch64**: Primary target with full feature support
//! - **x86_64**: Partial support for development and testing
//!
//! ## Boot Process
//!
//! 1. Bootloader transfers control to `kernel_main`
//! 2. Interrupts are disabled for initialization safety
//! 3. Kernel subsystems are initialized via `init()`
//! 4. Main application is launched
//! 5. System enters low-power wait state
//!
//! ## Safety Considerations
//!
//! This module contains unsafe code for:
//! - Direct assembly instruction execution
//! - Interrupt control register manipulation
//! - Low-level hardware initialization

#![no_std]
#![no_main]
// Enable ARM-specific hints when needed
#![feature(stdarch_arm_hints)]

use core::arch::asm;
use poison_girl_error::Rslt;
#[cfg(target_arch = "aarch64")]
use poison_girl_no_std::bridge::device_tree::DeviceTreeAddress;
use poison_girl_no_std::wfi;

// TODO: Re-enable graphics functionality when implemented
// use oso_kernel::base::graphic::FrameBuffer;
// #[cfg(feature = "bgr")]
// use oso_kernel::base::graphic::color::Bgr;
// #[cfg(feature = "bitmask")]
// use oso_kernel::base::graphic::color::Bitmask;
// #[cfg(feature = "bltonly")]
// use oso_kernel::base::graphic::color::BltOnly;
// #[cfg(feature = "rgb")]
// use oso_kernel::base::graphic::color::Rgb;
// use oso_kernel::base::graphic::fill_rectangle;
// use oso_kernel::base::graphic::outline_rectangle;

use poison_girl_kernel::init;

/// Main entry point for the OSO kernel on AArch64 architecture
///
/// This function is called by the bootloader after the kernel has been loaded
/// into memory. It performs critical initialization steps and launches the main
/// kernel application.
///
/// # Arguments
///
/// * `_device_tree_ptr` - Pointer to the device tree blob (DTB) passed by the
///   bootloader. Currently unused but reserved for future hardware discovery
///   implementation.
///
/// # Safety
///
/// This function is marked as `unsafe` because it:
/// - Directly manipulates interrupt control registers via inline assembly
/// - Performs low-level hardware initialization
/// - Must only be called once during the boot process
///
/// # Boot Sequence
///
/// 1. **Interrupt Disable**: Disables IRQ (Interrupt Request) to prevent
///    interruptions during critical initialization phases
/// 2. **Kernel Initialization**: Calls `init()` to set up all kernel subsystems
/// 3. **Application Launch**: Starts the main kernel application
/// 4. **Power Management**: Enters wait-for-interrupt state to conserve power
///
/// # Assembly Instructions
///
/// - `msr daifset, #2`: Sets the IRQ mask bit in the DAIF register to disable
///   interrupts
///
/// # Examples
///
/// This function is typically called by the bootloader:
///
/// ```asm
/// // Bootloader assembly code
/// bl kernel_main  // Branch to kernel entry point
/// ```
///
/// # TODO
///
/// - Implement device tree parsing for hardware discovery
/// - Add proper interrupt controller initialization
/// - Implement memory management setup
/// - Add error handling for initialization failures
#[unsafe(no_mangle)]
#[cfg(target_arch = "aarch64")]
pub extern "C" fn kernel_main(_device_tree_ptr: DeviceTreeAddress,) {
	// Disable IRQ (interrupt request) to prevent interruptions during
	// initialization This is critical for system stability during the boot
	// process
	unsafe {
		// Set IRQ mask bit (bit 1) in DAIF register
		// DAIF: Debug, SError, IRQ, FIQ exception mask register
		asm!("msr daifset, #2");
	}

	// Initialize all kernel subsystems
	init();

	// Launch the main kernel application
	let _ = app();

	// Enter wait-for-interrupt state for power efficiency
	// This stops the CPU until an interrupt occurs, conserving power
	// while keeping the system responsive to hardware events
	wfi();
}

/// Main entry point for the OSO kernel on x86_64 architecture
///
/// This function provides basic x86_64 support for development and testing
/// purposes. The x86_64 implementation is currently minimal and primarily used
/// for cross-platform development validation.
///
/// # Calling Convention
///
/// Uses the System V AMD64 ABI calling convention (`extern "sysv64"`), which is
/// the standard for x86_64 systems on Unix-like operating systems.
///
/// # Current Implementation
///
/// The current implementation immediately enters a halt loop for debugging
/// purposes. This prevents the system from continuing execution and allows for
/// debugging and development work.
///
/// # Assembly Instructions
///
/// - `hlt`: Halt instruction that stops the processor until the next interrupt
///
/// # Future Implementation
///
/// The commented code shows the intended future implementation that will:
/// - Support different pixel formats (RGB, BGR, Bitmask, BltOnly)
/// - Initialize graphics subsystems
/// - Launch applications based on feature flags
///
/// # Safety
///
/// This function is marked as `unsafe` because it uses inline assembly
/// to execute the `hlt` instruction directly.
///
/// # TODO
///
/// - Implement proper x86_64 initialization sequence
/// - Add interrupt handling for x86_64
/// - Enable graphics support for x86_64 targets
/// - Implement proper application launching
/// - Add memory management for x86_64
#[unsafe(no_mangle)]
#[cfg(target_arch = "x86_64")]
pub extern "sysv64" fn kernel_main() {
	// Current implementation: halt immediately for debugging
	// This prevents further execution and allows for system inspection
	loop {
		unsafe {
			// Halt the processor until the next interrupt
			// This is a power-efficient way to stop execution
			asm!("hlt");
		}
	}

	// TODO: Implement proper x86_64 kernel initialization
	// The following code represents the intended future implementation:

	// Feature-based application entry points for different pixel formats
	#[cfg(feature = "rgb")]
	enter_app!(Rgb, frame_buf_conf);
	#[cfg(feature = "bgr")]
	enter_app!(Bgr, frame_buf_conf);
	#[cfg(feature = "bitmask")]
	enter_app!(Bitmask, frame_buf_conf);
	#[cfg(feature = "bltonly")]
	// enter_app!(BltOnly, frame_buf_conf);

	// Fallback halt loop if no features are enabled
	loop {
		unsafe {
			asm!("hlt");
		}
	}
}

/// Main kernel application entry point
///
/// This function represents the primary application that runs after kernel
/// initialization. Currently, it serves as a placeholder for future application
/// functionality and contains commented-out graphics demonstration code.
///
/// # Returns
///
/// * `Rslt<()>` - Result indicating success or failure of application execution
///
/// # Current Implementation
///
/// The current implementation immediately returns `Ok(())` as a placeholder.
/// All graphics-related functionality is commented out pending implementation
/// of the graphics subsystem.
///
/// # Planned Functionality
///
/// The commented code demonstrates the intended graphics capabilities:
///
/// - **Rectangle Drawing**: Fill and outline rectangle operations
/// - **Color Support**: Various color formats and hex color parsing
/// - **Frame Buffer Operations**: Direct frame buffer manipulation
/// - **Cursor Support**: Mouse cursor rendering and management
/// - **Debug Output**: Frame buffer information logging
///
/// # Graphics Operations (Planned)
///
/// - `fill_rectangle()`: Fill rectangular areas with solid colors
/// - `outline_rectangle()`: Draw rectangular outlines
/// - `CursorBuf::draw_mouse_cursor()`: Render mouse cursor graphics
///
/// # Error Handling
///
/// Returns a `Rslt<()>` to allow for proper error propagation when
/// graphics operations are implemented. Currently always returns success.
///
/// # Examples
///
/// Future usage will include:
///
/// ```rust,ignore
/// // Fill background
/// fill_rectangle(&(0, 0), &frame_buffer.right_bottom(), &"#ffffff")?;
///
/// // Draw colored rectangles
/// fill_rectangle(&(100, 100), &(200, 200), &"#fedcba")?;
///
/// // Draw outlines
/// outline_rectangle(&(100, 100), &(300, 300), &"#fedcba")?;
/// ```
///
/// # TODO
///
/// - Implement graphics subsystem initialization
/// - Enable frame buffer operations
/// - Add color parsing and management
/// - Implement cursor rendering system
/// - Add user interface elements
/// - Implement application lifecycle management
fn app() -> Rslt<(),> {
	// TODO: Implement graphics operations
	// The following code represents planned graphics functionality:

	// Background and rectangle filling operations
	// fill_rectangle(&(100, 100,), &(700, 500,), &"#abcdef",)?;
	// fill_rectangle(&(0, 0,), &FRAME_BUFFER.right_bottom(), &"#012345",)?;
	// fill_rectangle(&(100, 100,), &(200, 200,), &"#fedcba",)?;
	// fill_rectangle(&(0, 0,), &FRAME_BUFFER.right_bottom(), &"#ffffff",)?;
	// fill_rectangle(&(0, 0,), &FRAME_BUFFER.right_bottom(), &"#abcdef",)?;

	// Outline rectangle operations
	// outline_rectangle(&(100, 100,), &(300, 300,), &"#fedcba",)?;
	// outline_rectangle(&(101, 101,), &(299, 299,), &"#fedcba",)?;
	// outline_rectangle(&(102, 102,), &(298, 298,), &"#fedcba",)?;

	// Debug information output
	// println!("width: {} height: {}", FRAME_BUFFER.width,
	// FRAME_BUFFER.height); println!("size: {} stride: {}", FRAME_BUFFER.size,
	// FRAME_BUFFER.stride); println!("buf address: {}", FRAME_BUFFER.buf);

	// Cursor rendering
	// let mut cursor_buf = CursorBuf::new();
	// cursor_buf.draw_mouse_cursor()?;

	// Return success for now
	Ok((),)
}
