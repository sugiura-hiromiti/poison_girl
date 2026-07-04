use {
	super::{CrateInfo, PoisonGirlCrate},
	crate::decl_manage::package::{
		Package, PackageAction, PackageInfo, PackageSurvey,
	},
	poison_girl_dev_cargo::host_tuple_by_rustc,
	poison_girl_dev_error::{PoisonGirlB, X},
};

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
