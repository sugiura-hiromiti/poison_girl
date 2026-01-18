use {
	crate::{
		cargo::{CompileOpt, Opts},
		decl_manage::{
			crate_::{Crate, CrateInfo, OsoCrate},
			package::PackageSurvey,
		},
	},
	poison_girl_dev_error::{PoisonGirlB, X, Y},
	poison_girl_dev_fs::fs::search_in_with,
	std::path::PathBuf,
};

pub mod crate_;
pub mod package;
pub mod workspace;

pub trait CargoCrate {
	fn specified_target(&self,) -> PoisonGirlB<impl Into<String,>,>;
	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,>;
	fn as_crate(&self,) -> &impl Crate;
	fn as_opts(&self,) -> &impl CompileOpt;
}

pub struct OsoCargoInterface {
	ws:  OsoCrate,
	opt: Opts,
}

impl CargoCrate for OsoCargoInterface {
	fn specified_target(&self,) -> PoisonGirlB<impl Into<String,>,> {
		let search_rslt = search_in_with(&self.ws.path(), |entry| {
			let file_name = entry
				.as_ref()
				.expect("file io error",)
				.file_name()
				.to_string_lossy()
				.to_string();
			let arch = self.opt.arch().into();

			file_name.contains(&arch,) && file_name.ends_with(".json",)
		},);

		match search_rslt {
			X(Some(p,),) => X(p.to_string_lossy().to_string(),),
			X(None,) => self.ws.default_target().map(|s| s.into(),),
			Y(e,) => Y(e,),
		}
	}

	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,> {
		X(self
			.ws
			.path()
			.join("target",)
			.join(self.specified_target()?.into(),)
			.join(self.opt.build_mode().into(),),)
	}

	fn as_crate(&self,) -> &impl Crate {
		&self.ws
	}

	fn as_opts(&self,) -> &impl CompileOpt {
		&self.opt
	}
}
