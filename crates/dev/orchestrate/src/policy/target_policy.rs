use poison_girl_dev_cargo::{Arch, Runtime};

use crate::AsCargoOpt;

pub trait TargetPolicyResolver
{
	fn target_policy(&self,) -> TargetPolicy;
}

pub struct TargetPolicy
{
	arch:    Arch,
	runtime: Runtime,
}

impl TargetPolicy
{
	pub fn new(arch: Arch, runtime: Runtime,) -> Self
	{
		Self { arch, runtime, }
	}

	pub fn runtime(&self,) -> &Runtime
	{
		&self.runtime
	}

	pub fn target_spec(&self,) -> Option<String,>
	{
		let mut tuple = self.target_tuple();
		if self.has_json_spec() {
			tuple.as_mut()?.push_str(".json",);
		}

		tuple
	}

	fn has_json_spec(&self,) -> bool
	{
		matches!(self.runtime, Runtime::PoisonGirl)
	}

	pub fn target_tuple(&self,) -> Option<String,>
	{
		let arch = self.arch.as_ref();
		let tuple = match self.runtime() {
			Runtime::Host => return None,
			Runtime::Efi => [arch, "unknown-uefi",].join("-",),
			Runtime::PoisonGirl => {
				[arch, "sugiura_hiromiti-poison_girl-elf",].join("-",)
			},
		};

		Some(tuple,)
	}
}

impl AsCargoOpt for TargetPolicy
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let Some(tuple,) = self.target_spec() else { return vec![] };

		let mut rslt = vec!["--target".to_owned(), tuple];
		if self.has_json_spec() {
			rslt.extend(["-Z".to_owned(), "json-target-spec".to_owned(),],);
		}

		rslt
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn host_target_emits_no_cargo_opts()
	{
		let policy = TargetPolicy::new(Arch::Aarch64, Runtime::Host,);

		assert_eq!(policy.as_cargo_opt(), Vec::<String,>::new());
	}

	#[test]
	fn poison_girl_target_emits_json_target_opts()
	{
		let policy = TargetPolicy::new(Arch::Aarch64, Runtime::PoisonGirl,);
		let target = format!(
			"{}-sugiura_hiromiti-poison_girl-elf.json",
			Arch::Aarch64.as_ref()
		);

		assert_eq!(
			policy.as_cargo_opt(),
			vec![
				"--target".to_string(),
				target,
				"-Z".to_string(),
				"json-target-spec".to_string(),
			]
		);
	}
}
