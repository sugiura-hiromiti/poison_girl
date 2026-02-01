//! # OSO Development Utilities
//!
//! A collection of development utilities and helper functions for the OSO
//! operating system project. This crate provides tools for workspace
//! management, command execution, and development workflow automation.
//!
//! ## Features
//!
//! - **Workspace Management**: Tools for managing multi-crate workspaces
//! - **Command Execution**: Enhanced command execution with better error
//!   handling and output formatting
//! - **Development Workflow**: Utilities to streamline the development process
//! - **Cross-platform Support**: Works across different operating systems
//!
//! ## Key Components
//!
//! ### Command Execution
//!
//! The [`Run`] trait provides enhanced command execution capabilities with:
//! - Colored output formatting
//! - Automatic error handling
//! - Inherited stdio streams
//! - Command display with arguments
//!
//! ### Workspace Management
//!
//! The workspace management system provides:
//! - Root directory detection
//! - Crate enumeration and management
//! - Workspace-wide operations
//!
//! ## Usage
//!
//! ### Basic Command Execution
//!
//! ```rust,no_run
//! use {oso_dev_util_helper::cli::Run, std::process::Command};
//!
//! // Execute a command with enhanced output
//! let mut cmd = Command::new("cargo",);
//! cmd.args(&["build", "--release",],);
//! cmd.run().expect("Build failed",);
//! ```
//!
//! ### Workspace Operations
//!
//! ```rust,ignore
//! use oso_dev_util::{OsoWorkspace, OsoWorkspaceManager};
//!
//! let workspace = OsoWorkspaceManager::new();
//! let root = workspace.root();
//! let crates = workspace.crates();
//!
//! println!("Workspace root: {}", root.display());
//! for crate_path in crates {
//!     println!("Crate: {}", crate_path.display());
//! }
//! ```
//!
//! ## Dependencies
//!
//! - [`anyhow`]: Error handling and context
//! - [`colored`]: Terminal color output
//! - [`toml`]: TOML configuration file parsing

#![feature(exit_status_error)]
#![feature(proc_macro_hygiene)]
#![feature(if_let_guard)]

pub mod cargo;
#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
/// flowchart TD
/// A[Crate] --> B[Workspace]
/// A --> C[Package]
/// B --> D[CrateBase]
/// C --> D
/// ```
pub mod decl_manage;
pub mod fs;

/// The path to the oso_dev_util crate manifest, set at compile time
pub const OSO_DEV_UTIL_PATH: &str = std::env!("CARGO_MANIFEST_PATH");

#[cfg(test)]
mod tests
{
	use crate::cargo::Arch;

	use super::*;

	#[test]
	fn test_oso_dev_util_path_constant()
	{
		// Test that the OSO_DEV_UTIL_PATH constant is set and valid
		assert!(OSO_DEV_UTIL_PATH.contains("Cargo.toml"));

		// Verify the path exists
		let path = std::path::Path::new(OSO_DEV_UTIL_PATH,);
		assert!(
			path.exists(),
			"OSO_DEV_UTIL_PATH should point to an existing file"
		);
		assert!(path.is_file(), "OSO_DEV_UTIL_PATH should point to a file");
	}

	#[test]
	fn test_module_accessibility()
	{
		// Test that all public modules are accessible
		// This is a compile-time test - if it compiles, the modules are
		// accessible

		// Test cargo module
		let _build_mode = cargo::BuildMode::Debug;
		let _arch = cargo::Arch::Aarch64;

		// Test that we can access the fs module functions
		// Note: These might fail in test environment, but we test they're
		// callable
		let _project_root_result = fs::project_root();
		let _current_crate_result = fs::current_crate();
	}

	#[test]
	fn test_module_structure()
	{
		// Test that the expected module structure exists
		// This is primarily a compile-time test

		// Verify we can create instances of key types
		use cargo::{Arch, BuildMode};

		let build_mode = BuildMode::Debug;
		assert!(build_mode.is_debug());

		let arch = Arch::Aarch64;
		assert!(arch.is_aarch_64());
	}

	#[test]
	fn test_feature_flags()
	{
		// Test that feature flags are accessible
		use cargo::Feature;

		// Since Feature is an empty enum with the #[features] attribute,
		// we can't create instances, but we can verify it exists
		// This is primarily a compile-time test

		// Test that we can reference the Feature type
		let _feature_type = std::marker::PhantomData::<Feature,>;
	}

	#[test]
	fn test_compile_opt_trait()
	{
		// Test the CompileOpt trait functionality
		use cargo::{BuildMode, CompileOpt, Feature, Opts};

		let opts = Opts {
			build_mode:    BuildMode::Debug,
			feature_flags: Vec::<Feature,>::new(),
			arch:          Arch::default(),
		};

		// Test trait methods
		let build_mode: String = opts.build_mode().into();
		assert_eq!(build_mode, "Debug");

		let feature_flags = opts.feature_flags();
		assert!(feature_flags.is_empty());

		let arch: String = opts.arch().into();
		assert_eq!(arch, "Aarch64");
	}

	#[test]
	fn test_cli_to_opts_conversion()
	{
		// Test CLI to Opts conversion
		use cargo::{Arch, BuildMode, Cli};

		let cli = Cli {
			build_mode:    Some(BuildMode::Release,),
			feature_flags: None,
			arch:          Some(Arch::Riscv64,),
		};

		let opts = cli.to_opts();
		assert!(opts.build_mode.is_release());
		assert!(opts.feature_flags.is_empty());
		assert!(opts.arch.is_riscv_64());
	}

	#[test]
	fn test_cli_defaults()
	{
		// Test CLI with default values
		use cargo::Cli;

		let cli = Cli {
			build_mode:    None,
			feature_flags: None,
			arch:          None,
		};

		let opts = cli.to_opts();
		assert!(opts.build_mode.is_debug()); // Default should be Debug
		assert!(opts.feature_flags.is_empty());
		assert!(opts.arch.is_aarch_64()); // Default should be Aarch64
	}

	#[test]
	fn test_firmware_structure()
	{
		// Test Firmware struct
		use {cargo::Firmware, std::path::PathBuf};

		let firmware = Firmware {
			code: PathBuf::from("/path/to/code",),
			vars: PathBuf::from("/path/to/vars",),
		};

		// Test Debug implementation
		let debug_string = format!("{:?}", firmware);
		assert!(debug_string.contains("Firmware"));
		assert!(debug_string.contains("/path/to/code"));
		assert!(debug_string.contains("/path/to/vars"));
	}

	#[test]
	fn test_assets_structure()
	{
		// Test Assets struct
		use {
			cargo::{Assets, Firmware},
			std::path::PathBuf,
		};

		let assets = Assets {
			firmware: Firmware {
				code: PathBuf::from("/ovmf/code",),
				vars: PathBuf::from("/ovmf/vars",),
			},
			host:     cargo::Runtime::Linux,
		};

		// Verify the structure exists and is accessible
		assert_eq!(assets.firmware.code, PathBuf::from("/ovmf/code"));
		assert_eq!(assets.firmware.vars, PathBuf::from("/ovmf/vars"));
	}

	#[test]
	fn test_enum_string_conversions()
	{
		// Test AsRefStr implementations
		use cargo::{Arch, BuildMode};

		assert_eq!(BuildMode::Debug.as_ref(), "Debug");
		assert_eq!(BuildMode::Release.as_ref(), "Release");

		assert_eq!(Arch::Aarch64.as_ref(), "Aarch64");
		assert_eq!(Arch::Riscv64.as_ref(), "Riscv64");
	}

	#[test]
	fn test_enum_is_methods()
	{
		// Test EnumIs implementations
		use cargo::{Arch, BuildMode};

		// BuildMode
		assert!(BuildMode::Debug.is_debug());
		assert!(!BuildMode::Debug.is_release());
		assert!(BuildMode::Release.is_release());
		assert!(!BuildMode::Release.is_debug());

		// Arch
		assert!(Arch::Aarch64.is_aarch_64());
		assert!(!Arch::Aarch64.is_riscv_64());
		assert!(Arch::Riscv64.is_riscv_64());
		assert!(!Arch::Riscv64.is_aarch_64());
	}

	#[test]
	fn test_clone_implementations()
	{
		// Test Clone implementations
		use cargo::{Arch, BuildMode};

		let build_mode = BuildMode::Debug;
		let cloned_build_mode = build_mode;
		assert_eq!(build_mode.as_ref(), cloned_build_mode.as_ref());

		let arch = Arch::Aarch64;
		let cloned_arch = arch;
		assert_eq!(arch.as_ref(), cloned_arch.as_ref());
	}

	#[test]
	fn test_default_implementations()
	{
		// Test Default implementations
		use cargo::{Arch, BuildMode};

		let default_build_mode = BuildMode::default();
		assert!(default_build_mode.is_debug());

		let default_arch = Arch::default();
		assert!(default_arch.is_aarch_64());
	}

	#[test]
	fn test_value_enum_implementations()
	{
		// Test that ValueEnum is implemented for CLI enums
		use {
			cargo::{Arch, BuildMode},
			clap::ValueEnum,
		};

		// Test that we can get possible values
		let build_mode_values = BuildMode::value_variants();
		assert_eq!(build_mode_values.len(), 2);
		assert!(build_mode_values.contains(&BuildMode::Debug));
		assert!(build_mode_values.contains(&BuildMode::Release));

		let arch_values = Arch::value_variants();
		assert_eq!(arch_values.len(), 2);
		assert!(arch_values.contains(&Arch::Aarch64));
		assert!(arch_values.contains(&Arch::Riscv64));
	}

	#[test]
	fn test_oso_dev_util_path_validation()
	{
		// Test that the OSO_DEV_UTIL_PATH constant points to a valid Cargo.toml
		let path = std::path::Path::new(OSO_DEV_UTIL_PATH,);

		// Should be an absolute path
		assert!(path.is_absolute());

		// Should end with Cargo.toml
		assert_eq!(path.file_name().unwrap(), "Cargo.toml");

		// Should be readable
		let content = std::fs::read_to_string(path,)
			.expect("Should be able to read Cargo.toml",);
		assert!(content.contains("[package]"));
		assert!(content.contains("oso_dev_util"));
	}

	#[test]
	fn test_module_imports()
	{
		// Test that all modules can be imported without conflicts
		use {cargo::*, fs::*};

		// Test that we can create instances of key types
		let _build_mode = BuildMode::Debug;
		let _arch = Arch::Aarch64;

		// Test that functions are accessible
		let _project_result = project_root();
		let _current_result = current_crate();
	}

	#[test]
	fn test_feature_flag_compilation()
	{
		// Test that we can reference the Feature enum even if empty
		use cargo::Feature;
		let _features: Vec<Feature,> = vec![];
	}

	#[test]
	fn test_type_system_constraints()
	{
		// Test that the type system enforces expected constraints
		use cargo::{Arch, BuildMode, CompileOpt, Feature, Opts};

		// Test that all enums implement required traits
		fn test_enum_traits<T,>(_value: T,)
		where T: Clone + Copy + PartialEq + Eq + std::fmt::Debug + Default
		{
			// If this compiles, the traits are implemented
		}

		test_enum_traits(BuildMode::Debug,);
		test_enum_traits(Arch::Aarch64,);

		// Test that Opts can be constructed with all combinations
		let all_build_modes = [BuildMode::Debug, BuildMode::Release,];
		let all_archs = [Arch::Aarch64, Arch::Riscv64,];

		for &build_mode in &all_build_modes {
			for &arch in &all_archs {
				let opts = Opts {
					build_mode,
					feature_flags: Vec::<Feature,>::new(),
					arch,
				};

				// Test CompileOpt trait methods
				let _build_mode_str: String = opts.build_mode().into();
				let _arch_str: String = opts.arch().into();
				let _features = opts.feature_flags();
			}
		}
	}

	#[test]
	fn test_path_handling()
	{
		// Test path handling throughout the crate
		use std::path::{Path, PathBuf};

		// Test that OSO_DEV_UTIL_PATH is a valid path
		let manifest_path = Path::new(OSO_DEV_UTIL_PATH,);
		assert!(manifest_path.exists());
		assert!(manifest_path.is_file());

		// Test path operations
		let parent =
			manifest_path.parent().expect("Should have parent directory",);
		assert!(parent.exists());
		assert!(parent.is_dir());

		// Test that we can construct relative paths
		let relative_path = PathBuf::from("./Cargo.toml",);
		assert!(!relative_path.is_absolute());

		// Test path conversion
		let path_str = manifest_path.to_string_lossy();
		assert!(path_str.contains("Cargo.toml"));
	}

	#[test]
	fn test_string_conversions_comprehensive()
	{
		// Test all string conversion patterns used in the crate
		use {
			cargo::{Arch, BuildMode},
			std::str::FromStr,
		};

		// Test round-trip conversions for all enum variants
		let build_modes = [BuildMode::Debug, BuildMode::Release,];
		for mode in build_modes {
			let as_str = mode.as_ref();
			let parsed = BuildMode::from_str(as_str,).unwrap();
			assert_eq!(mode, parsed);
		}

		let arch_variants = [Arch::Aarch64, Arch::Riscv64,];
		for arch in arch_variants {
			let as_str = arch.as_ref();
			let parsed = Arch::from_str(as_str,).unwrap();
			assert_eq!(arch, parsed);
		}

		// Test invalid string parsing
		assert!(BuildMode::from_str("Invalid").is_err());
		assert!(Arch::from_str("x86_64").is_err());
	}

	#[test]
	fn test_memory_safety()
	{
		// Test that the crate handles memory safely
		use cargo::{Arch, BuildMode, Feature, Opts};

		// Test that we can create and drop many instances without issues
		let mut opts_vec = Vec::new();
		for i in 0..1000 {
			let opts = Opts {
				build_mode:    if i % 2 == 0 {
					BuildMode::Debug
				} else {
					BuildMode::Release
				},
				feature_flags: Vec::<Feature,>::new(),
				arch:          if i % 2 == 0 {
					Arch::Aarch64
				} else {
					Arch::Riscv64
				},
			};
			opts_vec.push(opts,);
		}

		// Test that we can access all instances
		assert_eq!(opts_vec.len(), 1000);
	}

	#[test]
	fn test_concurrent_access()
	{
		// Test that constants and static data can be accessed concurrently
		use std::thread;

		let handles: Vec<_,> = (0..10)
			.map(|_| {
				thread::spawn(|| {
					// Access the constant from multiple threads
					let path = OSO_DEV_UTIL_PATH;
					assert!(path.contains("Cargo.toml"));

					// Create enum instances
					let build_mode = cargo::BuildMode::Debug;
					assert!(build_mode.is_debug());
				},)
			},)
			.collect();

		// Wait for all threads to complete
		for handle in handles {
			handle.join().unwrap();
		}
	}

	#[test]
	fn test_documentation_examples()
	{
		// Test that code examples from documentation work
		use cargo::{Arch, BuildMode, CompileOpt, Feature, Opts};

		// Example from CompileOpt documentation
		let opts = Opts {
			build_mode:    BuildMode::Debug,
			feature_flags: Vec::<Feature,>::new(),
			arch:          Arch::Aarch64,
		};

		let build_mode: String = opts.build_mode().into();
		assert_eq!(build_mode, "Debug");

		let feature_flags = opts.feature_flags();
		assert!(feature_flags.is_empty());

		let arch: String = opts.arch().into();
		assert_eq!(arch, "Aarch64");
	}
}
