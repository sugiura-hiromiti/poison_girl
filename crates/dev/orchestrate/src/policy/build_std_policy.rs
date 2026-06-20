use std::collections::HashSet;

use crate::{AsCargoOpt, cli_interface::CargoInvocationArgs};

pub trait BuildStdPoliyResolver
{
	fn build_std_policies(&self,) -> BuildStdPolicies;
}

pub struct BuildStdPolicies(HashSet<BuildStdPolicy,>,);

impl AsCargoOpt for BuildStdPolicies
{
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let build_std: Vec<_,> =
			self.0.iter().map(BuildStdPolicy::as_ref,).collect();

		let cargo_args = if build_std.is_empty() {
			vec![]
		} else {
			vec![
				"-Z".to_string(),
				format!("build-std={}", build_std.join(",",),),
			]
		};

		CargoInvocationArgs::from_cargo_args(cargo_args,)
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
