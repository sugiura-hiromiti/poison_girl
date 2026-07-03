use {
	crate::{
		AsCargoOpt, CliCommandDiscriminants, CompileOpt, Policy,
		decl_manage::crate_::{
			Crate, CrateAction, CrateInfo, PoisonGirlCrate,
			PoisonGirlCrateChart,
		},
		policy::{
			build_artifact_policy::{
				BuildArtifact, BuildArtifactPolicyResolver,
			},
			build_std_features_policy::{
				BuildStdFeaturesPolicies, BuildStdFeaturesPolicy,
				BuildStdFeaturesPolicyResolver,
			},
			build_std_policy::{
				BuildStdPolicies, BuildStdPolicy, BuildStdPoliyResolver,
			},
			target_policy::{TargetPolicy, TargetPolicyResolver},
		},
	},
	poison_girl_dev_cargo::Runtime,
	poison_girl_dev_error::{
		InvalidMetadataSchema, PoisonGirlB, X, Y,
		YourHostPlatformIsOutOfSupport, poison_girl_err,
	},
	poison_girl_dev_fs::{current_crate_path, project_root_path},
	std::{
		ffi::OsStr,
		path::{Path, PathBuf},
	},
};

pub mod crate_;
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
		self.ws()
			.cargo_xxx_with(self.policy().command_discriminant(), self.policy(),)
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

		X(BuildArtifact::new(
			target_dir,
			target_tuple_representation,
			profile,
			artifact_name,
		),)
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
		let cwd = std::env::current_dir()?;
		if let Some(path,) = env_var_path("CARGO_TARGET_DIR", cwd.as_path(),)? {
			return X(path,);
		}
		if let Some(path,) =
			env_var_path("CARGO_BUILD_TARGET_DIR", cwd.as_path(),)?
		{
			return X(path,);
		}

		// 3
		// TODO: make sure that cargo_conf method do not fails when config.toml
		// not exists or empty
		let config_toml = self.ws.cargo_conf();
		match config_toml {
			X(conf,) => {
				if let Some(Some(toml::Value::String(target_dir,),),) = conf
					.get("build",)
					.map(|build_section| build_section.get("target-dir",),)
				{
					let target_dir = PathBuf::from(target_dir,);
					let target_dir = if target_dir.is_relative() {
						let crate_path = self.ws.path();
						crate_path.join(target_dir,)
					} else {
						target_dir
					};

					return X(target_dir,);
				}
			},
			Y(e,) => return Y(e,),
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
		let arch = self.policy.arch();
		let chart = self.ws.as_chart();

		// TODO: extract kernel's vendor-os resolver logic. they should no be
		// tied to here
		let vendor_runtime = match chart {
			PoisonGirlCrateChart::Kernel => "sugiura_hiromiti-poison_girl-elf",
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

impl BuildStdPoliyResolver for PoisonGirlCargoInterface
{
	fn build_std_policies(&self,) -> BuildStdPolicies
	{
		let command = self.policy.command_discriminant();
		let chart = self.ws.as_chart();
		let uses_custom_target = command == CliCommandDiscriminants::Build
			|| command == CliCommandDiscriminants::Fix
			|| (command == CliCommandDiscriminants::Clippy
				&& !self.policy.clippy_uses_host_target());

		// TODO: avoid hardcoding crate chart. Instead, consider alternative
		// solutions below.
		// 1. add methods to cratechart which returns it is no_std feature/it
		//    requires unstable options
		// 2. move responsibility of building `build_std_policy` to policy
		//    definition
		let policies = if uses_custom_target {
			if *chart == PoisonGirlCrateChart::KERNEL {
				vec![BuildStdPolicy::Core]
			} else if *chart == PoisonGirlCrateChart::LOADER {
				vec![
					BuildStdPolicy::Core,
					BuildStdPolicy::Alloc,
					BuildStdPolicy::CompilerBuiltins,
				]
			} else {
				vec![]
			}
		} else {
			vec![]
		};
		BuildStdPolicies::from(policies,)
	}
}

impl BuildStdFeaturesPolicyResolver for PoisonGirlCargoInterface
{
	fn build_std_features_policies(&self,) -> BuildStdFeaturesPolicies
	{
		let command = self.policy.command_discriminant();
		let chart = self.ws.as_chart();
		let uses_custom_target = command == CliCommandDiscriminants::Build
			|| command == CliCommandDiscriminants::Fix
			|| (command == CliCommandDiscriminants::Clippy
				&& !self.policy.clippy_uses_host_target());
		let policies = if uses_custom_target {
			if *chart == PoisonGirlCrateChart::KERNEL
				|| *chart == PoisonGirlCrateChart::LOADER
			{
				vec![BuildStdFeaturesPolicy::CompilerBuiltinsMem]
			} else {
				vec![]
			}
		} else {
			vec![]
		};
		BuildStdFeaturesPolicies::from(policies,)
	}
}

impl TargetPolicyResolver for PoisonGirlCargoInterface
{
	fn target_policy(&self,) -> TargetPolicy
	{
		let arch = self.policy.arch();

		if self.policy.command_discriminant() == CliCommandDiscriminants::Test
			|| self.policy.clippy_uses_host_target()
		{
			return TargetPolicy::new(arch, Runtime::Host,);
		}

		let runtime = match *self.ws.as_chart() {
			PoisonGirlCrateChart::KERNEL => Runtime::PoisonGirl,
			PoisonGirlCrateChart::LOADER => Runtime::Efi,
			_ => Runtime::Host,
		};

		TargetPolicy::new(arch, runtime,)
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
		let policy =
			self.policy.clone().with_features_supported_by(self.ws.as_chart(),);
		let straight_cmd = policy.as_cargo_opt();
		let command = policy.command_discriminant();
		let target = self.target_policy().as_cargo_opt();
		let build_std = self.build_std_policies().as_cargo_opt();
		let build_std_features =
			self.build_std_features_policies().as_cargo_opt();

		let PoisonGirlPackageMetadata { no_std, } =
			self.ws.custom_metadata()?;
		let mut additional_opts: Vec<_,> =
			vec!["-p", self.ws().as_chart().package_name()]
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

/// if environment variable points to a relative path, this function resolve
/// absolute path based on 2nd argument
fn env_var_path(
	path: impl AsRef<OsStr,>,
	base_path: impl AsRef<Path,>,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	if let Ok(path,) = std::env::var(path,) {
		// CARGO_TARGET_DIR specifies relative path to current directory
		let path = PathBuf::from(path,);
		if path.is_relative() {
			PoisonGirlB::X(Some(base_path.as_ref().to_path_buf().join(path,),),)
		} else {
			X(Some(path,),)
		}
	} else {
		X(None,)
	}
}
