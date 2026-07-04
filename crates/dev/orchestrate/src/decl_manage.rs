use {
	crate::{
		AsCargoOpt, CliCommandDiscriminants, CompileOpt, Policy,
		decl_manage::crate_::{
			Crate, CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart,
		},
		policy::build_artifact_policy::{
			BuildArtifact, BuildArtifactPolicyResolver,
		},
	},
	poison_girl_dev_cli::Run,
	poison_girl_dev_error::{
		InvalidMetadataSchema, PoisonGirlB, X, Y, poison_girl_err,
	},
	poison_girl_dev_fs::{current_crate_path, project_root_path},
	std::{
		path::{Path, PathBuf},
		process::Command,
	},
};

use self::invocation_plan::CargoInvocationPlan;

pub mod crate_;
mod invocation_plan;
pub mod package;
pub mod workspace;

pub struct PoisonGirlCargoInterface
{
	ws:     PoisonGirlCrate,
	policy: Policy,
}

impl PoisonGirlCargoInterface
{
	pub fn new(chart: PoisonGirlCrateChart, policy: Policy,) -> Self
	{
		Self { ws: PoisonGirlCrate::from(chart,), policy, }
	}

	pub fn policy(&self,) -> &Policy
	{
		&self.policy
	}

	pub fn ws(&self,) -> PoisonGirlCrate
	{
		self.ws.clone()
	}

	pub fn run(&self,) -> PoisonGirlB<(),>
	{
		let command = self.policy().command_discriminant();
		for args in self.invocation_args()? {
			let mut cargo = Command::new("cargo",);
			let cargo = cargo.arg(command.as_ref(),);
			if !args.is_empty() {
				cargo.args(args,);
			}
			cargo.run()?;
		}

		X((),)
	}

	fn invocation_plan(&self,) -> CargoInvocationPlan
	{
		CargoInvocationPlan::new(*self.ws.as_chart(), self.policy.clone(),)
	}

	fn invocation_args(&self,) -> PoisonGirlB<Vec<Vec<String,>,>,>
	{
		let mut args = vec![];
		for policy in self.invocation_plan().invocation_policies()? {
			args.push(Self::new(*self.ws.as_chart(), policy,).as_cargo_opt()?,);
		}
		X(args,)
	}
}

impl BuildArtifactPolicyResolver for PoisonGirlCargoInterface
{
	fn build_artifact_policy(&self,) -> PoisonGirlB<BuildArtifact,>
	{
		let target_dir = self.resolve_target_dir()?;
		let target_tuple_representation =
			self.resolve_target_triple_representation()?;
		let profile = self.resolve_profile();
		let artifact_name = self.resolve_artifact_name()?;
		let artifact_path = target_dir
			.join(target_tuple_representation,)
			.join(profile,)
			.join(artifact_name,);

		X(BuildArtifact::new(artifact_path,),)
	}

	/// current(20260609) cargo's target directory determination follows these
	/// rules (numbers are priority)
	/// 1. --target-dir <path>
	/// 2. CARGO_TARGET_DIR=<path>
	/// 3. CARGO_BUILD_TARGET_DIR=<path>
	/// 4. .cargo/config.toml: [build] target-dir = ..
	/// 5. default <workspace-root>/target
	///
	/// for rule 1, we ignore by filtering in xtask. this keeps things easy
	fn resolve_target_dir(&self,) -> PoisonGirlB<PathBuf,>
	{
		let cwd = std::env::current_dir()?;
		if let Some(path,) =
			std::env::var_os("CARGO_TARGET_DIR",).map(PathBuf::from,)
		{
			return X(resolve_relative_path(path, cwd.as_path(),),);
		}
		if let Some(path,) =
			std::env::var_os("CARGO_BUILD_TARGET_DIR",).map(PathBuf::from,)
		{
			return X(resolve_relative_path(path, cwd.as_path(),),);
		}

		let conf = self.ws.cargo_conf()?;
		let config_target_dir = conf
			.get("build",)
			.and_then(|build_section| build_section.get("target-dir",),)
			.and_then(|target_dir| target_dir.as_str(),);
		if let Some(path,) = config_target_dir {
			return X(resolve_relative_path(
				PathBuf::from(path,),
				self.ws.path().as_path(),
			),);
		}

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
		X(self.invocation_plan().build_target_tuple_representation(),)
	}

	///then resolve profile. dev|test is debug/, release|bench is release/,
	/// custom profile foo is foo/
	///
	/// now, we only support debug/ and release/
	fn resolve_profile(&self,) -> PathBuf
	{
		PathBuf::from(self.policy.build_mode().as_ref(),)
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
		&self.policy
	}
}

impl AsCargoOpt for PoisonGirlCargoInterface
{
	type Out = PoisonGirlB<Vec<String,>,>;

	/// TODO: this code have to be refactored for better data structure,
	/// decralative architecture. in terms of reusability, extensibility, dev
	/// ergo and scalability, this code have to be refactored
	fn as_cargo_opt(&self,) -> Self::Out
	{
		let plan = self.invocation_plan().with_supported_features();
		let straight_cmd = plan.policy().as_cargo_opt();
		let command = plan.command();
		let target = plan.target_policy().as_cargo_opt();
		let build_std = plan.build_std_policies().as_cargo_opt();
		let build_std_features =
			plan.build_std_features_policies().as_cargo_opt();

		let PoisonGirlPackageMetadata { no_std, } =
			self.ws.custom_metadata()?;
		let mut additional_opts: Vec<_,> =
			vec!["-p", plan.chart().package_name()]
				.into_iter()
				.map(|s| s.to_owned(),)
				.collect();

		if no_std && command == CliCommandDiscriminants::Test {
			additional_opts.push("--lib".to_string(),);
		}

		let resolved_args =
			vec![straight_cmd, target, build_std, build_std_features]
				.as_cargo_opt();

		let opts = additional_opts
			.into_iter()
			.map(|s| s.to_owned(),)
			.chain(resolved_args,)
			.collect();

		X(opts,)
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_cargo::{Arch, BuildMode},
		poison_girl_dev_test::{PoisonGirlTestB, success},
	};

	#[test]
	fn clippy_custom_target_lib_for_loader_uses_uefi_build_std()
	-> PoisonGirlTestB
	{
		let policy = Policy::from_cmd(CliCommandDiscriminants::Clippy,)
			.with_clippy_custom_target_lib()?;
		let interface = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::LOADER,
			policy,
		);

		let args = interface.as_cargo_opt()?;

		assert!(args.iter().any(|arg| arg == "--lib",));
		assert!(!args.iter().any(|arg| arg == "--all-targets",));
		assert!(args.iter().any(|arg| arg == "--target",));
		assert!(args.iter().any(|arg| arg == "aarch64-unknown-uefi",));
		assert!(args.iter().any(|arg| {
			arg.starts_with("build-std=",)
				&& arg.contains("core",)
				&& arg.contains("alloc",)
				&& arg.contains("compiler_builtins",)
		},));
		assert!(
			args.iter().any(|arg| {
				arg == "build-std-features=compiler-builtins-mem"
			},)
		);
		success!()
	}

	#[test]
	fn clippy_default_for_loader_splits_custom_lib_and_host_tests()
	-> PoisonGirlTestB
	{
		let interface = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::LOADER,
			Policy::from_cmd(CliCommandDiscriminants::Clippy,),
		);
		let invocations = interface.invocation_args()?;

		assert_eq!(invocations.len(), 2);

		let first_args = &invocations[0];
		let second_args = &invocations[1];

		assert!(first_args.iter().any(|arg| arg == "--lib",));
		assert!(first_args.iter().any(|arg| arg == "--target",));
		assert!(first_args.iter().any(|arg| arg == "aarch64-unknown-uefi",));
		assert!(first_args.iter().any(|arg| arg.starts_with("build-std=",),));

		assert!(second_args.iter().any(|arg| arg == "--tests",));
		assert!(!second_args.iter().any(|arg| arg == "--target",));
		assert!(!second_args.iter().any(|arg| arg.starts_with("build-std=",),));
		success!()
	}

	#[test]
	fn clippy_host_tests_for_loader_do_not_use_custom_target() -> PoisonGirlTestB
	{
		let policy = Policy::from_cmd(CliCommandDiscriminants::Clippy,)
			.with_clippy_host_tests()?;
		let interface = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::LOADER,
			policy,
		);

		let args = interface.as_cargo_opt()?;

		assert!(args.iter().any(|arg| arg == "--tests",));
		assert!(!args.iter().any(|arg| arg == "--all-targets",));
		assert!(!args.iter().any(|arg| arg == "--target",));
		assert!(!args.iter().any(|arg| arg.starts_with("build-std=",),));
		assert!(
			!args.iter().any(|arg| arg.starts_with("build-std-features=",),)
		);
		success!()
	}

	#[test]
	fn host_test_uses_cargo_default_target_layout() -> PoisonGirlTestB
	{
		let policy = Policy::from_cmd(CliCommandDiscriminants::Test,);
		let interface = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::DevOrchestrate,
			policy,
		);

		let args = interface.as_cargo_opt()?;

		assert_eq!(
			interface.resolve_target_triple_representation()?,
			PathBuf::new()
		);
		assert!(!args.iter().any(|arg| arg == "--target",));
		assert!(!args.iter().any(|arg| arg.starts_with("build-std=",),));
		success!()
	}

	#[test]
	fn kernel_artifact_uses_build_target_when_policy_command_defaults_to_test()
	-> PoisonGirlTestB
	{
		let interface = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::Kernel,
			Policy::from_arch_build_mode(Arch::Aarch64, BuildMode::Debug,),
		);

		assert_eq!(
			interface.resolve_target_triple_representation()?,
			PathBuf::from("aarch64-sugiura_hiromiti-poison_girl-elf",)
		);
		success!()
	}

	#[test]
	fn resolve_target_dir_prefers_cargo_target_dir_env() -> PoisonGirlTestB
	{
		let cwd = Path::new("/workspace/current-crate",);
		let crate_path = Path::new("/workspace/loader",);

		let target_dir = resolve_target_dir_from_inputs(
			cwd,
			Some(PathBuf::from("target-from-cargo-target-dir",),),
			Some(PathBuf::from("target-from-build-target-dir",),),
			Some("target-from-config",),
			crate_path,
			PathBuf::from("/workspace/target",),
		);

		assert_eq!(target_dir, cwd.join("target-from-cargo-target-dir",));
		success!()
	}

	#[test]
	fn resolve_target_dir_uses_build_env_before_config() -> PoisonGirlTestB
	{
		let cwd = Path::new("/workspace/current-crate",);
		let crate_path = Path::new("/workspace/loader",);

		let target_dir = resolve_target_dir_from_inputs(
			cwd,
			None,
			Some(PathBuf::from("target-from-build-target-dir",),),
			Some("target-from-config",),
			crate_path,
			PathBuf::from("/workspace/target",),
		);

		assert_eq!(target_dir, cwd.join("target-from-build-target-dir",));
		success!()
	}

	#[test]
	fn resolve_target_dir_uses_config_relative_to_crate_path() -> PoisonGirlTestB
	{
		let cwd = Path::new("/workspace/current-crate",);
		let crate_path = Path::new("/workspace/loader",);

		let target_dir = resolve_target_dir_from_inputs(
			cwd,
			None,
			None,
			Some("target-from-config",),
			crate_path,
			PathBuf::from("/workspace/target",),
		);

		assert_eq!(target_dir, crate_path.join("target-from-config",));
		success!()
	}

	#[test]
	fn resolve_target_dir_defaults_to_workspace_target() -> PoisonGirlTestB
	{
		let default_target = PathBuf::from("/workspace/target",);

		let target_dir = resolve_target_dir_from_inputs(
			Path::new("/workspace/current-crate",),
			None,
			None,
			None,
			Path::new("/workspace/loader",),
			default_target.clone(),
		);

		assert_eq!(target_dir, default_target);
		success!()
	}

	#[test]
	fn package_metadata_defaults_to_std_when_no_std_is_absent()
	-> PoisonGirlTestB
	{
		let metadata =
			PoisonGirlPackageMetadata::from_toml_table(&toml::Table::new(),)?;

		assert!(!metadata.no_std);
		success!()
	}

	#[test]
	fn package_metadata_accepts_boolean_no_std() -> PoisonGirlTestB
	{
		let mut table = toml::Table::new();
		table.insert("no_std".to_owned(), toml::Value::Boolean(true,),);

		let metadata = PoisonGirlPackageMetadata::from_toml_table(&table,)?;

		assert!(metadata.no_std);
		success!()
	}

	#[test]
	fn package_metadata_rejects_non_boolean_no_std()
	{
		let mut table = toml::Table::new();
		table.insert("no_std".to_owned(), toml::Value::String("yes".into(),),);

		assert!(matches!(
			PoisonGirlPackageMetadata::from_toml_table(&table,),
			Y(_)
		));
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

/// this defines scheme of custom package metadata written in Cargo.toml
#[derive(Default,)]
pub struct PoisonGirlPackageMetadata
{
	no_std: bool,
}

impl PoisonGirlPackageMetadata
{
	pub const METADATA_PATH: [&str; 3] =
		["package", "metadata", "poison_girl",];

	pub fn from_toml_table(value: &toml::Table,) -> PoisonGirlB<Self,>
	{
		let no_std = match value.get("no_std",) {
			Some(toml::Value::Boolean(no_std,),) => *no_std,
			Some(_,) => return Y(poison_girl_err!(InvalidMetadataSchema),),
			None => Default::default(),
		};

		X(Self { no_std, },)
	}
}

#[cfg(test)]
fn resolve_target_dir_from_inputs(
	cwd: &Path,
	cargo_target_dir: Option<PathBuf,>,
	cargo_build_target_dir: Option<PathBuf,>,
	cargo_config_target_dir: Option<&str,>,
	cargo_config_base_path: &Path,
	default_target_dir: PathBuf,
) -> PathBuf
{
	if let Some(path,) = cargo_target_dir {
		return resolve_relative_path(path, cwd,);
	}
	if let Some(path,) = cargo_build_target_dir {
		return resolve_relative_path(path, cwd,);
	}
	if let Some(path,) = cargo_config_target_dir {
		return resolve_relative_path(
			PathBuf::from(path,),
			cargo_config_base_path,
		);
	}

	default_target_dir
}

fn resolve_relative_path(path: PathBuf, base_path: &Path,) -> PathBuf
{
	if path.is_relative() { base_path.join(path,) } else { path }
}
