use oso_dev_util::cargo::*;
use oso_dev_util::decl_manage::crate_::CrateInfo;
use oso_dev_util::fs::*;
use std::path::PathBuf;

#[test]
fn test_basic_functionality() {
	// Test basic enum functionality
	let build_mode = BuildMode::Debug;
	assert!(build_mode.is_debug());
	assert_eq!(build_mode.as_ref(), "Debug");

	let arch = Arch::Aarch64;
	assert!(arch.is_aarch_64());
	assert_eq!(arch.as_ref(), "Aarch64");
}

#[test]
fn test_cli_conversion() {
	let cli = Cli {
		build_mode:    Some(BuildMode::Debug,),
		feature_flags: None,
		arch:          Some(Arch::Riscv64,),
	};

	let opts = cli.to_opts();
	assert!(opts.build_mode.is_debug());
	assert!(opts.arch.is_riscv_64());
}

#[test]
fn test_compile_opt_trait() {
	let opts = Opts {
		build_mode:    BuildMode::Release,
		feature_flags: vec![],
		arch:          Arch::Aarch64,
	};

	let build_mode: String = opts.build_mode().into();
	assert_eq!(build_mode, "Release");

	let arch: String = opts.arch().into();
	assert_eq!(arch, "Aarch64");
}

#[test]
fn test_fs_functions_exist() {
	// Test that fs functions exist and return Results
	let _project_result = project_root();
	let _current_result = current_crate();

	// These functions may fail in test environment, but they should exist
}

#[test]
fn test_firmware_struct() {
	let firmware = Firmware {
		code: PathBuf::from("/test/code",),
		vars: PathBuf::from("/test/vars",),
	};

	assert_eq!(firmware.code, PathBuf::from("/test/code"));
	assert_eq!(firmware.vars, PathBuf::from("/test/vars"));
}

#[test]
fn test_assets_struct() {
	let assets = Assets {
		firmware: Firmware {
			code: PathBuf::from("/ovmf/code",),
			vars: PathBuf::from("/ovmf/vars",),
		},
	};

	assert_eq!(assets.firmware.code, PathBuf::from("/ovmf/code"));
	assert_eq!(assets.firmware.vars, PathBuf::from("/ovmf/vars"));
}

#[test]
fn test_enum_defaults() {
	assert!(BuildMode::default().is_debug());
	assert!(Arch::default().is_aarch_64());
}

#[test]
fn test_enum_cloning() {
	let build_mode = BuildMode::Release;
	let cloned = build_mode;
	assert_eq!(build_mode, cloned);

	let arch = Arch::Riscv64;
	let cloned = arch;
	assert_eq!(arch, cloned);
}

#[test]
fn test_enum_equality() {
	assert_eq!(BuildMode::Debug, BuildMode::Debug);
	assert_ne!(BuildMode::Debug, BuildMode::Release);

	assert_eq!(Arch::Aarch64, Arch::Aarch64);
	assert_ne!(Arch::Aarch64, Arch::Riscv64);
}

#[test]
fn test_string_conversions() {
	use std::str::FromStr;

	// Test AsRefStr
	assert_eq!(BuildMode::Debug.as_ref(), "Debug");
	assert_eq!(Arch::Aarch64.as_ref(), "Aarch64");

	// Test FromStr
	assert_eq!(BuildMode::from_str("Debug").unwrap(), BuildMode::Debug);
	assert_eq!(Arch::from_str("Aarch64").unwrap(), Arch::Aarch64);
}

#[test]
fn test_is_methods() {
	// BuildMode
	assert!(BuildMode::Debug.is_debug());
	assert!(!BuildMode::Release.is_debug());

	// Arch
	assert!(Arch::Aarch64.is_aarch_64());
	assert!(Arch::Riscv64.is_riscv_64());
	assert!(!Arch::Aarch64.is_riscv_64());
	assert!(!Arch::Riscv64.is_aarch_64());
}

#[test]
fn test_comprehensive_enum_combinations() {
	// Test all possible combinations of enums
	use clap::ValueEnum;

	for build_mode in BuildMode::value_variants() {
		for arch in Arch::value_variants() {
			let opts = Opts {
				build_mode:    *build_mode,
				feature_flags: vec![],
				arch:          *arch,
			};

			// Test CompileOpt trait methods
			let build_mode_str: String = opts.build_mode().into();
			let arch_str: String = opts.arch().into();

			// Verify string representations
			assert_eq!(build_mode_str, build_mode.as_ref());
			assert_eq!(arch_str, arch.as_ref());
		}
	}
}

#[test]
fn test_cli_all_combinations() {
	// Test CLI with all possible combinations

	for build_mode in
		[None, Some(BuildMode::Debug,), Some(BuildMode::Release,),]
	{
		for arch in [None, Some(Arch::Aarch64,), Some(Arch::Riscv64,),] {
			let cli = Cli { build_mode, feature_flags: Some(vec![],), arch, };

			let opts = cli.to_opts();

			// Verify defaults are applied correctly
			match build_mode {
				Some(bm,) => assert_eq!(opts.build_mode, bm),
				None => assert_eq!(opts.build_mode, BuildMode::default()),
			}

			match arch {
				Some(a,) => assert_eq!(opts.arch, a),
				None => assert_eq!(opts.arch, Arch::default()),
			}
		}
	}
}

#[test]
fn test_error_cases() {
	// Test error handling in string parsing
	use std::str::FromStr;

	// Invalid enum values should fail
	assert!(BuildMode::from_str("Invalid").is_err());
	assert!(BuildMode::from_str("release").is_err()); // case sensitive
	assert!(BuildMode::from_str("").is_err());

	assert!(Arch::from_str("x86_64").is_err());
	assert!(Arch::from_str("arm64").is_err());
	assert!(Arch::from_str("aarch64").is_err()); // case sensitive
	assert!(Arch::from_str("").is_err());
}

#[test]
fn test_memory_efficiency() {
	// Test that enums are memory efficient
	use std::mem;

	// Enums should be small
	assert!(mem::size_of::<BuildMode,>() <= 8);
	assert!(mem::size_of::<Arch,>() <= 8);
}

#[test]
fn test_thread_safety() {
	// Test that enums can be used across threads
	use std::sync::Arc;
	use std::thread;

	let build_mode = Arc::new(BuildMode::Debug,);
	let arch = Arc::new(Arch::Aarch64,);

	let handles: Vec<_,> = (0..5)
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
fn test_fs_integration() {
	// Test filesystem functions integration
	let project_result = project_root();
	let current_result = current_crate();

	// Functions should return consistent types
	match (project_result, current_result,) {
		(Ok(project_crate,), Ok(current_crate,),) => {
			// Both should be valid OsoCrate instances
			let project_path = project_crate.path();
			let current_path = current_crate.path();

			// Paths should be valid
			assert!(!project_path.as_os_str().is_empty());
			assert!(!current_path.as_os_str().is_empty());

			// Should be able to clone
			let _project_clone = project_crate.clone();
			let _current_clone = current_crate.clone();

			// Should be able to compare
			let project_crate2 = project_crate.clone();
			assert_eq!(project_crate, project_crate2);
		},
		(Err(project_err,), _,) => {
			// Error should be meaningful
			let error_msg = project_err.to_string();
			assert!(!error_msg.is_empty());
		},
		(_, Err(current_err,),) => {
			// Error should be meaningful
			let error_msg = current_err.to_string();
			assert!(!error_msg.is_empty());
		},
	}
}

#[test]
fn test_debug_formatting() {
	// Test that all types have useful Debug output
	let build_mode = BuildMode::Debug;
	let debug_str = format!("{:?}", build_mode);
	assert!(debug_str.contains("Debug"));

	let arch = Arch::Aarch64;
	let debug_str = format!("{:?}", arch);
	assert!(debug_str.contains("Aarch64"));

	assert!(!debug_str.is_empty());

	let firmware = Firmware {
		code: PathBuf::from("/test/code.fd",),
		vars: PathBuf::from("/test/vars.fd",),
	};
	let debug_str = format!("{:?}", firmware);
	assert!(debug_str.contains("Firmware"));
	assert!(debug_str.contains("code.fd"));
	assert!(debug_str.contains("vars.fd"));
}

#[test]
fn test_value_enum_completeness() {
	// Test that ValueEnum implementations are complete
	use clap::ValueEnum;

	// Test that all variants are included
	let build_mode_variants = BuildMode::value_variants();
	assert_eq!(build_mode_variants.len(), 2);
	assert!(build_mode_variants.contains(&BuildMode::Debug));
	assert!(build_mode_variants.contains(&BuildMode::Release));

	let arch_variants = Arch::value_variants();
	assert_eq!(arch_variants.len(), 2);
	assert!(arch_variants.contains(&Arch::Aarch64));
	assert!(arch_variants.contains(&Arch::Riscv64));
}

#[test]
fn test_feature_flags_empty_enum() {
	// Test that Feature enum is properly empty
	// Note: Feature enum doesn't implement ValueEnum since it's empty

	// Should be able to create empty vectors
	let features: Vec<Feature,> = vec![];
	assert!(features.is_empty());

	// Should work in Opts
	let opts = Opts {
		build_mode:    BuildMode::Debug,
		feature_flags: features,
		arch:          Arch::default(),
	};

	let returned_features = opts.feature_flags();
	assert!(returned_features.is_empty());
}

#[test]
fn test_path_handling() {
	// Test PathBuf handling in Firmware and Assets
	let code_path = PathBuf::from("/usr/share/ovmf/OVMF_CODE.fd",);
	let vars_path = PathBuf::from("/usr/share/ovmf/OVMF_VARS.fd",);

	let firmware =
		Firmware { code: code_path.clone(), vars: vars_path.clone(), };

	let assets = Assets { firmware, };

	// Test that paths are preserved
	assert_eq!(assets.firmware.code, code_path);
	assert_eq!(assets.firmware.vars, vars_path);

	// Test that paths can be manipulated
	let code_parent = assets.firmware.code.parent();
	let vars_parent = assets.firmware.vars.parent();
	assert_eq!(code_parent, vars_parent);

	// Test path string conversion
	let code_str = assets.firmware.code.to_string_lossy();
	let vars_str = assets.firmware.vars.to_string_lossy();
	assert!(code_str.contains("OVMF_CODE.fd"));
	assert!(vars_str.contains("OVMF_VARS.fd"));
}
