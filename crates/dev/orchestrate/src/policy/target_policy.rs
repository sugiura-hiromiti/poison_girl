use {
	crate::{
		AsCargoOpt, cli_interface::CargoInvocation,
		decl_manage::crate_::PoisonGirlCrateChart,
	},
	poison_girl_dev_cargo::{Arch, Runtime},
};

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
		if self.has_json_spec()
			&& let Some(ref mut tuple,) = tuple
		{
			tuple.push_str(".json",);
			let kernel_crate_path = PoisonGirlCrateChart::KERNEL.to_path_buf();
			*tuple = kernel_crate_path.join(&*tuple,).display().to_string();
		}

		tuple
	}

	fn has_json_spec(&self,) -> bool
	{
		// matches! can be used for comparing data types which do not implement
		// Eq
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
	type Out = CargoInvocation;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let Some(tuple,) = self.target_spec() else {
			return CargoInvocation::default();
		};

		let mut cargo_args = vec!["--target".to_owned(), tuple];
		if self.has_json_spec() {
			cargo_args
				.extend(["-Z".to_owned(), "json-target-spec".to_owned(),],);
		}

		CargoInvocation::from_cargo_args(cargo_args,)
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
		let target = PoisonGirlCrateChart::KERNEL
			.to_path_buf()
			.join(target,)
			.display()
			.to_string();

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
