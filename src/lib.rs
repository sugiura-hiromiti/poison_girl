#![feature(string_from_utf8_lossy_owned)]
#![feature(exit_status_error)]

// TODO: workspace内の未使用クレーとを検出

use {
	poison_girl_dev_cargo::{Arch, Assets, Opts},
	poison_girl_dev_orchestrate::decl_manage::crate_::PoisonGirlCrate,
	std::path::PathBuf,
};

// each module are basically mapped to functionality of named subcommand

/// orchestrate running qemu process
pub mod builder;
// just a wrapper of cargo check/clippy
// just a wrapper of cargo fmt
// detail implementation of qemu orchestration
pub mod qemu;
// difference from running default cargo t command is xtask version also runs
// emulation tests with qemu

pub struct Xtask
{
	opts:   Opts,
	ws:     PoisonGirlCrate,
	assets: Assets,
}

impl Xtask
{
	fn arch(&self,) -> Arch
	{
		self.opts.arch
	}

	fn firmware_code(&self,) -> &PathBuf
	{
		&self.assets.firmware.code
	}

	fn firmware_vars(&self,) -> &PathBuf
	{
		&self.assets.firmware.vars
	}
}
