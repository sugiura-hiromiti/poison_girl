use crate::{
	cli_interface::{CargoInvocation, RenderCargoInvocation},
	decl_manage::crate_::PoisonGirlCrateChart,
};

pub struct PackagePolicy
{
	package: PoisonGirlCrateChart,
}

impl PackagePolicy
{
	pub(crate) fn new(package: PoisonGirlCrateChart,) -> Self
	{
		Self { package, }
	}
}

impl RenderCargoInvocation for PackagePolicy
{
	fn render(&self,) -> CargoInvocation
	{
		CargoInvocation::from_cargo_args(vec![
			"-p".to_string(),
			self.package.package_name().to_string(),
		],)
	}
}
