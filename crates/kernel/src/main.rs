#![no_std]
#![no_main]
// Enable ARM-specific hints when needed
#![feature(stdarch_arm_hints)]

use {
	core::arch::asm,
	poison_girl_kernel::{init, println},
	poison_girl_macro::cfg_if,
	poison_girl_no_std::{bridge::device_tree::DeviceTreeAddress, wfe, wfi},
	poison_girl_no_std_error::{PoisonGirlB, X},
};

cfg_if! {
	if #[cfg(target_arch = "aarch64")] {
		#[unsafe(no_mangle)]
		pub extern "C" fn kernel_main(_device_tree_ptr: DeviceTreeAddress,)
		{
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
	} else if #[cfg(target_arch = "x86_64")] {
		#[unsafe(no_mangle)]
		pub extern "sysv64" fn kernel_main()
		{
			// Current implementation: halt immediately for debugging
			// This prevents further execution and allows for system inspection
			loop {
				unsafe {
					// Halt the processor until the next interrupt
					// This is a power-efficient way to stop execution
					asm!("hlt");
				}
			}
		}
	}
}

/// Custom panic handler for the kernel environment
///
/// This panic handler is called when the kernel encounters an unrecoverable
/// error. It prints diagnostic information and enters a low-power
/// wait-for-event state to preserve system stability.
///
/// # Arguments
///
/// * `info` - Panic information including location and message
///
/// # Behavior
///
/// 1. Prints the panic information to the console
/// 2. Enters an infinite wait-for-event loop to conserve power
/// 3. Never returns, maintaining system in a stable state
///
/// # Examples
///
/// The panic handler is automatically invoked by the Rust runtime:
///
/// ```rust,ignore
/// // This will trigger the panic handler
/// panic!("Critical kernel error occurred");
/// ```
#[panic_handler]
fn panic(info: &core::panic::PanicInfo,) -> !
{
	println!("{}", info);
	wfe()
}

fn app() -> PoisonGirlB<(),>
{
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
	X((),)
}
