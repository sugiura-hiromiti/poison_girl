//! # Builder Module
//!
//! Core functionality for building the OSO loader and kernel, creating disk
//! images, and running QEMU.
//!
//! This module handles:
//! - Building the OSO loader and kernel for the target architecture
//! - Creating and formatting a disk image
//! - Mounting the disk image and copying the built artifacts
//! - Configuring and running QEMU with the appropriate firmware and disk image
//! - Cleanup of temporary files and unmounting disk images

use {
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_orchestrate::{
		cargo::{Assets, Opts},
		fs::project_root,
	},
};

use crate::Xtask;

/// Directory path for EFI boot files
const BOOT_DIR: &str = "efi/boot";
/// mounting point path under target/
const MOUNT_DIR: &str = "xtask/mnt";

impl Xtask {
	/// Creates a new Builder instance with the specified options
	///
	/// This constructor initializes all the necessary components for the build
	/// process:
	/// - Parses command-line options and build configuration
	/// - Sets up the OSO workspace with project paths
	/// - Downloads and configures OVMF firmware for the target architecture
	/// - Detects the host operating system for platform-specific operations
	///
	/// # Initialization Process
	///
	/// 1. **Options Parsing**: Reads command-line arguments for architecture,
	///    build mode, etc.
	/// 2. **Workspace Setup**: Locates project root and validates workspace
	///    structure
	/// 3. **Firmware Download**: Fetches appropriate OVMF firmware files for
	///    UEFI boot
	/// 4. **Host Detection**: Identifies the host OS (macOS, Linux) for mount
	///    operations
	///
	/// # Returns
	///
	/// * `Ok(Builder)` - A fully initialized Builder instance ready for use
	/// * `Err(anyhow::Error)` - If initialization fails due to:
	///   - Invalid workspace structure
	///   - Firmware download failure
	///   - Unsupported host operating system
	///   - Network connectivity issues
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use xtask::builder::Builder;
	///
	/// // Create a builder with default configuration
	/// let builder = Builder::new()?;
	/// println!("Building for architecture: {:?}", builder.arch());
	/// ```
	///
	/// # Errors
	///
	/// This method can fail in several scenarios:
	/// - **Workspace Error**: If the OSO project structure is invalid or
	///   incomplete
	/// - **Firmware Error**: If OVMF firmware files cannot be downloaded or
	///   accessed
	/// - **Host OS Error**: If the host operating system is not supported
	///   (Windows)
	/// - **Network Error**: If firmware download requires internet access and
	///   fails
	pub fn new() -> PoisonGirlB<Self,> {
		let opts = Opts::new();
		let ws = project_root()?;
		let assets = Assets::new(opts.arch,)?;
		X(Self { opts, ws, assets, },)
	}
}
