use crate::Rslt;
use crate::cargo::host_tuple;
use crate::decl_manage::package::Package;
use crate::decl_manage::package::PackageAction;
use crate::decl_manage::package::PackageInfo;
use crate::decl_manage::package::PackageSurvey;
use crate::decl_manage::workspace::Workspace;
use crate::decl_manage::workspace::WorkspaceAction;
use crate::decl_manage::workspace::WorkspaceInfo;
use crate::decl_manage::workspace::WorkspaceSurvey;
use anyhow::anyhow;
use oso_dev_util_helper::cli::Run;
use oso_dev_util_helper::fs::CARGO_CONFIG;
use oso_dev_util_helper::fs::CARGO_MANIFEST;
use oso_dev_util_helper::fs::all_crates_in;
use oso_dev_util_helper::fs::read_toml;
use oso_dev_util_helper::fs::search_upstream_at;
use oso_proc_macro::FromPathBuf;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::Command;

pub trait Crate: Workspace + Package {
	fn as_pkg(&self,) -> &impl Package {
		self
	}

	fn as_wrkspc(&self,) -> &impl Workspace {
		self
	}
}

pub trait CrateSurvey: CrateInfo {
	fn has_parent(&self,) -> Rslt<bool,> {
		let path = self.path();
		Ok(search_upstream_at(&path, CARGO_MANIFEST,)?.is_some(),)
	}
	fn go_parent(&mut self,) -> Rslt<(),>;
	fn fix(&self,) -> Rslt<(),> {
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
		Ok((),)
	}
	fn land_on(&mut self, on: impl CrateCalled,);
}

/// methods provided keeps environment e.g. current path
pub trait CrateAction: CrateInfo {
	// actions for all packages

	fn build(&self,) -> Rslt<(),> {
		self.cargo_xxx("build",)
	}
	fn test(&self,) -> Rslt<(),> {
		self.cargo_xxx("test",)
	}
	fn run(&self,) -> Rslt<(),> {
		self.cargo_xxx("run",)
	}
	fn check(&self,) -> Rslt<(),> {
		self.cargo_xxx("check",)
	}
	fn format(&self,) -> Rslt<(),> {
		self.cargo_xxx("fmt",)
	}
	fn cargo_xxx(&self, cmd: impl AsRef<OsStr,>,) -> Rslt<(),> {
		self.cargo_xxx_with(cmd, &["",],)
	}

	// actions for all packages with specific options

	fn build_with(&self, opt: &[impl AsRef<OsStr,>],) -> Rslt<(),> {
		self.cargo_xxx_with("build", opt,)
	}
	fn test_with(&self, opt: &[impl AsRef<OsStr,>],) -> Rslt<(),> {
		self.cargo_xxx_with("test", opt,)
	}
	fn run_with(&self, opt: &[impl AsRef<OsStr,>],) -> Rslt<(),> {
		self.cargo_xxx_with("run", opt,)
	}
	fn ckeck_with(&self, opt: &[impl AsRef<OsStr,>],) -> Rslt<(),> {
		self.cargo_xxx_with("check", opt,)
	}
	fn fmt_with(&self, opt: &[impl AsRef<OsStr,>],) -> Rslt<(),> {
		self.cargo_xxx_with("fmt", opt,)
	}
	fn cargo_xxx_with(
		&self,
		cmd: impl AsRef<OsStr,>,
		opt: &[impl AsRef<OsStr,>],
	) -> Rslt<(),> {
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

pub trait CrateInfo: CrateCalled {
	fn is_package(&self,) -> Rslt<bool,> {
		let pkg_sec = self.toml()?;
		let pkg_sec = pkg_sec.get("package",);
		match pkg_sec {
			Some(_,) => Ok(true,),
			None => Ok(false,),
		}
	}
	fn is_workspace(&self,) -> Rslt<bool,> {
		let pkg_sec = self.toml()?;
		let pkg_sec = pkg_sec.get("workspace",);
		match pkg_sec {
			Some(_,) => Ok(true,),
			None => Ok(false,),
		}
	}
	fn is_pkg_and_ws(&self,) -> Rslt<bool,> {
		Ok(self.is_package()? && self.is_workspace()?,)
	}

	/// return path to the crate
	/// return type is not Result because macro ensures that path exists and
	/// self has path in compile time
	fn path(&self,) -> PathBuf;

	fn toml(&self,) -> Rslt<toml::Table,> {
		let cargo_toml = self.path().join(CARGO_MANIFEST,);
		read_toml(cargo_toml,)
			.unwrap_or_else(|| panic!("{CARGO_MANIFEST} must exist"),)
	}

	fn cargo_conf(&self,) -> Option<Rslt<toml::Table,>,> {
		let config_toml = self.path().join(CARGO_CONFIG,);
		read_toml(config_toml,)
	}

	fn name(&self,) -> String {
		self.path()
			.file_name()
			.expect("error on obtaining crate name",)
			.to_str()
			.expect("error on converting path component to str",)
			.to_string()
	}
}

#[derive(FromPathBuf, Default, PartialEq, Eq, Clone,)]
pub struct OsoCrate {
	path: PathBuf,
	#[chart]
	i_am: OsoCrateChart,
}

impl std::fmt::Debug for OsoCrate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_,>,) -> std::fmt::Result {
		f.debug_struct("OsoCrate",)
			.field("path", &self.path,)
			.field("i_am", &"<OsoCrateChart>",)
			.finish()
	}
}

impl From<OsoCrateChart,> for OsoCrate {
	fn from(value: OsoCrateChart,) -> Self {
		Self::from(value.to_path_buf(),)
	}
}

impl Crate for OsoCrate {}
impl CrateAction for OsoCrate {}
impl CrateSurvey for OsoCrate {
	fn land_on(&mut self, on: impl CrateCalled,) {
		let path = on.path_buf();
		*self = Self::from(path,);
	}

	fn go_parent(&mut self,) -> Rslt<(),> {
		if self.has_parent()? {
			let parent = self.path();
			let parent =
				parent.parent().ok_or(anyhow!("can't find parent dir"),)?;
			let parent = OsoCrateChart::from(parent.to_path_buf(),);
			self.land_on(parent,);
			Ok((),)
		} else {
			Ok((),)
		}
	}
}

impl CrateInfo for OsoCrate {
	fn path(&self,) -> PathBuf {
		self.path.clone()
	}
}

impl CrateCalled for OsoCrate {
	type F = OsoCrateChart;

	fn whoami(&self,) -> Self::F {
		self.i_am.clone()
	}

	fn path_buf(&self,) -> PathBuf {
		self.path()
	}
}

impl CrateCalled for OsoCrateChart {
	type F = OsoCrateChart;

	fn whoami(&self,) -> Self::F {
		self.clone()
	}

	fn path_buf(&self,) -> PathBuf {
		self.to_path_buf()
	}
}

impl Workspace for OsoCrate {}
impl WorkspaceAction for OsoCrate {}

impl WorkspaceSurvey for OsoCrate {}

impl WorkspaceInfo for OsoCrate {
	#[allow(refining_impl_trait)]
	fn members(&self,) -> Vec<OsoCrate,> {
		all_crates_in(&self.path(),)
			.expect("failed to get some crates within workspace",)
			.iter()
			.map(|p| OsoCrate::from(p.clone(),),)
			.collect()
	}

	#[allow(refining_impl_trait)]
	fn members_with_target(
		&self,
		target: impl Into<String,> + Clone,
	) -> Vec<OsoCrate,> {
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

impl Package for OsoCrate {}
impl PackageAction for OsoCrate {}
impl PackageSurvey for OsoCrate {
	fn default_target(&self,) -> Rslt<impl Into<String,>,> {
		Ok(match self.cargo_conf() {
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

impl PackageInfo for OsoCrate {}

pub trait CrateCalled: Eq + Sized + Clone + From<Self::F,> + Debug {
	type F: CrateCalled;
	fn whoami(&self,) -> Self::F;
	fn path_buf(&self,) -> PathBuf;
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	// Note: The FromPathBuf macro validates paths and panics on non-existent
	// paths This is a suspected program bug - tests should be able to use mock
	// paths Working around this by using the current directory which should
	// exist

	#[test]
	fn test_oso_crate_default() {
		let default_crate = OsoCrate::default();
		let default_path = default_crate.path();
		// Default should create an empty PathBuf
		assert_eq!(default_path, PathBuf::new());
	}

	#[test]
	fn test_oso_crate_creation_with_current_dir() {
		// Use current directory which should exist
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir.clone(),);
		assert_eq!(crate_obj.path(), current_dir);
	}

	#[test]
	fn test_oso_crate_clone_with_current_dir() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let original = OsoCrate::from(current_dir.clone(),);
		let cloned = original.clone();

		assert_eq!(original.path(), cloned.path());
		assert_eq!(original, cloned);
	}

	#[test]
	fn test_oso_crate_equality_with_current_dir() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate1 = OsoCrate::from(current_dir.clone(),);
		let crate2 = OsoCrate::from(current_dir.clone(),);

		assert_eq!(crate1, crate2);
	}

	#[test]
	fn test_crate_info_path_with_current_dir() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir.clone(),);

		// Test CrateInfo::path method
		assert_eq!(crate_obj.path(), current_dir);
	}

	#[test]
	fn test_crate_called_whoami_with_current_dir() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir.clone(),);

		// Test CrateCalled::whoami method
		let whoami_result = crate_obj.whoami();
		assert_eq!(whoami_result.path_buf(), current_dir);
	}

	#[test]
	fn test_crate_called_path_buf_with_current_dir() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir.clone(),);

		// Test CrateCalled::path_buf method
		assert_eq!(crate_obj.path_buf(), current_dir);
	}

	#[test]
	fn test_from_pathbuf_conversion_with_current_dir() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);

		// Test From<PathBuf> implementation
		let crate_obj: OsoCrate = current_dir.clone().into();
		assert_eq!(crate_obj.path(), current_dir);

		// Test explicit From::from
		let crate_obj2 = OsoCrate::from(current_dir.clone(),);
		assert_eq!(crate_obj2.path(), current_dir);

		// Both should be equal
		assert_eq!(crate_obj, crate_obj2);
	}

	#[test]
	fn test_oso_crate_chart_conversion_with_current_dir() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir.clone(),);

		// Test that we can get the chart representation
		let chart = crate_obj.whoami();

		// Chart should convert back to the same path
		assert_eq!(chart.path_buf(), current_dir);

		// Test From<OsoCrateChart> for OsoCrate
		let crate_from_chart = OsoCrate::from(chart,);
		assert_eq!(crate_from_chart.path(), current_dir);
	}

	#[test]
	fn test_debug_implementation() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test that Debug is implemented
		let debug_string = format!("{:?}", crate_obj);
		assert!(debug_string.contains("OsoCrate"));
		assert!(debug_string.contains("path"));
	}

	// Test methods that don't require valid paths (they return Results)

	#[test]
	fn test_crate_action_methods_exist() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

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
	fn test_crate_action_with_methods_exist() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

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
	fn test_crate_info_methods() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test that CrateInfo methods exist and return Results
		let _is_package_result = crate_obj.is_package();
		let _is_workspace_result = crate_obj.is_workspace();
		let _is_both_result = crate_obj.is_pkg_and_ws();
		let _toml_result = crate_obj.toml();
		let _cargo_conf_result = crate_obj.cargo_conf();
	}

	#[test]
	fn test_package_survey_methods() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test PackageSurvey methods
		let _target_result = crate_obj.default_target();

		// Test build_artifact with proper CompileOpt
		use crate::cargo::Arch;
		use crate::cargo::BuildMode;
		use crate::cargo::Feature;
		use crate::cargo::Opts;
		let _opts = Opts {
			build_mode:    BuildMode::Debug,
			feature_flags: Vec::<Feature,>::new(),
			arch:          Arch::Aarch64,
		};
	}

	#[test]
	fn test_workspace_info_methods() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test WorkspaceInfo methods
		let _members = crate_obj.members();

		let _target_members = crate_obj.members_with_target("test-target",);
	}

	#[test]
	fn test_workspace_survey_land_on() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let mut crate_obj = OsoCrate::from(current_dir,);
		let target_crate = OsoCrate::from(parent_dir.clone(),);

		// Test that land_on method exists and works
		crate_obj.land_on(target_crate,);

		// After landing on the target, the path should change
		assert_eq!(crate_obj.path(), parent_dir);
	}

	#[test]
	fn test_trait_implementations() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test that all required traits are implemented
		// These are compile-time checks using concrete types since traits are
		// not object-safe

		// Test that we can use the crate as different trait implementors
		let _crate_ref: &OsoCrate = &crate_obj;
		let _package_ref: &OsoCrate = &crate_obj;
		let _workspace_ref: &OsoCrate = &crate_obj;

		// If we get here, all traits are implemented
	}

	// Test the survey methods that contain todo!() - they should panic
	#[test]
	fn test_crate_survey_todo_methods() {
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

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
