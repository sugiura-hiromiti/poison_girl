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
	std::path::PathBuf,
};

pub mod crate_;
pub mod package;
pub mod workspace;

pub trait CargoCrate
{
	fn specified_target(&self,) -> PoisonGirlB<impl Into<String,>,>;
	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,>;
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

impl CargoCrate for PoisonGirlCargoInterface
{
	fn specified_target(&self,) -> PoisonGirlB<impl Into<String,>,>
	{
		let search_rslt = search_in_with(&self.ws.path(), |entry| {
			let file_name = entry
				.as_ref()
				.expect("file io error",)
				.file_name()
				.to_string_lossy()
				.to_string();
			let arch = self.opts.arch.to_string();

			X(file_name.contains(arch.as_str(),)
				&& file_name.ends_with(".json",),)
		},);

		match search_rslt {
			X(Some(p,),) => X(p.to_string_lossy().to_string(),),
			X(None,) => self.ws.default_target().map(|s| s.into(),),
			Y(e,) => Y(e,),
		}
	}

	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,>
	{
		todo!("cargo metadataを利用するように変更");
		X(self
			.ws
			.path()
			.join("target",)
			.join(self.specified_target()?.into(),)
			.join(self.opts.build_mode().into(),),)
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
