use {
	crate::cli_interface::{CargoInvocation, Invocate},
	std::{collections::HashSet, hash_map},
};

pub struct BuildStdPolicies(HashSet<BuildStdPolicy,>,);

impl Invocate for BuildStdPolicies
{
	type Out = CargoInvocation;

	fn invocate(self,) -> Self::Out
	{
		let build_std: Vec<_,> =
			self.0.iter().map(BuildStdPolicy::as_ref,).collect();

		let env_val = build_std
			.into_iter()
			.map(|s| s.to_string(),)
			.collect::<Vec<String,>>()
			.join(",",);

		let build_std_env_var_name = "CARGO_UNSTABLE_BUILD_STD".to_string();
		CargoInvocation::from_env(
			hash_map! { build_std_env_var_name => env_val},
		)
	}
}

impl From<Vec<BuildStdPolicy,>,> for BuildStdPolicies
{
	fn from(value: Vec<BuildStdPolicy,>,) -> Self
	{
		let hash_set = HashSet::from_iter(value,);
		Self(hash_set,)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, strum_macros::AsRefStr,)]
#[strum(serialize_all = "snake_case")]
pub enum BuildStdPolicy
{
	Core,
	Alloc,
	CompilerBuiltins,
}
