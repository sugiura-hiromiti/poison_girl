use {
	super::{CrateCalled, CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart},
	poison_girl_dev_error::{
		PathNotFound, PoisonGirlB, ReShape, X, poison_girl_err,
	},
	poison_girl_dev_fs::{CARGO_MANIFEST, search_upstream_at},
};

pub trait CrateSurvey: CrateInfo
{
	fn has_parent(&self,) -> PoisonGirlB<bool,>
	{
		let path = self.path();
		X(search_upstream_at(&path, CARGO_MANIFEST,)?.is_some(),)
	}

	fn go_parent(&mut self,) -> PoisonGirlB<(),>;

	fn fix_manifest(&self,) -> PoisonGirlB<(),>
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

	fn land_on(&mut self, on: impl CrateCalled,) -> PoisonGirlB<(),>;
}

impl CrateSurvey for PoisonGirlCrate
{
	fn land_on(&mut self, on: impl CrateCalled,) -> PoisonGirlB<(),>
	{
		let path = on.path_buf();
		*self = Self::from(path,);
		X((),)
	}

	fn go_parent(&mut self,) -> PoisonGirlB<(),>
	{
		if self.has_parent()? {
			let parent = self.path();
			let parent = parent.parent().reshape(poison_girl_err!(
				PathNotFound::new("parent directory do not exist")
			),)?;
			let parent = PoisonGirlCrateChart::from(parent.to_path_buf(),);
			self.land_on(parent,)?;
			X((),)
		} else {
			X((),)
		}
	}
}
