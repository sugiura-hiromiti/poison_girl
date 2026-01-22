#![feature(string_from_utf8_lossy_owned)]
#![feature(exit_status_error)]

use std::path::PathBuf;

use poison_girl_dev_orchestrate::{
	cargo::{Arch, Assets, Opts},
	decl_manage::crate_::OsoCrate,
};

pub mod builder;
pub mod qemu;

pub struct Xtask {
	opts:   Opts,
	ws:     OsoCrate,
	assets: Assets,
}

impl Xtask {
	fn arch(&self,) -> Arch {
		self.opts.arch
	}

	fn firmware_code(&self,) -> &PathBuf {
		&self.assets.firmware.code
	}

	fn firmware_vars(&self,) -> &PathBuf {
		&self.assets.firmware.vars
	}
}
