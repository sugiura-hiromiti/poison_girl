use {
	crate::decl_manage::crate_::{CrateAction, CrateInfo, CrateSurvey},
	poison_girl_dev_error::PoisonGirlB,
	std::path::PathBuf,
};

pub trait Package: PackageAction + PackageSurvey
{
	fn as_action(&self,) -> &impl PackageAction
	{
		self
	}

	fn as_survey(&self,) -> &impl PackageSurvey
	{
		self
	}
}

pub trait PackageAction: PackageInfo + CrateAction
{
}
pub trait PackageSurvey: PackageInfo + CrateSurvey
{
	fn default_target(&self,) -> PoisonGirlB<impl Into<String,>,>;
}

pub trait PackageInfo: Sized + CrateInfo
{
}
