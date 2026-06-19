#![feature(exit_status_error)]

use {
	ovmf_prebuilt::{FileType, Prebuilt, Source},
	poison_girl_dev_error::{
		HostTupleNotFound, PoisonGirlB, ReShape, X, poison_girl_err,
	},
	std::{path::PathBuf, process::Command},
	strum_macros::Display,
};

#[deprecated]
pub trait TargetSpec
{
	fn tuple(&self,) -> String;
	fn arch(&self,) -> Arch;
	fn runtime(&self,) -> Runtime;
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
	pub fn new(arch: Arch,) -> Self
	{
		let firmware = Firmware::new(arch,);
		Self { firmware, host: Runtime::Host, }
	}
}

/// Manages OVMF firmware files for UEFI boot
#[derive(Debug,)]
pub struct Firmware
{
	// /// Path to the OVMF code file
	// pub code: PathBuf,
	// /// Path to the OVMF variables file
	// pub vars: PathBuf,
	arch: Arch,
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
	pub fn new(arch: Arch,) -> Self
	{
		// let path = PathBuf::from("/tmp/",);
		// let ovmf_files = Prebuilt::fetch(Source::LATEST, path,)?;
		// let code = ovmf_files.get_file(arch.into(), FileType::Code,);
		// let vars = ovmf_files.get_file(arch.into(), FileType::Vars,);
		// X(Self { code, vars, },)
		Self { arch, }
	}

	/// Gets the path to the OVMF code file
	///
	/// # Returns
	///
	/// A reference to the path to the OVMF code file
	pub fn code(&self,) -> PoisonGirlB<PathBuf,>
	{
		let path = Self::ovmf_path()?;
		let file_path = path.get_file(self.arch.into(), FileType::Code,);
		X(file_path,)
	}

	/// Gets the path to the OVMF variables file
	///
	/// # Returns
	///
	/// A reference to the path to the OVMF variables file
	pub fn vars(&self,) -> PoisonGirlB<PathBuf,>
	{
		let path = Self::ovmf_path()?;
		let file_path = path.get_file(self.arch.into(), FileType::Vars,);
		X(file_path,)
	}

	fn ovmf_path() -> PoisonGirlB<ovmf_prebuilt::Prebuilt,>
	{
		let path = Prebuilt::fetch(Source::LATEST, PathBuf::from("/tmp/",),)?;
		X(path,)
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
