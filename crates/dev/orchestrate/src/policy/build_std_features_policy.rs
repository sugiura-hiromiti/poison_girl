use std::collections::HashSet;

use crate::{AsCargoOpt, cli_interface::CargoInvocationArgs};

pub trait BuildStdFeaturesPolicyResolver
{
	fn build_std_features_policies(&self,) -> BuildStdFeaturesPolicies;
}

pub struct BuildStdFeaturesPolicies(HashSet<BuildStdFeaturesPolicy,>,);

impl AsCargoOpt for BuildStdFeaturesPolicies
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let build_std_features: Vec<_,> =
			self.0.iter().map(BuildStdFeaturesPolicy::as_ref,).collect();

		let cargo_args = if build_std_features.is_empty() {
			vec![]
		} else {
			vec![
				"-Z".to_string(),
				format!("build-std-features={}", build_std_features.join(",",),),
			]
		};
		CargoInvocationArgs::from_cargo_args(cargo_args,)
	}
}

impl From<Vec<BuildStdFeaturesPolicy,>,> for BuildStdFeaturesPolicies
{
	fn from(value: Vec<BuildStdFeaturesPolicy,>,) -> Self
	{
		let hash_set = HashSet::from_iter(value,);
		Self(hash_set,)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, strum_macros::AsRefStr,)]
pub enum BuildStdFeaturesPolicy
{
	#[strum(serialize = "compiler-builtins-mem")]
	CompilerBuiltinsMem,
}
