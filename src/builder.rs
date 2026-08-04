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
	poison_girl_dev_cargo::Assets,
	poison_girl_dev_cli::Run,
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_orchestrate::{
		CliCommandDiscriminants, Policy,
		decl_manage::{
			PoisonGirlCargoInterface, crate_::PoisonGirlCrateChart,
			workspace::WorkspaceAction,
		},
	},
	std::process::Command,
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
		let policy = Policy::new();
		let chart = PoisonGirlCrateChart::XTASK;
		let assets = Assets::new(policy.arch(),);
		X(Self {
			interface: PoisonGirlCargoInterface::new(chart, policy,),
			assets,
		},)
	}

	pub fn runner(&self,) -> PoisonGirlB<(),>
	{
		match self.interface.policy().command_discriminant() {
			CliCommandDiscriminants::Build => self.build(),
			CliCommandDiscriminants::Test => self.test(),
			CliCommandDiscriminants::Run => self.run(),
			CliCommandDiscriminants::Clippy => self.clippy(),
			CliCommandDiscriminants::Fixture => self.fixture(),
			CliCommandDiscriminants::Fix => self.fix(),
		}
	}

	/// Run every repository check used by CI.
	///
	/// This is the single source of truth for `nix flake check`; Nix only
	/// supplies the toolchain, dependencies, and sandbox.
	pub fn check() -> PoisonGirlB<(),>
	{
		run_cargo(&[
			"metadata",
			"--locked",
			"--no-deps",
			"--format-version",
			"1",
		],)?;
		run_cargo(&["build", "--workspace", "--locked",],)?;
		run_cargo(&[
			"clippy",
			"--workspace",
			"--locked",
			"--all-targets",
			"--",
			"-D",
			"warnings",
		],)?;
		run_cargo(&[
			"nextest",
			"run",
			"--workspace",
			"--locked",
			"--no-tests",
			"pass",
		],)?;

		Command::new("cargo",)
			.env(
				"RUSTDOCFLAGS",
				"-Z unstable-options --enable-index-page",
			)
			.args([
				"doc",
				"--workspace",
				"--locked",
				"--no-deps",
				"--document-private-items",
			],)
			.run()?;

		run_cargo(&["fmt", "--all", "--", "--check",],)?;
		run_cargo(&[
			"check",
			"-p",
			"poison_girl_kernel",
			"--locked",
			"--target",
			"aarch64-unknown-none",
		],)?;
		run_cargo(&[
			"check",
			"-p",
			"poison_girl_loader",
			"--locked",
			"--target",
			"aarch64-unknown-uefi",
		],)?;
		X((),)
	}

	/// this is workspace build.
	/// not a package build
	fn build(&self,) -> PoisonGirlB<(),>
	{
		let args = self.interface.policy();
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

	fn clippy(&self,) -> PoisonGirlB<(),>
	{
		let args = self.interface.policy();
		PoisonGirlCrateChart::all_variants()
			.into_iter()
			.try_for_each(|at| self.ws().clippy_at_with(at, args,),)?;
		X((),)
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
		let args = self.interface.policy();
		PoisonGirlCrateChart::all_variants()
			.into_iter()
			.try_for_each(|at| self.ws().test_at_with(at, args,),)?;
		X((),)
	}

	/// this runs cargo fix for all crates in this repository
	fn fix(&self,) -> PoisonGirlB<(),>
	{
		let args = self.interface.policy();
		PoisonGirlCrateChart::all_variants()
			.into_iter()
			.try_for_each(|at| self.ws().fix_at_with(at, args,),)?;
		X((),)
	}
}

fn run_cargo(args: &[&str],) -> PoisonGirlB<(),>
{
	Command::new("cargo",).args(args,).run()
}
