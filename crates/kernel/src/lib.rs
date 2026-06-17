#![no_std]
#![allow(incomplete_features)]
#![feature(associated_type_defaults)]
// #![feature(impl_trait_in_assoc_type)]
// #![feature(slice_index_methods)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]

#[cfg(test)] extern crate std;

/// Application execution and management subsystem
///
/// This module provides functionality for running user applications and
/// managing their lifecycle within the kernel environment.
pub mod app;

/// Core kernel functionality and basic data structures
///
/// This module contains fundamental kernel components including memory
/// management, process management, and core system utilities.
pub mod base;

/// Hardware device drivers and low-level hardware abstraction
///
/// This module provides device drivers for various hardware components and
/// abstractions for hardware-specific operations.
pub mod driver;

/// Initializes the kernel and all its subsystems
///
/// This function is responsible for setting up the kernel environment,
/// initializing hardware components, and preparing the system for operation. It
/// should be called once during the boot process after the bootloader has
/// transferred control to the kernel.
///
/// # Initialization Sequence
///
/// The initialization process includes:
///
/// 1. **Hardware Initialization**: Set up CPU, memory management unit, and
///    interrupt controllers
/// 2. **Kernel Setup**: Initialize core kernel data structures and subsystems
/// 3. **Utility Setup**: Configure system utilities and services
/// 4. **Driver Initialization**: Load and initialize device drivers
/// 5. **Application Framework**: Prepare the application execution environment
///
/// # Safety
///
/// This function performs low-level hardware initialization and should only be
/// called once during the boot process. Multiple calls may result in undefined
/// behavior.
///
/// # Examples
///
/// ```rust,ignore
/// use poison_girl_kernel::init;
///
/// // Called by the bootloader after kernel loading
/// #[no_mangle]
/// pub extern "C" fn kernel_main() -> ! {
///     // Initialize all kernel subsystems
///     init();
///
///     // Start the main kernel loop
///     loop {
///         // Handle system events
///     }
/// }
/// ```
///
/// # TODO
///
/// - Implement memory management initialization
/// - Set up interrupt handling
/// - Initialize device drivers
/// - Configure system services
/// - Set up application execution environment
pub fn init()
{
	// TODO: Implement hardware initialization
	// TODO: Set up memory management
	// TODO: Initialize interrupt controllers
	// TODO: Load device drivers
	// TODO: Configure system services
}

// pub mod test {
// 	use crate::print;
// 	use crate::println;
//
// 	#[cfg(test)]
// 	pub fn test_runner(tests: &[&dyn Testable],) {
// 		println!("running {} tests", tests.len());
// 		for test in tests {
// 			test.run_test()
// 		}
// 		loop {}
// 	}
//
// 	pub trait Testable {
// 		fn run_test(&self,);
// 	}
//
// 	impl<T: Fn(),> Testable for T {
// 		fn run_test(&self,) {
// 			print!("{}   ---------------\n", core::any::type_name::<T,>());
// 			self();
// 			println!("\t\t\t\t...[ok]");
// 		}
// 	}
//
// 	#[test_case]
// 	fn exmpl() {
// 		let a = 1 + 1;
// 		assert_eq!(2, a);
// 	}
// }
