use std::collections::HashSet;

use crate::AsCargoOpt;

pub trait BuildStdFeaturesPolicyResolver
{
	fn build_std_features_policies(&self,) -> BuildStdFeaturesPolicies;
}

pub struct BuildStdFeaturesPolicies(HashSet<BuildStdFeaturesPolicy,>,);

impl AsCargoOpt for BuildStdFeaturesPolicies
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		// let build_std_features =
		// [BuildStdFeaturesPolicy::CompilerBuiltinsMem,] 	.into_iter()
		// 	.filter(|policy| self.0.contains(policy,),)
		// 	.map(BuildStdFeaturesPolicy::as_cargo_name,)
		// 	.collect::<Vec<_,>>();
		let build_std_features: Vec<_,> =
			self.0.iter().map(BuildStdFeaturesPolicy::as_ref,).collect();

		if build_std_features.is_empty() {
			vec![]
		} else {
			vec![
				"-Z".to_string(),
				format!("build-std-features={}", build_std_features.join(",",),),
			]
		}
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
#[strum(serialize_all = "snake_case")]
pub enum BuildStdFeaturesPolicy
{
	CompilerBuiltinsMem,
}
