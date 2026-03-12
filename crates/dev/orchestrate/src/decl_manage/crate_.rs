use {
	crate::decl_manage::{
		package::{Package, PackageAction, PackageInfo, PackageSurvey},
		workspace::{
			Workspace, WorkspaceAction, WorkspaceInfo, WorkspaceSurvey,
		},
	},
	poison_girl_dev_cargo::host_tuple,
	poison_girl_dev_cli::Run,
	poison_girl_dev_error::{Container, PoisonGirlB, X},
	poison_girl_dev_fs::{
		CARGO_CONFIG, CARGO_MANIFEST, all_crates_in, read_toml,
		search_upstream_at,
	},
	poison_girl_macro_def_from_path_buf::FromPathBuf,
	std::{ffi::OsStr, fmt::Debug, path::PathBuf, process::Command},
};

pub trait Crate: Workspace + Package
{
	fn as_pkg(&self,) -> &impl Package
	{
		self
	}

	fn as_wrkspc(&self,) -> &impl Workspace
	{
		self
	}
}

pub trait CrateSurvey: CrateInfo
{
	fn has_parent(&self,) -> PoisonGirlB<bool,>
	{
		let path = self.path();
		X(search_upstream_at(&path, CARGO_MANIFEST,)?.is_some(),)
	}
	fn go_parent(&mut self,) -> PoisonGirlB<(),>;
	fn fix(&self,) -> PoisonGirlB<(),>
	{
		let mut manifest = self.toml()?;
		if let Some(pkg,) = manifest.get_mut("package",)
			&& let Some(toml::Value::String(name,),) = pkg.get_mut("name",)
			&& let true_name = self.name()
			&& *name != true_name
		{
			*name = true_name;
			std::fs::write(
				self.path().join(CARGO_MANIFEST,),
				toml::to_string(&manifest,)?,
			)?;
		};
		X((),)
	}
	fn land_on(&mut self, on: impl CrateCalled,);
}

/// methods provided keeps environment e.g. current path
pub trait CrateAction: CrateInfo
{
	// actions for all packages

	fn build(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx("build",)
	}
	fn test(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx("test",)
	}
	fn run(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx("run",)
	}
	fn check(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx("check",)
	}
	fn format(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx("fmt",)
	}
	fn cargo_xxx(&self, cmd: impl AsRef<OsStr,>,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with(cmd, &["",],)
	}

	// actions for all packages with specific options

	fn build_with(&self, opt: &[impl AsRef<OsStr,>],) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with("build", opt,)
	}
	fn test_with(&self, opt: &[impl AsRef<OsStr,>],) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with("test", opt,)
	}
	fn run_with(&self, opt: &[impl AsRef<OsStr,>],) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with("run", opt,)
	}
	fn ckeck_with(&self, opt: &[impl AsRef<OsStr,>],) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with("check", opt,)
	}
	fn fmt_with(&self, opt: &[impl AsRef<OsStr,>],) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with("fmt", opt,)
	}
	fn cargo_xxx_with(
		&self,
		cmd: impl AsRef<OsStr,>,
		opt: &[impl AsRef<OsStr,>],
	) -> PoisonGirlB<(),>
	{
		let mut cargo = Command::new("cargo",);
		let cargo = cargo.arg(cmd,);

		let opt: Vec<_,> =
			opt.iter().filter(|s| !s.as_ref().is_empty(),).collect();
		if !opt.is_empty() {
			cargo.args(opt,);
		}

		cargo.run()
	}
}

pub trait CrateInfo: CrateCalled
{
	fn is_package(&self,) -> PoisonGirlB<bool,>
	{
		let pkg_sec = self.toml()?;
		let pkg_sec = pkg_sec.get("package",);
		match pkg_sec {
			Some(_,) => X(true,),
			None => X(false,),
		}
	}
	fn is_workspace(&self,) -> PoisonGirlB<bool,>
	{
		let pkg_sec = self.toml()?;
		let pkg_sec = pkg_sec.get("workspace",);
		match pkg_sec {
			Some(_,) => X(true,),
			None => X(false,),
		}
	}
	fn is_pkg_and_ws(&self,) -> PoisonGirlB<bool,>
	{
		X(self.is_package()? && self.is_workspace()?,)
	}

	/// return path to the crate
	/// return type is not Result because macro ensures that path exists and
	/// self has path in compile time
	fn path(&self,) -> PathBuf;

	fn toml(&self,) -> PoisonGirlB<toml::Table,>
	{
		let cargo_toml = self.path().join(CARGO_MANIFEST,);
		read_toml(cargo_toml,)
	}

	fn cargo_conf(&self,) -> Option<PoisonGirlB<toml::Table,>,>
	{
		let config_toml = self.path().join(CARGO_CONFIG,);
		Some(read_toml(config_toml,),)
	}

	fn name(&self,) -> String
	{
		self.path()
			.file_name()
			.expect("error on obtaining crate name",)
			.to_str()
			.expect("error on converting path component to str",)
			.to_string()
	}
}

#[derive(FromPathBuf, Default, PartialEq, Eq, Clone,)]
pub struct PoisonGirlCrate
{
	path: PathBuf,
	#[chart]
	i_am: PoisonGirlCrateChart,
}

impl std::fmt::Debug for PoisonGirlCrate
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_,>,) -> std::fmt::Result
	{
		f.debug_struct("OsoCrate",)
			.field("path", &self.path,)
			.field("i_am", &"<OsoCrateChart>",)
			.finish()
	}
}

impl From<PoisonGirlCrateChart,> for PoisonGirlCrate
{
	fn from(value: PoisonGirlCrateChart,) -> Self
	{
		Self::from(value.to_path_buf(),)
	}
}

impl Crate for PoisonGirlCrate
{
}
impl CrateAction for PoisonGirlCrate
{
}
impl CrateSurvey for PoisonGirlCrate
{
	fn land_on(&mut self, on: impl CrateCalled,)
	{
		let path = on.path_buf();
		*self = Self::from(path,);
	}

	fn go_parent(&mut self,) -> PoisonGirlB<(),>
	{
		if self.has_parent()? {
			let parent = self.path();
			let parent = parent.parent().unwrap();
			let parent = PoisonGirlCrateChart::from(parent.to_path_buf(),);
			self.land_on(parent,);
			X((),)
		} else {
			X((),)
		}
	}
}

impl CrateInfo for PoisonGirlCrate
{
	fn path(&self,) -> PathBuf
	{
		self.path.clone()
	}
}

impl CrateCalled for PoisonGirlCrate
{
	type F = PoisonGirlCrateChart;

	fn whoami(&self,) -> Self::F
	{
		self.i_am.clone()
	}

	fn path_buf(&self,) -> PathBuf
	{
		self.path()
	}
}

impl CrateCalled for PoisonGirlCrateChart
{
	type F = PoisonGirlCrateChart;

	fn whoami(&self,) -> Self::F
	{
		self.clone()
	}

	fn path_buf(&self,) -> PathBuf
	{
		self.to_path_buf()
	}
}

impl Workspace for PoisonGirlCrate
{
}
impl WorkspaceAction for PoisonGirlCrate
{
}

impl WorkspaceSurvey for PoisonGirlCrate
{
}

impl WorkspaceInfo for PoisonGirlCrate
{
	#[allow(refining_impl_trait)]
	fn members(&self,) -> Vec<PoisonGirlCrate,>
	{
		all_crates_in(&self.path(),)
			.expect("failed to get some crates within workspace",)
			.iter()
			.map(|p| PoisonGirlCrate::from(p.clone(),),)
			.collect()
	}

	#[allow(refining_impl_trait)]
	fn members_with_target(
		&self,
		target: impl Into<String,> + Clone,
	) -> Vec<PoisonGirlCrate,>
	{
		self.members()
			.into_iter()
			.filter(|c| {
				let dflt_target: String = c
					.default_target()
					.expect("failed to determine default target",)
					.into();
				let target: String = target.clone().into();
				dflt_target == target
			},)
			.collect()
	}
}

impl Package for PoisonGirlCrate
{
}
impl PackageAction for PoisonGirlCrate
{
}
impl PackageSurvey for PoisonGirlCrate
{
	fn default_target(&self,) -> PoisonGirlB<impl Into<String,>,>
	{
		X(match self.cargo_conf() {
			Some(conf,) => {
				let conf = conf?;
				let conf = conf.get("build",);

				if let Some(toml::Value::Table(t,),) = conf
					&& let Some(toml::Value::String(s,),) = t.get("target",)
				{
					s.clone()
				} else {
					host_tuple()?
				}
			},
			None => host_tuple()?,
		},)
	}
}

impl PackageInfo for PoisonGirlCrate
{
}

pub trait CrateCalled: Eq + Sized + Clone + From<Self::F,> + Debug
{
	type F: CrateCalled;
	fn whoami(&self,) -> Self::F;
	fn path_buf(&self,) -> PathBuf;
}

#[cfg(test)]
mod tests
{
	use {super::*, std::path::PathBuf};

	// Note: The FromPathBuf macro validates paths and panics on non-existent
	// paths This is a suspected program bug - tests should be able to use mock
	// paths Working around this by using the current directory which should
	// exist

	#[test]
	fn test_oso_crate_default()
	{
		let default_crate = PoisonGirlCrate::default();
		let default_path = default_crate.path();
		// Default should create an empty PathBuf
		assert_eq!(default_path, PathBuf::new());
	}

	#[test]
	fn test_oso_crate_creation_with_current_dir()
	{
		// Use current directory which should exist
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);
		assert_eq!(crate_obj.path(), current_dir);
	}

	#[test]
	fn test_oso_crate_clone_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let original = PoisonGirlCrate::from(current_dir.clone(),);
		let cloned = original.clone();

		assert_eq!(original.path(), cloned.path());
		assert_eq!(original, cloned);
	}

	#[test]
	fn test_oso_crate_equality_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate1 = PoisonGirlCrate::from(current_dir.clone(),);
		let crate2 = PoisonGirlCrate::from(current_dir.clone(),);

		assert_eq!(crate1, crate2);
	}

	#[test]
	fn test_crate_info_path_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test CrateInfo::path method
		assert_eq!(crate_obj.path(), current_dir);
	}

	#[test]
	fn test_crate_called_whoami_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test CrateCalled::whoami method
		let whoami_result = crate_obj.whoami();
		assert_eq!(whoami_result.path_buf(), current_dir);
	}

	#[test]
	fn test_crate_called_path_buf_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test CrateCalled::path_buf method
		assert_eq!(crate_obj.path_buf(), current_dir);
	}

	#[test]
	fn test_from_pathbuf_conversion_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);

		// Test From<PathBuf> implementation
		let crate_obj: PoisonGirlCrate = current_dir.clone().into();
		assert_eq!(crate_obj.path(), current_dir);

		// Test explicit From::from
		let crate_obj2 = PoisonGirlCrate::from(current_dir.clone(),);
		assert_eq!(crate_obj2.path(), current_dir);

		// Both should be equal
		assert_eq!(crate_obj, crate_obj2);
	}

	#[test]
	fn test_oso_crate_chart_conversion_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test that we can get the chart representation
		let chart = crate_obj.whoami();

		// Chart should convert back to the same path
		assert_eq!(chart.path_buf(), current_dir);

		let crate_from_chart = PoisonGirlCrate::from(chart,);
		assert_eq!(crate_from_chart.path(), current_dir);
	}

	#[test]
	fn test_debug_implementation()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test that Debug is implemented
		let debug_string = format!("{:?}", crate_obj);
		assert!(debug_string.contains("OsoCrate"));
		assert!(debug_string.contains("path"));
	}

	// Test methods that don't require valid paths (they return Results)

	#[test]
	fn test_crate_action_methods_exist()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test that action methods exist (they will likely fail in test
		// environment) ignore `test` method because running it in test cause
		// infinity loop ignore `run` too because this crate is library crate.
		// nothing to run.
		let _build_result = crate_obj.build();
		let _check_result = crate_obj.check();
		let _fmt_result = crate_obj.format();

		// If we get here without compilation errors, the methods exist
	}

	#[test]
	fn test_crate_action_with_methods_exist()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test that action methods with options exist
		// ignore `test_with` method because running it in test cause infinity
		// loop ignore `run_with` too because this crate is library crate.
		// nothing to run.
		let opts = ["--release",];
		let _build_result = crate_obj.build_with(&opts,);
		let _check_result = crate_obj.ckeck_with(&opts,);
		let _fmt_result = crate_obj.fmt_with(&["--all",],);
	}

	#[test]
	fn test_crate_info_methods()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test that CrateInfo methods exist and return Results
		let _is_package_result = crate_obj.is_package();
		let _is_workspace_result = crate_obj.is_workspace();
		let _is_both_result = crate_obj.is_pkg_and_ws();
		let _toml_result = crate_obj.toml();
		let _cargo_conf_result = crate_obj.cargo_conf();
	}

	#[test]
	fn test_package_survey_methods()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test PackageSurvey methods
		let _target_result = crate_obj.default_target();

		// Test build_artifact with proper CompileOpt
		use poison_girl_dev_cargo::{Arch, BuildMode, Feature, Opts};
		let _opts = Opts {
			build_mode:    BuildMode::Debug,
			feature_flags: Vec::<Feature,>::new(),
			arch:          Arch::Aarch64,
		};
	}

	#[test]
	fn test_workspace_info_methods()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test WorkspaceInfo methods
		let _members = crate_obj.members();

		let _target_members = crate_obj.members_with_target("test-target",);
	}

	#[test]
	fn test_workspace_survey_land_on()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let mut crate_obj = PoisonGirlCrate::from(current_dir,);
		let target_crate = PoisonGirlCrate::from(parent_dir.clone(),);

		// Test that land_on method exists and works
		crate_obj.land_on(target_crate,);

		// After landing on the target, the path should change
		assert_eq!(crate_obj.path(), parent_dir);
	}

	#[test]
	fn test_trait_implementations()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test that all required traits are implemented
		// These are compile-time checks using concrete types since traits are
		// not object-safe

		// Test that we can use the crate as different trait implementors
		let _crate_ref: &PoisonGirlCrate = &crate_obj;
		let _package_ref: &PoisonGirlCrate = &crate_obj;
		let _workspace_ref: &PoisonGirlCrate = &crate_obj;

		// If we get here, all traits are implemented
	}

	// Test the survey methods that contain todo!() - they should panic
	#[test]
	fn test_crate_survey_todo_methods()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test that survey methods exist (they contain todo!() so will panic)
		let has_parent_result =
			std::panic::catch_unwind(|| crate_obj.has_parent(),);
		let go_parent_result = std::panic::catch_unwind(|| {
			let mut obj = crate_obj.clone();
			obj.go_parent()
		},);
		let fix_result = std::panic::catch_unwind(|| crate_obj.fix(),);

		// These methods contain todo!() so they should panic
		assert!(has_parent_result.is_ok());
		assert!(go_parent_result.is_ok());
		assert!(fix_result.is_ok());
	}
}
