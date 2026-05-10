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
	crate::Xtask,
	poison_girl_dev_cargo::{AsCargoOpt, Assets, CheckKind, CliCommand, Opts},
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_orchestrate::decl_manage::{
		crate_::CrateAction, project_root,
	},
};

impl Xtask
{
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
	pub fn new() -> PoisonGirlB<Self,>
	{
		let opts = Opts::new();
		let ws = project_root()?;
		let assets = Assets::new(opts.arch,)?;
		X(Self { opts, ws, assets, },)
	}

	pub fn runner(&self,) -> PoisonGirlB<(),>
	{
		match &self.opts.command {
			CliCommand::Build => self.build(),
			CliCommand::Test => self.test(),
			CliCommand::Run => self.run(),
			CliCommand::Check { kind, } => match kind {
				Some(CheckKind::KernelAarch64,) => self.kernel_check(),
				Some(CheckKind::LoaderAarch64Uefi,) => self.loader_check(),
				Some(CheckKind::Clippy,) => self.clippy(),
				None => self.check(),
			},
			CliCommand::Fmt => self.fmt(),
			CliCommand::Fixture => self.fixture(),
			CliCommand::Fix => self.fix(),
		}
	}

	fn build(&self,) -> PoisonGirlB<(),>
	{
		let args = self.opts.as_cargo_opt();
		self.ws.build_with(&args,)
	}

	fn run(&self,) -> PoisonGirlB<(),>
	{
		let args = self.opts.as_cargo_opt();
		todo!()
	}

	fn check(&self,) -> PoisonGirlB<(),>
	{
		let args = self.opts.as_cargo_opt();
		self.kernel_check()?;
		self.loader_check()?;
		self.clippy()
	}

	fn kernel_check(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	fn loader_check(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	fn clippy(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	fn fixture(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	fn test(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	fn fmt(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	fn fix(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}
}
