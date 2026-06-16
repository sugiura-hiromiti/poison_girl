//! # Builder Module
//!
//! Core functionality for building the poison_girl loader and kernel, creating
//! disk images, and running QEMU.
//!
//! This module handles:
//! - Building the poison_girl loader and kernel for the target architecture
//! - Creating and formatting a disk image
//! - Mounting the disk image and copying the built artifacts
//! - Configuring and running QEMU with the appropriate firmware and disk image
//! - Cleanup of temporary files and unmounting disk images

use {
	crate::Xtask,
	poison_girl_dev_cargo::{Assets, CheckKind, CliCommand},
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_orchestrate::{
		Task,
		decl_manage::{
			PoisonGirlCargoInterface,
			crate_::{CrateAction, PoisonGirlCrateChart},
			workspace::WorkspaceAction,
		},
	},
};

impl Xtask
{
	/// Creates a new Xtask instance with the specified options
	///
	/// This constructor initializes all the necessary components for the build
	/// process:
	/// - Parses command-line options and build configuration
	/// - Sets up the poison_girl workspace with project paths
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
	/// * `Ok(Xtask)` - A fully initialized Xtask instance ready for use
	/// * `Err(anyhow::Error)` - If initialization fails due to:
	///   - Invalid workspace structure
	///   - Firmware download failure
	///   - Unsupported host operating system
	///   - Network connectivity issues
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// use poison_girl::Xtask;
	///
	/// // Create an xtask runner with default configuration.
	/// let xtask = Xtask::new()?;
	/// ```
	///
	/// # Errors
	///
	/// This method can fail in several scenarios:
	/// - **Workspace Error**: If the poison_girl project structure is invalid
	///   or incomplete
	/// - **Firmware Error**: If OVMF firmware files cannot be downloaded or
	///   accessed
	/// - **Host OS Error**: If the host operating system is not supported
	///   (Windows)
	/// - **Network Error**: If firmware download requires internet access and
	///   fails
	pub fn new() -> PoisonGirlB<Self,>
	{
		let task = Task::new()?;
		let chart = PoisonGirlCrateChart::XTASK;
		let assets = Assets::new(task.opts().arch,)?;
		X(Self {
			interface: PoisonGirlCargoInterface::new(chart, task,),
			assets,
		},)
	}

	pub fn runner(&self,) -> PoisonGirlB<(),>
	{
		match &self.interface.task().cmd() {
			CliCommand::Build => self.build(),
			CliCommand::Test => self.test(),
			CliCommand::Run => self.run(),
			CliCommand::Check { kind, } => match kind {
				Some(CheckKind::KernelAarch64,) => self.kernel_check(),
				Some(CheckKind::LoaderAarch64Uefi,) => self.loader_check(),
				Some(CheckKind::Clippy,) => self.clippy(),
				None => self.check(),
			},
			CliCommand::Fixture => self.fixture(),
			CliCommand::Fix => self.fix(),
		}
	}

	/// this is workspace build.
	/// not a package build
	fn build(&self,) -> PoisonGirlB<(),>
	{
		let args = self.interface.task().opts();
		self.ws().build_at_with(PoisonGirlCrateChart::KERNEL, args,)?;
		self.ws().build_at_with(PoisonGirlCrateChart::LOADER, args,)?;
		X((),)
	}

	/// this is workspace run.
	/// not a package run
	fn run(&self,) -> PoisonGirlB<(),>
	{
		self.build()?;
		self.qemu_run()
	}

	fn check(&self,) -> PoisonGirlB<(),>
	{
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

	/// this function generates fixture for kernel/loader test of low layer
	/// parts. e.g. this function setups toy eif/elf binaries for tests
	fn fixture(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	/// run all tests. since this workspace contains both std codes and no_std
	/// ones, we have to orchestrate them for running by one command
	fn test(&self,) -> PoisonGirlB<(),>
	{
		todo!()
	}

	/// this runs cargo fix for all crates in this repository
	fn fix(&self,) -> PoisonGirlB<(),>
	{
		let args = self.interface.task().opts();
		self.ws().fix_with(args,)
	}
}
