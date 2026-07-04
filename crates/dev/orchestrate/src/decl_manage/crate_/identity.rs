use {
	poison_girl_macro_def_from_path_buf::FromPathBuf,
	std::{fmt::Debug, path::PathBuf},
};

#[derive(FromPathBuf, Default, PartialEq, Eq, Clone,)]
pub struct PoisonGirlCrate
{
	pub(super) path: PathBuf,
	#[chart]
	pub(super) i_am: PoisonGirlCrateChart,
}

/// this block is subject and responsible for crate name change
impl PoisonGirlCrateChart
{
	pub const KERNEL: Self = Self::Kernel;
	pub const LOADER: Self = Self::Loader;
	pub const XTASK: Self = Self::PoisonGirl;

	pub fn uses_custom_runtime(&self,) -> bool
	{
		matches!(self, Self::Kernel | Self::Loader)
	}
}

impl PoisonGirlCrate
{
	pub fn as_chart(&self,) -> &PoisonGirlCrateChart
	{
		&self.i_am
	}
}

impl Debug for PoisonGirlCrate
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_,>,) -> std::fmt::Result
	{
		f.debug_struct("PoisonGirlCrate",)
			.field("path", &self.path,)
			.field("i_am", &"<PoisonGirlCrateChart>",)
			.finish()
	}
}

impl From<PoisonGirlCrateChart,> for PoisonGirlCrate
{
	fn from(value: PoisonGirlCrateChart,) -> Self
	{
		Self::from(value.to_path_buf(),)
	}
}

pub trait CrateCalled: Eq + Sized + Clone + From<Self::F,> + Debug
{
	type F: CrateCalled;
	fn whoami(&self,) -> Self::F;
	fn path_buf(&self,) -> PathBuf;
}

impl CrateCalled for PoisonGirlCrate
{
	type F = PoisonGirlCrateChart;

	fn whoami(&self,) -> Self::F
	{
		self.i_am
	}

	fn path_buf(&self,) -> PathBuf
	{
		self.path.clone()
	}
}

impl CrateCalled for PoisonGirlCrateChart
{
	type F = PoisonGirlCrateChart;

	fn whoami(&self,) -> Self::F
	{
		*self
	}

	fn path_buf(&self,) -> PathBuf
	{
		self.to_path_buf()
	}
}
