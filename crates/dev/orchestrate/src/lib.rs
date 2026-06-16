#![feature(proc_macro_hygiene)]
#![feature(iterator_try_collect)]

use {
	crate::decl_manage::{
		OrchestrationResolver, PoisonGirlCargoInterface,
		crate_::PoisonGirlCrateChart,
	},
	clap::Parser,
	poison_girl_dev_cargo::{Arch, BuildMode, Cli, CliCommand},
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
		Task {
			opts: Opts { arch, build_mode, ..Default::default() },
			..Default::default()
		},
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

#[derive(Default, Clone,)]
pub struct Opts
{
	pub build_mode:    BuildMode,
	pub feature_flags: Vec<Feature,>,
	pub arch:          Arch,
	pub lock_deps:     bool,
	context:           ContextualOpts,
}

impl Opts
{
	fn fix_context_by(&self, context: ContextualOpts,) -> Self
	{
		let mut rslt = self.clone();
		rslt.context = context;
		rslt
	}

	pub fn allow_dirty(&mut self, allow: bool,)
	{
		self.context.allow_dirty = allow;
	}

	pub fn allow_staged(&mut self, allow: bool,)
	{
		self.context.allow_staged = allow;
	}

	pub fn workspace_op(&mut self, allow: bool,)
	{
		self.context.workspace = allow;
	}
}

/// this option should be determined within orchestration. not given by
/// user
#[derive(Default, Clone, Copy,)]
struct ContextualOpts
{
	allow_dirty:  bool,
	allow_staged: bool,
	workspace:    bool,
}

impl AsCargoOpt for ContextualOpts
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let Self { allow_dirty, allow_staged, workspace, } = self;
		let mut opt_list = vec![];

		if *allow_dirty {
			opt_list.push("--allow-dirty",);
		}

		if *allow_staged {
			opt_list.push("--allow-staged",);
		}

		if *workspace {
			opt_list.push("--workspace",);
		}

		opt_list.into_iter().map(|s| s.to_string(),).collect()
	}
}

#[derive(Default, Clone,)]
pub struct Task
{
	cmd:  CliCommand,
	opts: Opts,
}

impl Task
{
	pub fn from_cli(cli: Cli,) -> PoisonGirlB<Self,>
	{
		X(Self {
			cmd:  cli.command.unwrap_or_default(),
			opts: Opts {
				build_mode: cli.build_mode.unwrap_or_default(),
				feature_flags: cli
					.feature_flags
					.map(|features| -> PoisonGirlB<_,> {
						let rslt = features
							.into_iter()
							.map(|feature| Feature::from_str(&feature,),)
							.try_collect()?;
						X(rslt,)
					},)
					.unwrap_or(X(vec![],),)?,
				arch: cli.arch.unwrap_or_default(),
				lock_deps: cli.lock_deps,
				..Default::default()
			},
		},)
	}

	pub fn new() -> PoisonGirlB<Self,>
	{
		let cli = Cli::parse();
		Self::from_cli(cli,)
	}

	pub fn opts(&self,) -> &Opts
	{
		&self.opts
	}

	pub fn cmd(&self,) -> CliCommand
	{
		self.cmd
	}

	pub fn from_arch_build_mode(arch: Arch, build_mode: BuildMode,) -> Self
	{
		Self {
			opts: Opts { build_mode, arch, ..Default::default() },
			..Default::default()
		}
	}
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
		self.feature_flags.iter().map(|f| f.as_ref(),).collect()
	}
}

impl CompileOpt for Task
{
	fn build_mode(&self,) -> impl Into<String,>
	{
		self.opts.build_mode()
	}

	fn feature_flags(&self,) -> Vec<impl Into<String,>,>
	{
		self.opts.feature_flags()
	}
}

pub trait AsCargoOpt
{
	type Out;
	fn as_cargo_opt(&self,) -> Self::Out;
}

// impl AsCargoOpt for Opts
// {
// 	type Out = PoisonGirlB<Vec<String,>,>;

// 	fn as_cargo_opt(&self,) -> Self::Out
// 	{
// 		let Self { build_mode, feature_flags, lock_deps, .. } = self;
// 		let build_mode = build_mode.as_cargo_opt();
// 		let feature_flags: Vec<_,> =
// 			feature_flags.iter().map(|f| f.as_ref().to_string(),).collect();
// 		// single architecture info itself is useless
// 		// target tuple is truth
// 		// let arch = arch.as_cargo_opt();
// 		let lock_deps =
// 			if *lock_deps { Some("--locked".to_string(),) } else { None };

// 		X(build_mode
// 			.into_iter()
// 			.chain(feature_flags,)
// 			.chain(lock_deps,)
// 			.collect(),)
// 	}
// }

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

// impl AsCargoOpt for CliCommand
// {
// 	type Out = Option<String,>;

// 	fn as_cargo_opt(&self,) -> Self::Out
// 	{
// 		match self {
// 			Self::Check { .. } => None,
// 			Self::Fixture => None,
// 			_ => Some(self.as_ref().to_string(),),
// 		}
// 	}
// }

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_test::{PoisonGirlTestB, success},
	};

	#[test]
	fn test_cli_to_task_with_values() -> PoisonGirlTestB
	{
		let cli = Cli::default();

		let task = Task::from_cli(cli,)?;
		assert!(task.opts.build_mode.is_debug());
		assert!(task.opts.feature_flags.is_empty());
		assert!(task.opts.arch.is_aarch_64());
		assert!(!task.opts.lock_deps);
		success!()
	}
}
