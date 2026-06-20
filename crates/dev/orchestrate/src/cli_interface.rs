use {
	crate::decl_manage::crate_::PoisonGirlCrateChart,
	clap::{Parser, Subcommand},
	poison_girl_dev_cargo::{Arch, BuildMode},
	poison_girl_macro_def_features::features,
	std::path::PathBuf,
	strum::IntoDiscriminant,
};

#[derive(Default, Clone,)]
pub struct Policy
{
	pub global:  GlobalArg,
	pub command: CliCommand,
}

impl Policy
{
	pub fn new() -> Self
	{
		Self::from_cli(Cli::parse(),)
	}

	pub fn from_cli(cli: Cli,) -> Self
	{
		let Cli { global, command, } = cli;
		let command = command.unwrap_or_default();
		Self { global, command, }
	}

	pub fn from_arch_build_mode(arch: Arch, build_mode: BuildMode,) -> Self
	{
		Self {
			global: GlobalArg { arch, build_mode, ..Default::default() },
			..Default::default()
		}
	}

	pub fn with_command(mut self, command: CliCommand,) -> Self
	{
		self.command = command;
		self
	}

	pub fn command(&self,) -> &CliCommand
	{
		&self.command
	}

	pub fn command_discriminant(&self,) -> CliCommandDiscriminants
	{
		self.command.discriminant()
	}

	pub fn arch(&self,) -> Arch
	{
		self.global.arch
	}

	pub fn build_mode(&self,) -> BuildMode
	{
		self.global.build_mode
	}

	pub fn features(&self,) -> &[Feature]
	{
		&self.global.features
	}

	pub fn locked(&self,) -> bool
	{
		self.global.locked
	}
}

impl AsCargoOpt for Policy
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let args = self.global.as_cargo_opt();
		let mut args = CargoInvocationArgs::from_cargo_args(args,);
		let cmd_args = self.command.as_cargo_opt();

		args.extend(cmd_args,);
		args
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

impl CompileOpt for Policy
{
	fn build_mode(&self,) -> impl Into<String,>
	{
		self.global.build_mode.as_ref()
	}

	fn feature_flags(&self,) -> Vec<impl Into<String,>,>
	{
		self.global.features.iter().map(|f| f.as_ref(),).collect()
	}
}

pub trait AsCargoOpt
{
	type Out;

	fn as_cargo_opt(&self,) -> Self::Out;
}

impl AsCargoOpt for BuildMode
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		match self {
			Self::Release => vec!["--release".to_string()],
			Self::Debug => vec![],
		}
	}
}

#[derive(Parser, Default, Clone,)]
// #[command(version, about)]
pub struct Cli
{
	#[command(flatten)]
	global:  GlobalArg,
	#[command(subcommand)]
	command: Option<CliCommand,>,
}

#[derive(clap::Args, Default, Clone,)]
pub struct GlobalArg
{
	#[arg(short, long, value_enum, default_value_t)]
	arch:       Arch,
	#[arg(long, default_value_t)]
	locked:     bool,
	#[arg(short = 'm', long, value_enum, default_value_t)]
	build_mode: BuildMode,
	#[arg(short = 'F', long)]
	features:   Vec<Feature,>,
}

impl AsCargoOpt for GlobalArg
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let Self { locked, build_mode, features, .. } = self;

		let locked = if *locked { Some("--locked".to_string(),) } else { None };
		let build_mode = build_mode.as_cargo_opt();
		let features = features.as_cargo_opt();

		features.into_iter().chain(build_mode,).chain(locked,).collect()
	}
}

#[derive(Debug, Default, Eq, PartialEq,)]
pub struct CargoInvocationArgs
{
	cargo_args: Vec<String,>,
	tool_args:  Vec<String,>,
}

impl CargoInvocationArgs
{
	pub fn from_cargo_args(cargo_args: Vec<String,>,) -> Self
	{
		Self { cargo_args, tool_args: vec![], }
	}

	pub fn from_tool_args(tool_args: Vec<String,>,) -> Self
	{
		Self { cargo_args: vec![], tool_args, }
	}

	pub fn extend(&mut self, Self { cargo_args, tool_args, }: Self,)
	{
		self.cargo_args.extend(cargo_args,);
		self.tool_args.extend(tool_args,);
	}

	pub fn into_cargo_args(self,) -> Vec<String,>
	{
		let Self { mut cargo_args, tool_args, } = self;

		if !tool_args.is_empty() {
			cargo_args.push("--".to_owned(),);
			cargo_args.extend(tool_args,);
		}

		cargo_args
	}
}

impl AsCargoOpt for Vec<CargoInvocationArgs,>
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let mut args = CargoInvocationArgs::default();
		for arg_set in self {
			args.extend(CargoInvocationArgs {
				cargo_args: arg_set.cargo_args.clone(),
				tool_args:  arg_set.tool_args.clone(),
			},);
		}

		args.into_cargo_args()
	}
}

#[derive(
	Subcommand,
	strum_macros::EnumIs,
	strum_macros::AsRefStr,
	strum_macros::EnumDiscriminants,
	Clone,
)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(derive(Hash, strum_macros::AsRefStr))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum CliCommand
{
	Build(BuildArgs,),
	Test(TestArgs,),
	Run(RunArgs,),
	Clippy(ClippyArgs,),
	Fixture(FixtureArgs,),
	/// cargo fix
	Fix(FixArgs,),
}

impl AsCargoOpt for CliCommand
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		match self {
			Self::Build(args,) => args.as_cargo_opt(),
			Self::Test(args,) => args.as_cargo_opt(),
			Self::Run(args,) => args.as_cargo_opt(),
			Self::Clippy(args,) => args.as_cargo_opt(),
			Self::Fixture(args,) => args.as_cargo_opt(),
			Self::Fix(args,) => args.as_cargo_opt(),
		}
	}
}

impl CliCommandDiscriminants
{
	pub fn default_command(self,) -> CliCommand
	{
		match self {
			Self::Build => CliCommand::Build(Default::default(),),
			Self::Test => CliCommand::Test(Default::default(),),
			Self::Run => CliCommand::Run(Default::default(),),
			Self::Clippy => CliCommand::Clippy(Default::default(),),
			Self::Fixture => CliCommand::Fixture(Default::default(),),
			Self::Fix => CliCommand::Fix(Default::default(),),
		}
	}
}

impl Default for CliCommand
{
	fn default() -> Self
	{
		Self::Test(Default::default(),)
	}
}

#[derive(clap::Args, Default, Clone,)]
pub struct BuildArgs {}

impl AsCargoOpt for BuildArgs
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		CargoInvocationArgs { cargo_args: vec![], tool_args: vec![], }
	}
}

#[derive(clap::Args, Default, Clone,)]
pub struct TestArgs {}

impl AsCargoOpt for TestArgs
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		CargoInvocationArgs { cargo_args: vec![], tool_args: vec![], }
	}
}

#[derive(clap::Args, Default, Clone,)]
pub struct RunArgs {}

impl AsCargoOpt for RunArgs
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		CargoInvocationArgs { cargo_args: vec![], tool_args: vec![], }
	}
}

#[derive(clap::Args, Default, Clone,)]
pub struct ClippyArgs
{
	#[arg(long)]
	deny_warnings: bool,
}

impl AsCargoOpt for ClippyArgs
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let tool_args =
			if self.deny_warnings { vec!["-D", "warnings"] } else { vec![] }
				.into_iter()
				.map(|s| s.to_string(),)
				.collect();
		CargoInvocationArgs::from_tool_args(tool_args,)
	}
}

#[derive(clap::Args, Default, Clone,)]
pub struct FixtureArgs {}

impl AsCargoOpt for FixtureArgs
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		CargoInvocationArgs { cargo_args: vec![], tool_args: vec![], }
	}
}

#[derive(clap::Args, Default, Clone,)]
pub struct FixArgs
{
	#[arg(long)]
	allow_dirty:  bool,
	#[arg(long)]
	allow_staged: bool,
}

impl FixArgs
{
	pub fn allow_dirty_and_staged() -> Self
	{
		Self { allow_dirty: true, allow_staged: true, }
	}
}

impl AsCargoOpt for FixArgs
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let allow_dirty =
			if self.allow_dirty { vec!["--allow-dirty"] } else { vec![] };
		let allow_staged =
			if self.allow_staged { vec!["--allow-staged"] } else { vec![] };

		let cargo_args = allow_dirty
			.into_iter()
			.chain(allow_staged,)
			.map(|s| s.to_string(),)
			.collect();

		CargoInvocationArgs { cargo_args, tool_args: vec![], }
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_test::{PoisonGirlTestB, success},
	};

	#[test]
	fn test_cli_to_policy_with_defaults() -> PoisonGirlTestB
	{
		let cli = Cli::default();

		let policy = Policy::from_cli(cli,);
		assert!(policy.build_mode().is_debug());
		assert!(policy.features().is_empty());
		assert!(policy.arch().is_aarch_64());
		assert!(!policy.locked());
		assert_eq!(
			policy.command_discriminant(),
			CliCommandDiscriminants::Test
		);
		success!()
	}

	#[test]
	fn policy_with_command_replaces_command() -> PoisonGirlTestB
	{
		let policy = Policy::default()
			.with_command(CliCommand::Fix(FixArgs::allow_dirty_and_staged(),),);

		assert_eq!(policy.command_discriminant(), CliCommandDiscriminants::Fix);
		assert!(policy.arch().is_aarch_64());
		assert!(policy.build_mode().is_debug());
		success!()
	}

	#[test]
	fn policy_as_cargo_opt_does_not_emit_subcommand() -> PoisonGirlTestB
	{
		let policy = Policy {
			global:  GlobalArg {
				build_mode: BuildMode::Release,
				locked: true,
				..Default::default()
			},
			command: CliCommand::Build(Default::default(),),
		};

		let opt = policy.as_cargo_opt();

		assert_eq!(
			opt.into_cargo_args(),
			vec!["--release".to_string(), "--locked".to_string()]
		);
		success!()
	}

	#[test]
	fn policy_as_cargo_opt_places_tool_args_after_separator() -> PoisonGirlTestB
	{
		let policy = Policy {
			global:  GlobalArg {
				build_mode: BuildMode::Release,
				..Default::default()
			},
			command: CliCommand::Clippy(ClippyArgs { deny_warnings: true, },),
		};

		let opt = policy.as_cargo_opt();

		assert_eq!(
			opt.into_cargo_args(),
			vec![
				"--release".to_string(),
				"--".to_string(),
				"-D".to_string(),
				"warnings".to_string()
			]
		);
		success!()
	}
}
