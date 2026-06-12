use {
	clap::{Parser, Subcommand},
	ovmf_prebuilt::{FileType, Prebuilt, Source},
	poison_girl_dev_error::{
		HostTupleNotFound, PoisonGirlB, ReShape, X, poison_girl_err,
	},
	poison_girl_macro_def_features::features,
	std::{path::PathBuf, process::Command, str::FromStr},
	strum_macros::Display,
};

pub trait CompileOpt
{
	fn build_mode(&self,) -> impl Into<String,>;
	fn feature_flags(&self,) -> Vec<impl Into<String,>,>;
}

pub trait AsCargoOpt
{
	type Out;
	fn as_cargo_opt(&self,) -> Self::Out;
}

pub trait TargetSpec
{
	fn tuple(&self,) -> String;
	fn arch(&self,) -> Arch;
	fn runtime(&self,) -> Runtime;
}

#[features]
#[derive(
	strum_macros::AsRefStr,
	strum_macros::EnumIs,
	strum_macros::EnumString,
	Clone,
)]
#[strum(serialize_all = "snake_case")]
pub enum Feature {}

impl AsCargoOpt for Vec<Feature,>
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		if self.is_empty() {
			return vec![];
		}

		vec![
			"-F".to_string(),
			self.iter().map(|f| f.as_ref(),).collect::<Vec<_,>>().join(",",),
		]
	}
}

#[derive(Default, Clone,)]
pub struct Opts
{
	pub command:       CliCommand,
	pub build_mode:    BuildMode,
	pub feature_flags: Vec<Feature,>,
	pub arch:          Arch,
	pub lock_deps:     bool,
}

impl Opts
{
	pub fn new() -> Self
	{
		Cli::parse().to_opts()
	}
}

impl AsCargoOpt for Opts
{
	type Out = Vec<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		let Self { command, build_mode, feature_flags, lock_deps, .. } = self;
		let Some(command,) = command.as_cargo_opt() else { return vec![] };
		let build_mode = build_mode.as_cargo_opt();
		let feature_flags = feature_flags.as_cargo_opt();
		// single architecture info itself is useless
		// target tuple is truth
		// let arch = arch.as_cargo_opt();
		let lock_deps =
			if *lock_deps { Some("--locked".to_string(),) } else { None };

		std::iter::once(command,)
			.chain(build_mode,)
			.chain(feature_flags,)
			.chain(lock_deps,)
			.collect()
	}
}

impl CompileOpt for Opts
{
	fn build_mode(&self,) -> impl Into<String,>
	{
		self.build_mode.as_ref()
	}

	fn feature_flags(&self,) -> Vec<impl Into<String,>,>
	{
		self.feature_flags.iter().map(|f| f.as_ref(),).collect()
	}
}

#[derive(clap::Parser, Default,)]
#[command(version, about)]
pub struct Cli
{
	#[arg(value_enum, short)]
	pub build_mode:    Option<BuildMode,>,
	#[arg(short)]
	pub feature_flags: Option<Vec<Feature,>,>,
	#[arg(short)]
	pub arch:          Option<Arch,>,
	#[command(subcommand)]
	pub command:       Option<CliCommand,>,
	#[arg(short, default_value_t = false)]
	pub lock_deps:     bool,
}

impl Cli
{
	pub fn to_opts(self,) -> Opts
	{
		Opts {
			build_mode:    self.build_mode.unwrap_or_default(),
			feature_flags: self.feature_flags.unwrap_or_default(),
			arch:          self.arch.unwrap_or_default(),
			command:       self.command.unwrap_or_default(),
			lock_deps:     self.lock_deps,
		}
	}
}

#[derive(
	Clone,
	Copy,
	clap::ValueEnum,
	Default,
	strum_macros::AsRefStr,
	strum_macros::EnumIs,
	strum_macros::EnumString,
	PartialEq,
	Eq,
	Debug,
	Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum BuildMode
{
	Release,
	#[default]
	Debug,
}

impl AsCargoOpt for BuildMode
{
	type Out = Option<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		match self {
			Self::Release => Some("-r".to_string(),),
			Self::Debug => None,
		}
	}
}

#[derive(
	Subcommand, Default, strum_macros::EnumIs, strum_macros::AsRefStr, Clone,
)]
#[strum(serialize_all = "snake_case")]
pub enum CliCommand
{
	Build,
	Test,
	#[default]
	Run,
	Check
	{
		/// 指定無しの場合はfull check
		#[command(subcommand)]
		kind: Option<CheckKind,>,
	},
	Fmt,
	Fixture,
	Fix,
}

impl AsCargoOpt for CliCommand
{
	type Out = Option<String,>;

	fn as_cargo_opt(&self,) -> Self::Out
	{
		match self {
			Self::Check { .. } => None,
			Self::Fixture => None,
			_ => Some(self.as_ref().to_string(),),
		}
	}
}

#[derive(Subcommand, strum_macros::EnumIter, Clone,)]
pub enum CheckKind
{
	KernelAarch64,
	LoaderAarch64Uefi,
	Clippy,
}

pub enum Runtime
{
	Host,
	Efi,
	PoisonGirl,
}

impl Runtime
{
	// pub fn host() -> PoisonGirlB<Self,>
	// {
	// 	let host_name = host_tuple_by_rustc()?;
	// 	let host_name = host_name.split('-',).next().reshape(
	// 		poison_girl_err!(InvalidHostName::new(
	// 			"host's target tuple of rustc is weird. they do not contain \
	// 			 `-`."
	// 		)),
	// 	)?;
	// 	Runtime::from_str(host_name,)
	// }
}

pub struct Assets
{
	pub firmware: Firmware,
	pub host:     Runtime,
}

impl Assets
{
	pub fn new(arch: Arch,) -> PoisonGirlB<Self,>
	{
		let firmware = Firmware::new(arch,)?;
		X(Self { firmware, host: Runtime::Host, },)
	}
}

/// Manages OVMF firmware files for UEFI boot
#[derive(Debug,)]
pub struct Firmware
{
	/// Path to the OVMF code file
	pub code: PathBuf,
	/// Path to the OVMF variables file
	pub vars: PathBuf,
}

impl Firmware
{
	/// Creates a new Firmware instance for the specified architecture
	///
	/// Downloads the latest OVMF firmware files if they don't exist.
	///
	/// # Parameters
	///
	/// * `arch` - The target architecture
	///
	/// # Returns
	///
	/// A new Firmware instance or an error if initialization fails
	pub fn new(arch: Arch,) -> PoisonGirlB<Self,>
	{
		let path = PathBuf::from_str("/tmp/",).unwrap();
		let ovmf_files = Prebuilt::fetch(Source::LATEST, path,)?;
		let code = ovmf_files.get_file(arch.into(), FileType::Code,);
		let vars = ovmf_files.get_file(arch.into(), FileType::Vars,);
		X(Self { code, vars, },)
	}

	/// Gets the path to the OVMF code file
	///
	/// # Returns
	///
	/// A reference to the path to the OVMF code file
	pub fn code(&self,) -> &PathBuf
	{
		&self.code
	}

	/// Gets the path to the OVMF variables file
	///
	/// # Returns
	///
	/// A reference to the path to the OVMF variables file
	pub fn vars(&self,) -> &PathBuf
	{
		&self.vars
	}
}

impl From<Arch,> for ovmf_prebuilt::Arch
{
	fn from(value: Arch,) -> Self
	{
		match value {
			Arch::Aarch64 => ovmf_prebuilt::Arch::Aarch64,
			Arch::Riscv64 => ovmf_prebuilt::Arch::Riscv64,
		}
	}
}

#[derive(
	Default,
	strum_macros::AsRefStr,
	strum_macros::EnumIs,
	strum_macros::EnumString,
	Clone,
	Copy,
	clap::ValueEnum,
	PartialEq,
	Eq,
	Debug,
	Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum Arch
{
	#[default]
	Aarch64,
	Riscv64,
}

impl Arch
{
	/// Gets the boot file name for the architecture
	///
	/// # Returns
	///
	/// The boot file name (e.g., "bootaa64.efi" for aarch64)
	pub fn boot_file_name(&self,) -> &str
	{
		match self {
			Self::Aarch64 => "bootaa64.efi",
			Self::Riscv64 => "bootriscv64.efi",
		}
	}
}

pub fn host_tuple_by_rustc() -> PoisonGirlB<String,>
{
	let target = Command::new("rustc",).arg("-vV",).output()?.stdout;
	let target = String::from_utf8(target,)?;
	target
		.lines()
		.find_map(|l| {
			if l.contains("host: ",) {
				Some(l.replace("host: ", "",).to_string(),)
			} else {
				None
			}
		},)
		.reshape(poison_girl_err!(HostTupleNotFound),)
}

#[cfg(test)]
mod tests
{
	use super::*;

	/// defaultが効いてるかも確認できるテスト
	#[test]
	fn test_cli_to_opts_with_values()
	{
		let cli = Cli::default();

		let opts = cli.to_opts();
		assert!(opts.build_mode.is_debug());
		assert!(opts.feature_flags.is_empty());
		assert!(opts.arch.is_aarch_64());
		assert!(opts.command.is_run());
		assert!(!opts.lock_deps);
	}
}
