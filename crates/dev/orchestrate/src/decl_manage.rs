use {
	crate::decl_manage::crate_::{
		Crate, CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart,
	},
	poison_girl_dev_cargo::{CompileOpt, Opts, TargetSpec},
	poison_girl_dev_error::{
		PoisonGirlB, X, Y, YourHostPlatformIsOutOfSupport, poison_girl_err,
	},
	poison_girl_dev_fs::{current_crate_path, project_root_path},
	std::path::PathBuf,
};

pub mod crate_;
pub mod package;
pub mod workspace;

pub struct BuildArtifact
{
	target_dir:                  PathBuf,
	/// maybe should be Option
	target_tuple_representation: PathBuf,
	profile:                     PathBuf,
	artifact_name:               PathBuf,
}

impl BuildArtifact
{
	pub fn path(&self,) -> PathBuf
	{
		let Self {
			target_dir,
			target_tuple_representation,
			profile,
			artifact_name,
		} = self;

		let workspace_root = PoisonGirlCrateChart::XTASK.to_path_buf();

		// TODO: support `target_dir` format as relative path and absolute path.
		workspace_root
			.join(target_dir,)
			.join(target_tuple_representation,)
			.join(profile,)
			.join(artifact_name,)
	}
}

/// orchestration functionality which is finally resolved.
/// this is required over any traits in this crate because finally resolved
/// orchestration graph is not statically determined e.g. by env var, and this
/// trait also connect to existing cargo orchestration as needed
pub trait OrchestrationResolver
{
	fn build_artifact(&self,) -> PoisonGirlB<BuildArtifact,>;
	fn resolve_target_dir(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_target_triple_representation(&self,) -> PoisonGirlB<PathBuf,>;
	fn resolve_profile(&self,) -> PathBuf;
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

	pub fn opts(&self,) -> Opts
	{
		self.opts.clone()
	}

	pub fn ws(&self,) -> PoisonGirlCrate
	{
		self.ws.clone()
	}
}

impl OrchestrationResolver for PoisonGirlCargoInterface
{
	/// TODO: I want to refactor centerizing orchestration info to upper level
	/// xtask struct. to do that, this function should return the list of
	/// information which is used on building build artifact path instead of one
	/// PathBuf
	fn build_artifact(&self,) -> PoisonGirlB<BuildArtifact,>
	{
		let target_dir = self.resolve_target_dir()?;
		let target_tuple_representation =
			self.resolve_target_triple_representation()?;
		let profile = self.resolve_profile();
		let artifact_name = self.resolve_artifact_name()?;

		X(BuildArtifact {
			target_dir,
			target_tuple_representation,
			profile,
			artifact_name,
		},)
	}

	/// current(20260609) cargo's target directory determination follows these
	/// rules (numbers are priority)
	/// 1. --target-dir <path>
	/// 2. CARGO_TARGET_DIR=<path>
	/// 3. .cargo/config.toml: [build] target-dir = ..
	/// 4. default <workspace-root>/target
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
		let vendor_runtime = match chart {
			PoisonGirlCrateChart::Kernel => {
				"sugiura_hiromiti-poison_girl-elf.json"
			},
			PoisonGirlCrateChart::Loader => "unknown-uefi",
			_ => match std::env::consts::OS {
				"linux" => "unknown-linux",
				"macos" => "apple-darwin",
				_ => {
					return Y(poison_girl_err!(YourHostPlatformIsOutOfSupport),);
				},
			},
		};
		X(PathBuf::from([arch.as_ref(), vendor_runtime,].join("-",),),)
	}

	///then resolve profile. dev|test is debug/, release|bench is release/,
	/// custom profile foo is foo/
	///
	/// now, we only support debug/ and release/
	fn resolve_profile(&self,) -> PathBuf
	{
		PathBuf::from(self.opts.build_mode.as_ref(),)
	}

	/// artifact file name is crate name or [[bin.name]] in Cargo.toml"
	fn resolve_artifact_name(&self,) -> PoisonGirlB<PathBuf,>
	{
		let bin_name = self.ws.as_chart().bin_name();
		X(PathBuf::from(bin_name,),)
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
		let arch = self.arch();
		let arch = arch.as_ref();

		use poison_girl_dev_cargo::Runtime::*;
		match self.runtime() {
			Host => "host-tuple".to_string(),
			Efi => [arch, "unknown-uefi",].join("-",),
			PoisonGirl => {
				[arch, "sugiura_hiromiti-poison_girl.json",].join("-",)
			},
		}
	}

	fn arch(&self,) -> poison_girl_dev_cargo::Arch
	{
		self.opts.arch
	}

	fn runtime(&self,) -> poison_girl_dev_cargo::Runtime
	{
		let chart = self.ws.as_chart();
		if chart == &PoisonGirlCrateChart::KERNEL {
			poison_girl_dev_cargo::Runtime::PoisonGirl
		} else if chart == &PoisonGirlCrateChart::LOADER {
			poison_girl_dev_cargo::Runtime::Efi
		} else {
			poison_girl_dev_cargo::Runtime::Host
		}
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
