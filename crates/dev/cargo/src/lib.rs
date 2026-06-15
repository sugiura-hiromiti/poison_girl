#![feature(exit_status_error)]

use {
	clap::Subcommand,
	ovmf_prebuilt::{FileType, Prebuilt, Source},
	poison_girl_dev_error::{
		HostTupleNotFound, PoisonGirlB, ReShape, X, poison_girl_err,
	},
	std::{path::PathBuf, process::Command},
	strum_macros::Display,
};

pub trait TargetSpec
{
	fn tuple(&self,) -> String;
	fn arch(&self,) -> Arch;
	fn runtime(&self,) -> Runtime;
}

#[derive(clap::Parser, Default,)]
#[command(version, about)]
pub struct Cli
{
	#[arg(value_enum, short)]
	pub build_mode:    Option<BuildMode,>,
	#[arg(short)]
	/// this is not Option<Vec<Feature,>,> in order to prevent cyclic
	/// referencing
	pub feature_flags: Option<Vec<String,>,>,
	#[arg(short)]
	pub arch:          Option<Arch,>,
	#[command(subcommand)]
	pub command:       Option<CliCommand,>,
	#[arg(short, default_value_t = false)]
	pub lock_deps:     bool,
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

#[derive(
	Subcommand,
	Default,
	strum_macros::EnumIs,
	strum_macros::AsRefStr,
	Clone,
	Copy,
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
	Fixture,
	/// cargo fix
	Fix,
}

#[derive(Subcommand, strum_macros::EnumIter, Clone, Copy,)]
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
		let path = PathBuf::from("/tmp/",);
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
	let target =
		checked_stdout(Command::new("rustc",).arg("-vV",), "rustc -vV",)?;
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

fn checked_stdout(
	command: &mut Command,
	context: &str,
) -> PoisonGirlB<Vec<u8,>,>
{
	let output = command.output()?;
	if let Err(status,) = output.status.exit_ok() {
		let stderr = String::from_utf8_lossy(&output.stderr,);
		return poison_girl_dev_error::Y(poison_girl_err!(format!(
			"{context} failed with {status}: {stderr}"
		)),);
	}
	X(output.stdout,)
}

#[cfg(test)]
mod tests
{
	use {super::*, poison_girl_dev_error::Y};

	#[test]
	fn checked_stdout_fails_on_non_zero_status()
	{
		let rslt = checked_stdout(
			Command::new("sh",).args(["-c", "printf stderr-msg >&2; exit 7",],),
			"test command",
		);

		assert!(matches!(rslt, Y(_)));
	}
}
