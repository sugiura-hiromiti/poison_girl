#![feature(proc_macro_hygiene)]

use {
	crate::decl_manage::{
		PoisonGirlCargoInterface, crate_::PoisonGirlCrateChart,
	},
	poison_girl_dev_error::{PathNotFound, PoisonGirlB, X, Y},
};

pub use {
	crate::{
		cli_interface::{
			AsCargoOpt, Cli, CliCommand, CliCommandDiscriminants, CompileOpt,
			FixArgs, Policy,
		},
		policy::build_artifact_policy::{
			BuildArtifact, BuildArtifactPolicyResolver,
		},
	},
	poison_girl_dev_cargo::{Arch, BuildMode},
};

pub mod cli_interface;
#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
/// flowchart TD
/// A[Crate] --> B[Workspace]
/// A --> C[Package]
/// B --> D[CrateBase]
/// C --> D
/// ```
pub mod decl_manage;
pub(crate) mod policy;

#[deprecated]
pub fn check_poison_girl_kernel(
	arch: Arch,
	build_mode: BuildMode,
) -> PoisonGirlB<(),>
{
	let kernel_crate = PoisonGirlCargoInterface::new(
		PoisonGirlCrateChart::Kernel,
		Policy::from_arch_build_mode(arch, build_mode,),
	);
	// Construct the expected path to the kernel ELF file
	let target_path = kernel_crate.build_artifact_policy()?.path();

	// Check if the file exists and return appropriate result
	if target_path.exists() {
		X((),)
	} else {
		Y(PathNotFound(target_path.display().to_string(),),)
	}?;
	X((),)
}
