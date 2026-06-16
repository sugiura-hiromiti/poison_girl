// #![feature(string_from_utf8_lossy_owned)]
// #![feature(exit_status_error)]

//! # poison_girl xtask
//!
//! A build and run utility for the poison_girl project that automates the
//! process of building, packaging, and running the kernel and loader.
//!
//! This crate provides a convenient way to:
//! - Build the poison_girl loader (UEFI application) and kernel
//! - Create and format a disk image
//! - Mount the disk image and copy the built artifacts
//! - Configure and run QEMU with the appropriate firmware and disk image
//!
//! ## Usage
//!
//! Run from the poison_girl project root:
//!
//! ```bash
//! cargo xt [OPTIONS] [COMMAND]
//! ```
//!
//! ### Common options
//!
//! - `-b release`: Build in release mode (default is debug mode)
//! - `-a aarch64`: Build for the aarch64 target
//! - `-l`: Pass `--locked` through to Cargo commands

use {poison_girl::Xtask, poison_girl_dev_error::PoisonGirlB};

/// Entry point for the xtask utility.
///
/// Creates a new Xtask instance, builds the poison_girl loader and kernel,
/// and runs QEMU with the appropriate configuration.
fn main() -> PoisonGirlB<(),>
{
	let poison_girl = Xtask::new()?;

	let app = || poison_girl.runner();

	app()
}
