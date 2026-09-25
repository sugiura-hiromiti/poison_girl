use {
	crate::cli_interface::{CargoInvocation, RenderCargoInvocation},
	std::{collections::HashSet, hash_map},
};

pub struct BuildStdFeaturesPolicies(HashSet<BuildStdFeaturesPolicy,>,);

impl RenderCargoInvocation for BuildStdFeaturesPolicies
{
	fn render(&self,) -> CargoInvocation
	{
		let build_std_features: Vec<_,> =
			self.0.iter().map(BuildStdFeaturesPolicy::as_ref,).collect();

		let env_val = build_std_features
			.into_iter()
			.map(|s| s.to_string(),)
			.collect::<Vec<String,>>()
			.join(",",);
		let build_std_features_env_var_name =
			"CARGO_UNSTABLE_BUILD_STD_FEATURES".to_string();
		CargoInvocation::from_env(
			hash_map! { build_std_features_env_var_name => env_val },
		)
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
