// #![feature(string_from_utf8_lossy_owned)]
// #![feature(exit_status_error)]

// TODO: workspace内の未使用クレーとを検出

use {
	poison_girl_dev_cargo::Assets,
	poison_girl_dev_error::PoisonGirlB,
	poison_girl_dev_orchestrate::{
		Opts,
		decl_manage::{PoisonGirlCargoInterface, crate_::PoisonGirlCrate},
	},
	std::path::PathBuf,
};

/// orchestrate running qemu process
pub mod builder;
/// make up disk image file
mod disk_img;
/// detail implementation of qemu orchestration
pub mod qemu_command;
/// centerize target spec
mod target_spec;

pub struct Xtask
{
	interface: PoisonGirlCargoInterface,
	assets:    Assets,
}

impl Xtask
{
	fn opts(&self,) -> &Opts
	{
		self.interface.task().opts()
	}

	fn ws(&self,) -> PoisonGirlCrate
	{
		self.interface.ws()
	}

	fn firmware_code(&self,) -> PoisonGirlB<PathBuf,>
	{
		self.assets.firmware.code()
	}

	fn firmware_vars(&self,) -> PoisonGirlB<PathBuf,>
	{
		self.assets.firmware.vars()
	}
}
