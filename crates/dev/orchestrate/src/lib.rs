#![feature(exit_status_error)]
#![feature(proc_macro_hygiene)]

use {
	crate::decl_manage::{
		CargoCrate, PoisonGirlCargoInterface, crate_::PoisonGirlCrateChart,
	},
	poison_girl_dev_cargo::{Arch, BuildMode, Opts},
	poison_girl_dev_error::{PathNotFound, PoisonGirlB, X, Y},
};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
/// flowchart TD
/// A[Crate] --> B[Workspace]
/// A --> C[Package]
/// B --> D[CrateBase]
/// C --> D
/// ```
pub mod decl_manage;

pub fn check_poison_girl_kernel(
	arch: Arch,
	build_mode: BuildMode,
) -> PoisonGirlB<(),>
{
	let kernel_crate = PoisonGirlCargoInterface::new(
		PoisonGirlCrateChart::Kernel,
		Opts { arch, build_mode, ..Default::default() },
	);
	// Construct the expected path to the kernel ELF file
	let target_path = kernel_crate.build_artifact()?;

	// Check if the file exists and return appropriate result
	if target_path.exists() {
		X((),)
	} else {
		Y(PathNotFound(target_path.to_str().unwrap().to_string(),),)
	}?;
	X((),)
}
