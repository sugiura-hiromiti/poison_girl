use {
	crate::decl_manage::{
		package::{Package, PackageAction, PackageInfo, PackageSurvey},
		workspace::{
			Workspace, WorkspaceAction, WorkspaceInfo, WorkspaceSurvey,
		},
	},
	poison_girl_dev_cargo::host_tuple_by_rustc,
	poison_girl_dev_cli::Run,
	poison_girl_dev_error::{
		Container, PathIsNotValidUtf8, PathNotFound, PoisonGirlB, ReShape, X,
		poison_girl_err,
	},
	poison_girl_dev_fs::{
		CARGO_CONFIG, CARGO_MANIFEST, all_crates_in, read_toml,
		search_upstream_at,
	},
	poison_girl_dev_util::toml_tools::TomlMerge,
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
			&& let true_name = self.name()?
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
			.map(|toml| toml.unwrap_or(toml::map::Map::new(),),)
	}

	/// NOTE: we've changed return type from `PoisonGirlB<Option<toml::Table>>`
	/// to `PoisonGirlB<toml::Table>` this is because, `cargo.toml` is
	/// originally optional and it has default fallback. so empty `cargo.toml`
	/// is completely vaild, in this case, distinguishing the file existence is
	/// meaningless. instead, this function provides simple semantics: the
	/// config value exists or not.
	fn cargo_conf(&self,) -> PoisonGirlB<toml::Table,>
	{
		// let config_toml = self.path().join(CARGO_CONFIG,);
		// if !config_toml.exists() {
		// 	return None;
		// }
		// Some(read_toml(config_toml,),)
		let mut path = self.path().join(CARGO_CONFIG,);
		// we can treat this as true depth of the path because `path` is
		// canonicalized.
		let depth = path.components().collect::<Vec<_,>>().len() - 1;
		let global_cargo_config_path = std::env::var("CARGO_HOME",)
			.map_or_else(|_| std::env::home_dir(), |s| Some(PathBuf::from(s,),),)
			.reshape(poison_girl_err!(PathNotFound::new("home directory")),)?;

		path.pop();
		(0..depth)
			.map(|_| {
				path.pop();
				// path.push(CARGO_CONFIG,);
				path.clone()
				// read_toml(&path,)
			},)
			.chain([global_cargo_config_path,],)
			.rev()
			.map(|mut p| {
				p.push(CARGO_CONFIG,);
				p
			},)
			.map(read_toml,)
			.try_fold(toml::Table::new(), |acc, config| {
				let config = config?;
				let Some(config,) = config else { return X(acc,) };
				let acc = acc.into_updated_by(config,);
				X(acc,)
			},)
	}

	fn name(&self,) -> PoisonGirlB<String,>
	{
		self.path()
			.file_name()
			.reshape(poison_girl_err!(PathNotFound::new(
				"path of the crate is terminated with .. or root",
			)),)?
			.to_str()
			.reshape(poison_girl_err!(PathIsNotValidUtf8),)
			.map(|s| s.to_string(),)
	}
}

#[derive(FromPathBuf, Default, PartialEq, Eq, Clone,)]
pub struct PoisonGirlCrate
{
	path: PathBuf,
	#[chart]
	i_am: PoisonGirlCrateChart,
}

/// this block is subject and responsible for crate name change
impl PoisonGirlCrateChart
{
	pub const KERNEL: Self = Self::Kernel;
	pub const LOADER: Self = Self::Loader;
	pub const XTASK: Self = Self::PoisonGirl;
}

impl PoisonGirlCrate
{
	pub fn as_chart(&self,) -> &PoisonGirlCrateChart
	{
		&self.i_am
	}
}

impl std::fmt::Debug for PoisonGirlCrate
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_,>,) -> std::fmt::Result
	{
		f.debug_struct("PoisonGirlCrate",)
			.field("path", &self.path,)
			.field("i_am", &"<PoisonGirlCrateChart>",)
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
			let parent = parent.parent().reshape(poison_girl_err!(
				PathNotFound::new("parent directory do not exist")
			),)?;
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
		self.i_am
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
		*self
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
		// X(match self.cargo_conf() {
		// 	Some(conf,) => {
		// 		let conf = conf?;
		// 		let conf = conf.get("build",);

		// 		if let Some(toml::Value::Table(t,),) = conf
		// 			&& let Some(toml::Value::String(s,),) = t.get("target",)
		// 		{
		// 			s.clone()
		// 		} else {
		// 			host_tuple_by_rustc()?
		// 		}
		// 	},
		// None => host_tuple_by_rustc()?,
		// },)
		let conf = self.cargo_conf()?;
		let tuple = if conf.is_empty() {
			host_tuple_by_rustc()?
		} else {
			let Some(toml::Value::Table(build,),) = conf.get("build",) else {
				return host_tuple_by_rustc();
			};
			let Some(toml::Value::String(s,),) = build.get("target",) else {
				return host_tuple_by_rustc();
			};
			s.clone()
		};

		X(tuple,)
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
	fn test_poison_girl_crate_default()
	{
		let default_crate = PoisonGirlCrate::default();
		let default_path = default_crate.path();
		// Default should create an empty PathBuf
		assert_eq!(default_path, PathBuf::new());
	}

	#[test]
	fn test_poison_girl_crate_creation_with_current_dir()
	{
		// Use current directory which should exist
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);
		assert_eq!(crate_obj.path(), current_dir);
	}

	#[test]
	fn test_poison_girl_crate_clone_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let original = PoisonGirlCrate::from(current_dir.clone(),);
		let cloned = original.clone();

		assert_eq!(original.path(), cloned.path());
		assert_eq!(original, cloned);
	}

	#[test]
	fn test_poison_girl_crate_equality_with_current_dir()
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
	fn test_poison_girl_crate_chart_conversion_with_current_dir()
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
		assert!(debug_string.contains("PoisonGirlCrate"));
		assert!(debug_string.contains("path"));
	}

	#[test]
	fn test_workspace_survey_land_on()
	{
		let mut crate_obj =
			PoisonGirlCrate::from(PoisonGirlCrateChart::DevOrchestrate,);
		let target_crate = PoisonGirlCrate::from(PoisonGirlCrateChart::DevFs,);
		let target_path = target_crate.path();

		crate_obj.land_on(target_crate,);

		assert_eq!(crate_obj.path(), target_path);
	}
}
