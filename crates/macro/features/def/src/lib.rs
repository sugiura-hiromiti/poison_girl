#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_features as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::atr,
};

// TODO: correct doc comment.

atr! {
	features,
	[as syn::Expr,],
	[as syn::ItemEnum,],
	r#"# Params

this attribute macro takes one argument
the argument is datatype that the subject enum implements conversion method.
for example, if you specified `PoisonGirlCrateChart`,
then the subject enum implements method like
`fn from_pgcc(pgcc: PoisonGirlCrateChart) -> Vec<Self>`
and
`fn into_pgcc(self) -> Vec<PoisonGirlCrateChart>`.

NOTE: parameter datatype have to implement both From<PathBuf> and Into<PathBuf>."#
}
