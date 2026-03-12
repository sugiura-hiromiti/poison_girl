#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_from_path_buf as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::drv,
};

drv! {
	FromPathBuf,
	from_path_buf,
	[as syn::DeriveInput,],
	attributes: chart,
	r#""#
}
