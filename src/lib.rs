#![feature(string_from_utf8_lossy_owned)]
#![feature(exit_status_error)]

use poison_girl_dev_orchestrate::cargo::Assets;
use poison_girl_dev_orchestrate::cargo::Opts;
use poison_girl_dev_orchestrate::decl_manage::crate_::OsoCrate;

pub mod builder;
pub mod qemu;

pub struct Xtask {
	opts:   Opts,
	ws:     OsoCrate,
	assets: Assets,
}
