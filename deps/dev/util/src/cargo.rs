use anyhow::Context as _;
use anyhow::Result as Rslt;
use clap::Parser;
use oso_proc_macro::features;
use ovmf_prebuilt::FileType;
use ovmf_prebuilt::Prebuilt;
use ovmf_prebuilt::Source;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use strum_macros::Display;

pub trait CompileOpt {
	fn build_mode(&self,) -> impl Into<String,>;
	fn feature_flags(&self,) -> Vec<impl Into<String,>,>;
	fn arch(&self,) -> impl Into<String,>;
}

#[features]
#[derive(
	strum_macros::AsRefStr,
	strum_macros::EnumIs,
	strum_macros::EnumString,
	Clone,
)]
pub enum Feature {}

pub struct Opts {
	pub build_mode:    BuildMode,
	pub feature_flags: Vec<Feature,>,
	pub arch:          Arch,
}

impl Default for Opts {
	fn default() -> Self {
		Self::new()
	}
}

impl Opts {
	pub fn new() -> Self {
		Cli::parse().to_opts()
	}
}

impl CompileOpt for Opts {
	fn build_mode(&self,) -> impl Into<String,> {
		self.build_mode.as_ref()
	}

	fn feature_flags(&self,) -> Vec<impl Into<String,>,> {
		self.feature_flags.iter().map(|f| f.as_ref(),).collect()
	}

	fn arch(&self,) -> impl Into<String,> {
		self.arch.as_ref().replace("_", "",)
	}
}

#[derive(clap::Parser,)]
pub struct Cli {
	#[arg(value_enum, short)]
	pub build_mode:    Option<BuildMode,>,
	#[arg(short)]
	pub feature_flags: Option<Vec<Feature,>,>,
	#[arg(short)]
	pub arch:          Option<Arch,>,
}

impl Cli {
	pub fn to_opts(self,) -> Opts {
		Opts {
			build_mode:    self.build_mode.unwrap_or_default(),
			feature_flags: self.feature_flags.unwrap_or_default(),
			arch:          self.arch.unwrap_or_default(),
		}
	}
}

#[derive(
	Clone,
	Copy,
	clap::ValueEnum,
	Default,
	strum_macros::AsRefStr,
	strum_macros::EnumIs,
	strum_macros::EnumString,
	PartialEq,
	Eq,
	Debug,
	Display,
)]
pub enum BuildMode {
	Release,
	#[default]
	Debug,
}

pub enum Runtime {
	Mac,
	Linux,
	Efi,
	Oso,
}

impl Runtime {
	pub fn host() -> Rslt<Self,> {
		host_tuple()?
			.split('-',)
			.next()
			.context(
				"target tuple for host does not include `-`. that is not \
				 usual.",
			)
			.map(Runtime::from_str,)
	}
}

impl Runtime {
	fn from_str(value: &str,) -> Self {
		match value {
			"mac" | "darwin" => Self::Mac,
			"linux" => Self::Linux,
			"oso" => Self::Oso,
			"efi" => Self::Efi,
			a => unimplemented!("{a} is not supported runtime"),
		}
	}
}

pub struct Assets {
	pub firmware: Firmware,
	pub host:     Runtime,
}

impl Assets {
	pub fn new(arch: Arch,) -> Rslt<Self,> {
		let firmware = Firmware::new(arch,)?;
		Ok(Self { firmware, host: Runtime::host()?, },)
	}
}

/// Manages OVMF firmware files for UEFI boot
#[derive(Debug,)]
pub struct Firmware {
	/// Path to the OVMF code file
	pub code: PathBuf,
	/// Path to the OVMF variables file
	pub vars: PathBuf,
}

impl Firmware {
	/// Creates a new Firmware instance for the specified architecture
	///
	/// Downloads the latest OVMF firmware files if they don't exist.
	///
	/// # Parameters
	///
	/// * `arch` - The target architecture
	///
	/// # Returns
	///
	/// A new Firmware instance or an error if initialization fails
	pub fn new(arch: Arch,) -> Rslt<Self,> {
		let path = PathBuf::from_str("/tmp/",)?;
		let ovmf_files = Prebuilt::fetch(Source::LATEST, path,)?;
		let code = ovmf_files.get_file(arch.into(), FileType::Code,);
		let vars = ovmf_files.get_file(arch.into(), FileType::Vars,);
		Ok(Self { code, vars, },)
	}

	/// Gets the path to the OVMF code file
	///
	/// # Returns
	///
	/// A reference to the path to the OVMF code file
	pub fn code(&self,) -> &PathBuf {
		&self.code
	}

	/// Gets the path to the OVMF variables file
	///
	/// # Returns
	///
	/// A reference to the path to the OVMF variables file
	pub fn vars(&self,) -> &PathBuf {
		&self.vars
	}
}

impl From<Arch,> for ovmf_prebuilt::Arch {
	fn from(value: Arch,) -> Self {
		match value {
			Arch::Aarch64 => ovmf_prebuilt::Arch::Aarch64,
			Arch::Riscv64 => ovmf_prebuilt::Arch::Riscv64,
		}
	}
}

#[derive(
	Default,
	strum_macros::AsRefStr,
	strum_macros::EnumIs,
	strum_macros::EnumString,
	Clone,
	Copy,
	clap::ValueEnum,
	PartialEq,
	Eq,
	Debug,
	Display,
)]
pub enum Arch {
	#[default]
	Aarch64,
	Riscv64,
}

impl Arch {
	/// Gets the boot file name for the architecture
	///
	/// # Returns
	///
	/// The boot file name (e.g., "bootaa64.efi" for aarch64)
	pub fn boot_file_name(&self,) -> &str {
		match self {
			Self::Aarch64 => "bootaa64.efi",
			Self::Riscv64 => "bootriscv64.efi",
		}
	}
}

pub fn host_tuple() -> Rslt<String,> {
	let target = Command::new("rustc",).arg("-vV",).output()?.stdout;
	let target = String::from_utf8(target,)?;
	target
		.lines()
		.find_map(|l| {
			if l.contains("host: ",) {
				Some(l.replace("host: ", "",).to_string(),)
			} else {
				None
			}
		},)
		.context("can't get host target tuple",)
}

#[cfg(test)]
mod tests {
	use super::*;
	use proptest::prelude::*;
	use std::str::FromStr;

	#[test]
	fn test_build_mode_default() {
		let default_mode = BuildMode::default();
		assert!(default_mode.is_debug());
		assert_eq!(default_mode.as_ref(), "Debug");
	}

	#[test]
	fn test_build_mode_variants() {
		assert!(BuildMode::Debug.is_debug());
		assert!(!BuildMode::Debug.is_release());
		assert!(BuildMode::Release.is_release());
		assert!(!BuildMode::Release.is_debug());
	}

	#[test]
	fn test_build_mode_string_conversion() {
		assert_eq!(BuildMode::Debug.as_ref(), "Debug");
		assert_eq!(BuildMode::Release.as_ref(), "Release");
	}

	#[test]
	fn test_build_mode_from_string() {
		assert_eq!(BuildMode::from_str("Debug").unwrap(), BuildMode::Debug);
		assert_eq!(BuildMode::from_str("Release").unwrap(), BuildMode::Release);
		assert!(BuildMode::from_str("Invalid").is_err());
	}

	#[test]
	fn test_arch_default() {
		let default_arch = Arch::default();
		assert!(default_arch.is_aarch_64());
		assert_eq!(default_arch.as_ref(), "Aarch64");
	}

	#[test]
	fn test_arch_variants() {
		assert!(Arch::Aarch64.is_aarch_64());
		assert!(Arch::Riscv64.is_riscv_64());

		assert!(!Arch::Aarch64.is_riscv_64());
		assert!(!Arch::Riscv64.is_aarch_64());
	}

	#[test]
	fn test_arch_string_conversion() {
		assert_eq!(Arch::Aarch64.as_ref(), "Aarch64");
		assert_eq!(Arch::Riscv64.as_ref(), "Riscv64");
	}

	#[test]
	fn test_arch_from_string() {
		assert_eq!(Arch::from_str("Aarch64").unwrap(), Arch::Aarch64);
		assert_eq!(Arch::from_str("Riscv64").unwrap(), Arch::Riscv64);
		assert!(Arch::from_str("x86_64").is_err());
	}

	#[test]
	fn test_cli_to_opts_with_values() {
		let cli = Cli {
			build_mode:    Some(BuildMode::Release,),
			feature_flags: Some(vec![],),
			arch:          Some(Arch::Riscv64,),
		};

		let opts = cli.to_opts();
		assert!(opts.build_mode.is_release());
		assert!(opts.feature_flags.is_empty());
	}

	#[test]
	fn test_cli_to_opts_with_defaults() {
		let cli = Cli {
			build_mode:    None,
			feature_flags: None,
			arch:          None,
		};

		let opts = cli.to_opts();
		assert!(opts.build_mode.is_debug());
		assert!(opts.feature_flags.is_empty());
	}

	#[test]
	fn test_compile_opt_implementation() {
		let opts = Opts {
			build_mode:    BuildMode::Release,
			feature_flags: vec![],
			arch:          Arch::Riscv64,
		};

		let build_mode: String = opts.build_mode().into();
		assert_eq!(build_mode, "Release");

		let feature_flags = opts.feature_flags();
		assert!(feature_flags.is_empty());

		let arch: String = opts.arch().into();
		assert_eq!(arch, "Riscv64");
	}

	#[test]
	fn test_firmware_creation() {
		let firmware = Firmware {
			code: PathBuf::from("/path/to/ovmf_code.fd",),
			vars: PathBuf::from("/path/to/ovmf_vars.fd",),
		};

		assert_eq!(firmware.code, PathBuf::from("/path/to/ovmf_code.fd"));
		assert_eq!(firmware.vars, PathBuf::from("/path/to/ovmf_vars.fd"));
	}

	#[test]
	fn test_firmware_debug() {
		let firmware = Firmware {
			code: PathBuf::from("/code",),
			vars: PathBuf::from("/vars",),
		};

		let debug_str = format!("{:?}", firmware);
		assert!(debug_str.contains("Firmware"));
		assert!(debug_str.contains("/code"));
		assert!(debug_str.contains("/vars"));
	}

	#[test]
	fn test_assets_creation() {
		let assets = Assets {
			firmware: Firmware {
				code: PathBuf::from("/ovmf/code",),
				vars: PathBuf::from("/ovmf/vars",),
			},
			host:     Runtime::Mac,
		};

		assert_eq!(assets.firmware.code, PathBuf::from("/ovmf/code"));
		assert_eq!(assets.firmware.vars, PathBuf::from("/ovmf/vars"));
	}

	#[test]
	fn test_feature_enum_exists() {
		// Test that Feature enum exists and can be used in collections
		let features: Vec<Feature,> = vec![];
		assert!(features.is_empty());

		// Test that Feature implements required traits
		let _phantom: std::marker::PhantomData<Feature,> =
			std::marker::PhantomData;
	}

	// Property-based tests
	proptest! {
		#[test]
		fn test_build_mode_roundtrip(mode in prop::sample::select(vec![BuildMode::Debug, BuildMode::Release])) {
			let as_str = mode.as_ref();
			let parsed = BuildMode::from_str(as_str).unwrap();
			assert_eq!(mode, parsed);
		}

		#[test]
		fn test_arch_roundtrip(arch in prop::sample::select(vec![Arch::Aarch64, Arch::Riscv64])) {
			let as_str = arch.as_ref();
			let parsed = Arch::from_str(as_str).unwrap();
			assert_eq!(arch, parsed);
		}

		#[test]
		fn test_cli_opts_conversion_preserves_values(
			build_mode in prop::option::of(prop::sample::select(vec![BuildMode::Debug, BuildMode::Release])),
			arch in prop::option::of(prop::sample::select(vec![Arch::Aarch64, Arch::Riscv64]))
		) {
			let cli = Cli {
				build_mode,
				feature_flags: Some(vec![]),
				arch,
			};

			let opts = cli.to_opts();

			// Check that values are preserved or defaults are used
			match build_mode {
				Some(bm) => assert_eq!(opts.build_mode, bm),
				None => assert_eq!(opts.build_mode, BuildMode::default()),
			}


			match arch {
				Some(a) => assert_eq!(opts.arch, a),
				None => assert_eq!(opts.arch, Arch::default()),
			}
		}
	}

	#[test]
	fn test_enum_value_variants() {
		use clap::ValueEnum;

		// Test BuildMode variants
		let build_modes = BuildMode::value_variants();
		assert_eq!(build_modes.len(), 2);
		assert!(build_modes.contains(&BuildMode::Debug));
		assert!(build_modes.contains(&BuildMode::Release));

		// Test Arch variants
		let arch_variants = Arch::value_variants();
		assert_eq!(arch_variants.len(), 2);
		assert!(arch_variants.contains(&Arch::Aarch64));
		assert!(arch_variants.contains(&Arch::Riscv64));
	}

	#[test]
	fn test_partial_eq_implementations() {
		// Test that enums implement PartialEq correctly
		assert_eq!(BuildMode::Debug, BuildMode::Debug);
		assert_ne!(BuildMode::Debug, BuildMode::Release);

		assert_eq!(Arch::Aarch64, Arch::Aarch64);
		assert_ne!(Arch::Aarch64, Arch::Riscv64);
	}

	#[test]
	fn test_edge_cases() {
		// Test empty feature flags
		let opts = Opts {
			build_mode:    BuildMode::Debug,
			feature_flags: vec![],
			arch:          Arch::default(),
		};

		let flags = opts.feature_flags();
		assert!(flags.is_empty());
	}

	#[test]
	fn test_struct_field_access() {
		// Test that all struct fields are accessible
		let cli = Cli {
			build_mode:    Some(BuildMode::Debug,),
			feature_flags: Some(vec![],),
			arch:          Some(Arch::Riscv64,),
		};

		assert!(cli.build_mode.unwrap().is_debug());
		assert!(cli.feature_flags.unwrap().is_empty());
		assert!(cli.arch.unwrap().is_riscv_64());

		let opts = Opts {
			build_mode:    BuildMode::Release,
			feature_flags: vec![],
			arch:          Arch::Aarch64,
		};

		assert!(opts.build_mode.is_release());
		assert!(opts.feature_flags.is_empty());
		assert!(opts.arch.is_aarch_64());
	}

	#[test]
	fn test_enum_exhaustiveness() {
		// Test that we handle all enum variants
		use clap::ValueEnum;

		// Test that all BuildMode variants are covered
		for variant in BuildMode::value_variants() {
			match variant {
				BuildMode::Debug => assert!(variant.is_debug()),
				BuildMode::Release => assert!(variant.is_release()),
			}
		}

		// Test that all Arch variants are covered
		for variant in Arch::value_variants() {
			match variant {
				Arch::Aarch64 => assert!(variant.is_aarch_64()),
				Arch::Riscv64 => assert!(variant.is_riscv_64()),
			}
		}
	}

	#[test]
	fn test_debug_implementations() {
		// Test that Debug is implemented for all types
		let build_mode = BuildMode::Debug;
		let debug_str = format!("{:?}", build_mode);
		assert!(debug_str.contains("Debug"));

		let arch = Arch::Aarch64;
		let debug_str = format!("{:?}", arch);
		assert!(debug_str.contains("Aarch64"));

		assert!(!debug_str.is_empty());

		let firmware = Firmware {
			code: PathBuf::from("/test/code",),
			vars: PathBuf::from("/test/vars",),
		};
		let debug_str = format!("{:?}", firmware);
		assert!(debug_str.contains("Firmware"));
		assert!(debug_str.contains("/test/code"));
		assert!(debug_str.contains("/test/vars"));
	}

	#[test]
	fn test_memory_layout() {
		// Test that enums have expected memory layout
		use std::mem;

		// Enums should be small since they're Copy
		assert!(mem::size_of::<BuildMode,>() <= 8);
		assert!(mem::size_of::<Arch,>() <= 8);

		// Structs should have reasonable sizes
		assert!(mem::size_of::<Cli,>() <= 256);
	}

	#[test]
	fn test_serialization_compatibility() {
		// Test that string representations are stable
		// This is important for CLI compatibility

		// BuildMode strings should be stable
		assert_eq!(BuildMode::Debug.as_ref(), "Debug");
		assert_eq!(BuildMode::Release.as_ref(), "Release");

		// Arch strings should be stable
		assert_eq!(Arch::Aarch64.as_ref(), "Aarch64");
		assert_eq!(Arch::Riscv64.as_ref(), "Riscv64");
	}

	#[test]
	fn test_concurrent_access() {
		// Test that enums can be used concurrently
		use std::sync::Arc;
		use std::thread;

		let build_mode = Arc::new(BuildMode::Debug,);
		let arch = Arc::new(Arch::Aarch64,);

		let handles: Vec<_,> = (0..10)
			.map(|_| {
				let bm = Arc::clone(&build_mode,);
				let a = Arc::clone(&arch,);

				thread::spawn(move || {
					assert!(bm.is_debug());
					assert!(a.is_aarch_64());

					let _opts = Opts {
						build_mode:    *bm,
						feature_flags: vec![],
						arch:          *a,
					};
				},)
			},)
			.collect();

		for handle in handles {
			handle.join().unwrap();
		}
	}

	#[test]
	fn test_cli_parser_integration() {
		// Test that CLI parsing works with clap
		use clap::CommandFactory;

		// Test that we can create a parser
		let _parser = Cli::command();

		// Test default CLI
		let cli = Cli {
			build_mode:    None,
			feature_flags: None,
			arch:          None,
		};

		let opts = cli.to_opts();
		assert!(opts.build_mode.is_debug());
		assert!(opts.arch.is_aarch_64());
	}

	#[test]
	fn test_error_handling() {
		// Test error handling in string parsing
		use std::str::FromStr;

		// Test invalid BuildMode
		let result = BuildMode::from_str("InvalidMode",);
		assert!(result.is_err());

		// Test invalid Arch
		let result = Arch::from_str("x86_64",);
		assert!(result.is_err());

		// Test case sensitivity
		let result = BuildMode::from_str("debug",);
		assert!(result.is_err());

		let result = Arch::from_str("aarch64",);
		assert!(result.is_err());
	}

	#[test]
	fn test_feature_flags_empty() {
		// Test that Feature enum is empty as expected
		// Note: Feature enum doesn't implement ValueEnum since it's empty

		// Test that we can create empty vectors
		let features: Vec<Feature,> = vec![];
		assert!(features.is_empty());

		// Test in Opts
		let opts = Opts {
			build_mode:    BuildMode::Debug,
			feature_flags: features,
			arch:          Arch::default(),
		};

		let returned_features = opts.feature_flags();
		assert!(returned_features.is_empty());
	}

	#[test]
	fn test_assets_and_firmware() {
		// Test Assets and Firmware structs
		let firmware = Firmware {
			code: PathBuf::from("/ovmf/OVMF_CODE.fd",),
			vars: PathBuf::from("/ovmf/OVMF_VARS.fd",),
		};

		let assets = Assets { firmware, host: Runtime::host().unwrap(), };

		// Test field access
		assert_eq!(assets.firmware.code, PathBuf::from("/ovmf/OVMF_CODE.fd"));
		assert_eq!(assets.firmware.vars, PathBuf::from("/ovmf/OVMF_VARS.fd"));

		// Test Debug implementation
		let debug_str = format!("{:?}", assets.firmware);
		assert!(debug_str.contains("Firmware"));
		assert!(debug_str.contains("OVMF_CODE.fd"));
		assert!(debug_str.contains("OVMF_VARS.fd"));
	}
}
