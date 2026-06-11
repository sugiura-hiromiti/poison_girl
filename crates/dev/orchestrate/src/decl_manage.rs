use {
	crate::decl_manage::{
		crate_::{Crate, CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart},
		package::PackageSurvey,
	},
	poison_girl_dev_cargo::{CompileOpt, Opts, TargetSpec},
	poison_girl_dev_error::{PoisonGirlB, X, Y},
	poison_girl_dev_fs::{
		current_crate_path, project_root_path, search_in_with,
	},
	std::{path::PathBuf, str::FromStr},
};

pub mod crate_;
pub mod package;
pub mod workspace;

/// orchestration fuctionality which is finally resolved.
/// this is required over any traits in this crate because finally resolved
/// orchestration graph is not statically determined e.g. by env var, and this
/// trait also connect to existing cargo orchestration as needed
pub trait WorkspaceOrchestrate
{
	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_target_dir(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_target_triple_representation(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_profile(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_artifact_name(&self,) -> PoisonGirlB<PathBuf,>;

	fn as_crate(&self,) -> &impl Crate;
	fn as_opts(&self,) -> &impl CompileOpt;
}

pub struct PoisonGirlCargoInterface
{
	ws:   PoisonGirlCrate,
	opts: Opts,
}

impl PoisonGirlCargoInterface
{
	pub fn new(chart: PoisonGirlCrateChart, opts: Opts,) -> Self
	{
		Self { ws: PoisonGirlCrate::from(chart,), opts, }
	}
}

impl WorkspaceOrchestrate for PoisonGirlCargoInterface
{
	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,>
	{
		todo!(
			"1. --target-dir <path>
2. CARGO_TARGET_DIR=<path>
3. .cargo/config.toml: [build] target-dir = ..
4 default <workspace-root>/target

then target resolving
-> no --target leads target/debug|release
-> explicit leads target/<target tuple>/debug|release

even if user specifies host target, these use different directories

then resolve profile. dev|test is debug/, release|bench is release/, custom \
			 profile foo is foo/

artifact file name is crate name or [[bin.name]] in Cargo.toml"
		);
		let target_dir = self.resolve_target_dir()?;
		let target_tuple_representation =
			self.resolve_target_triple_representation()?;
		let profile = self.resolve_profile()?;
		let artifact_name = self.resolve_artifact_name()?;
		X(PathBuf::from_iter([
			target_dir,
			target_tuple_representation,
			profile,
			artifact_name,
		],),)
	}

	/// current(20260609) cargo's target directory determination follows these
	/// rules (numbers are priority)
	/// 1. --target-dir <path>
	/// 2. CARGO_TARGET_DIR=<path>
	/// 3. .cargo/config.toml: [build] target-dir = ..
	/// 4 default <workspace-root>/target
	///
	/// for rule 1, we ignore by filtering in xtask. this keeps things easy
	fn resolve_target_dir(&self,) -> PoisonGirlB<PathBuf,>
	{
		// 2 TODO: check does xtask managed process really affected parent env
		// var.
		if let Some(path,) = option_env!("CARGO_TARGET_DIR") {
			return X(PathBuf::from(path,),);
		}

		// 3
		// TODO: make sure that cargo_conf method do not fails when config.toml
		// not exists or empty
		let config_toml = self.ws.cargo_conf();
		match config_toml {
			Some(X(conf,),) => {
				if let Some(Some(toml::Value::String(target_dir,),),) = conf
					.get("build",)
					.map(|build_section| build_section.get("target-dir",),)
				{
					return X(PathBuf::from(target_dir,),);
				}
			},
			None => (),
			Some(Y(e,),) => return Y(e,),
		}

		// 4
		X(project_root()?.path().join("target",),)
	}

	/// if --target do not specified, profile name comes after target/
	/// if --target specified, target/<target tuple>/...
	/// even when user specifies host target, these use different directories.
	/// that means, if user specifies their host target explicitly by `--target
	/// <host tuple>`, then directory goes `target/<host's target tuple>/ ..`,
	/// not `target/ ...`
	///
	/// above is cargo's logic. here we can only specify cpu architecture for
	/// xtask
	fn resolve_target_triple_representation(&self,) -> PoisonGirlB<PathBuf,>
	{
		let arch = self.opts.arch;
		let chart = self.ws.as_chart();

		// TODO: extract kernel's vendor-os resolver logic. they should no be
		// tied to here
		match chart {
			PoisonGirlCrateChart::Kernel => "sugiura_hiromiti-poison_girl.json",
			PoisonGirlCrateChart::Loader => "unknown-uefi",
			_ => match std::env::consts::OS {
				"linux" => "unknown-linux",
				"macos" => "apple-darwin",
				other => {
					return Y(poison_girl_err!(YourHostPlatformIsOutOfSupport),);
				},
			},
		}
		[arch.to_string(),]
	}

	fn as_crate(&self,) -> &impl Crate
	{
		&self.ws
	}

	fn as_opts(&self,) -> &impl CompileOpt
	{
		&self.opts
	}
}

impl TargetSpec for PoisonGirlCargoInterface
{
	fn tuple(&self,) -> String
	{
		todo!()
	}

	fn arch(&self,) -> poison_girl_dev_cargo::Arch
	{
		todo!()
	}

	fn runtime(&self,) -> poison_girl_dev_cargo::Runtime
	{
		todo!()
	}
}

/// TODO: we can detect project_root while compile time. it's ideal to provide
/// root as PoisonGirlCrate const value
pub fn project_root() -> PoisonGirlB<PoisonGirlCrate,>
{
	let pr = project_root_path()?;
	X(PoisonGirlCrate::from(pr,),)
}

pub fn current_crate() -> PoisonGirlB<PoisonGirlCrate,>
{
	let ccp = current_crate_path()?;

	X(PoisonGirlCrate::from(ccp,),)
}
