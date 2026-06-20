use poison_girl_dev_cargo::{Arch, Runtime};

use crate::{AsCargoOpt, cli_interface::CargoInvocationArgs};

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
	type Out = CargoInvocationArgs;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let Some(tuple,) = self.target_spec() else {
			return CargoInvocationArgs::default();
		};

		let mut cargo_args = vec!["--target".to_owned(), tuple];
		if self.has_json_spec() {
			cargo_args
				.extend(["-Z".to_owned(), "json-target-spec".to_owned(),],);
		}

		CargoInvocationArgs::from_cargo_args(cargo_args,)
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

		assert_eq!(
			policy.as_cargo_opt().into_cargo_args(),
			Vec::<String,>::new()
		);
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
			policy.as_cargo_opt().into_cargo_args(),
			vec![
				"--target".to_string(),
				target,
				"-Z".to_string(),
				"json-target-spec".to_string(),
			]
		);
	}
}
