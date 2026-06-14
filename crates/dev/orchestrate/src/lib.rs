#![feature(proc_macro_hygiene)]
#![feature(iterator_try_collect)]

use {
	crate::decl_manage::{
		OrchestrationResolver, PoisonGirlCargoInterface,
		crate_::PoisonGirlCrateChart,
	},
	poison_girl_dev_cargo::{Arch, BuildMode, CliCommand, Opts},
	poison_girl_dev_error::{PathNotFound, PoisonGirlB, X, Y},
	poison_girl_macro_def_features::features,
	std::{path::PathBuf, str::FromStr},
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
	let target_path = kernel_crate.build_artifact()?.path();

	// Check if the file exists and return appropriate result
	if target_path.exists() {
		X((),)
	} else {
		Y(PathNotFound(target_path.display().to_string(),),)
	}?;
	X((),)
}

#[features(PoisonGirlCrateChart)]
#[derive(
	strum_macros::AsRefStr,
	strum_macros::EnumIs,
	strum_macros::EnumString,
	Clone,
)]
#[strum(serialize_all = "snake_case")]
pub enum Feature {}

impl AsCargoOpt for Vec<Feature,>
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		if self.is_empty() {
			return vec![];
		}

		vec![
			"-F".to_string(),
			self.iter().map(|f| f.as_ref(),).collect::<Vec<_,>>().join(",",),
		]
	}
}

pub trait CompileOpt
{
	fn build_mode(&self,) -> impl Into<String,>;
	fn feature_flags(&self,) -> Vec<impl Into<String,>,>;
}

impl CompileOpt for Opts
{
	fn build_mode(&self,) -> impl Into<String,>
	{
		self.build_mode.as_ref()
	}

	fn feature_flags(&self,) -> Vec<impl Into<String,>,>
	{
		self.feature_flags.clone()
	}
}

pub trait AsCargoOpt
{
	type Out;
	fn as_cargo_opt(&self,) -> Self::Out;
}

impl AsCargoOpt for Opts
{
	type Out = PoisonGirlB<Vec<String,>,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let Self { command, build_mode, feature_flags, lock_deps, .. } = self;
		let Some(command,) = command.as_cargo_opt() else { return X(vec![],) };
		let build_mode = build_mode.as_cargo_opt();
		let feature_flags = feature_flags
			.iter()
			.map(|s| Feature::from_str(s,),)
			.try_collect::<Vec<_,>>()?
			.as_cargo_opt();
		// single architecture info itself is useless
		// target tuple is truth
		// let arch = arch.as_cargo_opt();
		let lock_deps =
			if *lock_deps { Some("--locked".to_string(),) } else { None };

		X(std::iter::once(command,)
			.chain(build_mode,)
			.chain(feature_flags,)
			.chain(lock_deps,)
			.collect(),)
	}
}

impl AsCargoOpt for BuildMode
{
	type Out = Option<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		match self {
			Self::Release => Some("-r".to_string(),),
			Self::Debug => None,
		}
	}
}

impl AsCargoOpt for CliCommand
{
	type Out = Option<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		match self {
			Self::Check { .. } => None,
			Self::Fixture => None,
			_ => Some(self.as_ref().to_string(),),
		}
	}
}
