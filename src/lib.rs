#![feature(string_from_utf8_lossy_owned)]
#![feature(exit_status_error)]

// TODO: workspace内の未使用クレーとを検出

use {
	poison_girl_dev_cargo::{Arch, Assets, Opts, TargetSpec},
	poison_girl_dev_orchestrate::decl_manage::crate_::PoisonGirlCrate,
	std::{path::PathBuf, process::Command},
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
	opts:   Opts,
	ws:     PoisonGirlCrate,
	assets: Assets,
}

impl Xtask
{
	// fn arch(&self,) -> Arch
	// {
	// 	self.opts.arch
	// }

	fn firmware_code(&self,) -> &PathBuf
	{
		&self.assets.firmware.code
	}

	fn firmware_vars(&self,) -> &PathBuf
	{
		&self.assets.firmware.vars
	}
}
