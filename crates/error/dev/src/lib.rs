#![feature(exit_status_error, const_convert, const_trait_impl)]

use std::process::ExitStatusError;

pub use poison_girl_this_is_b_wrapper_dev::{
	B,
	B::{X, Y},
	Container, ReShape,
};

use {
	core::{fmt::Debug, panic::Location},
	std::fmt::Display,
};

/// X/Y はResultの別名ではなく、分岐値 B の左右である
/// `PoisonGirlB<T>` は error-specialized B である
/// no_std/stdをまたぐ統一的な失敗伝播モデルである
pub type PoisonGirlB<T,> = B<T, PoisonGirlError,>;

#[derive(Debug,)]
pub struct PoisonGirlError
{
	loc: &'static Location<'static,>,
	src: DevError,
}

impl std::error::Error for PoisonGirlError
{
}

impl Display for PoisonGirlError
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_,>,) -> std::fmt::Result
	{
		f.write_fmt(format_args!("at: {}\nsrc: {:?}", self.loc, self.src),)
	}
}

const impl From<std::io::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: std::io::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::Io(value,), }
	}
}

impl From<&std::io::Error,> for PoisonGirlError
{
	fn from(value: &std::io::Error,) -> Self
	{
		let value = std::io::Error::new(value.kind(), value.to_string(),);
		Self::from(value,)
	}
}

const impl From<std::process::ExitStatusError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: std::process::ExitStatusError,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::ExitStatus(value,), }
	}
}

const impl From<PathNotFound,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: PathNotFound,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::PathNotFound(value,), }
	}
}

const impl From<std::string::FromUtf8Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: std::string::FromUtf8Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::FromUtf8(value,), }
	}
}

const impl From<toml::de::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: toml::de::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::TomlDeError(value,), }
	}
}

const impl From<HostTupleNotFound,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: HostTupleNotFound,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::HostTupleNotFound(value,),
		}
	}
}

const impl From<ovmf_prebuilt::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: ovmf_prebuilt::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::OvmfError(value,), }
	}
}

const impl From<toml::ser::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: toml::ser::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::TomlSerError(value,), }
	}
}

const impl From<InvalidManifest,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidManifest,) -> Self
	{
		Self {
			loc: Location::caller(), src: DevError::InvalidManifest(value,),
		}
	}
}

const impl From<PathIsNotValidUtf8,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: PathIsNotValidUtf8,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::PathIsNotValidUtf8(value,),
		}
	}
}

const impl From<NotObedientPath,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: NotObedientPath,) -> Self
	{
		Self {
			loc: Location::caller(), src: DevError::NotObedientPath(value,),
		}
	}
}

const impl From<hadris_fat::error::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: hadris_fat::error::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::FatError(value,), }
	}
}

const impl From<ProjectRootNotFound,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: ProjectRootNotFound,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::ProjectRootNotFound(value,),
		}
	}
}

const impl From<InvalidProjectRootFound,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidProjectRootFound,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::InvalidProjectRootFound(value,),
		}
	}
}

const impl From<InvalidCurrentCratePath,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidCurrentCratePath,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::InvalidCurrentCratePath(value,),
		}
	}
}

const impl From<InvalidHostName,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidHostName,) -> Self
	{
		Self {
			loc: Location::caller(), src: DevError::InvalidHostName(value,),
		}
	}
}

const impl From<YourHostPlatformIsOutOfSupport,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: YourHostPlatformIsOutOfSupport,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::YourHostPlatformIsOutOfSupport(value,),
		}
	}
}

const impl From<PointerOperationFailed,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: PointerOperationFailed,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::PointerOperationFailed(value,),
		}
	}
}

const impl From<strum::ParseError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: strum::ParseError,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::StrumError(value,), }
	}
}

const impl From<InvalidMetadataSchema,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidMetadataSchema,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::InvalidMetadataSchema(value,),
		}
	}
}

const impl From<InvalidPolicy,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidPolicy,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::InvalidPolicy(value,), }
	}
}

const impl From<CargoError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: CargoError,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::Cargo(value,), }
	}
}

#[allow(dead_code)]
#[derive(Debug,)]
enum DevError
{
	Cargo(CargoError,),
	Io(std::io::Error,),
	ExitStatus(std::process::ExitStatusError,),
	FromUtf8(std::string::FromUtf8Error,),
	TomlDeError(toml::de::Error,),
	TomlSerError(toml::ser::Error,),
	OvmfError(ovmf_prebuilt::Error,),
	PathNotFound(PathNotFound,),
	HostTupleNotFound(HostTupleNotFound,),
	InvalidManifest(InvalidManifest,),
	PathIsNotValidUtf8(PathIsNotValidUtf8,),
	NotObedientPath(NotObedientPath,),
	FatError(hadris_fat::error::Error,),
	ProjectRootNotFound(ProjectRootNotFound,),
	InvalidProjectRootFound(InvalidProjectRootFound,),
	InvalidCurrentCratePath(InvalidCurrentCratePath,),
	InvalidHostName(InvalidHostName,),
	YourHostPlatformIsOutOfSupport(YourHostPlatformIsOutOfSupport,),
	PointerOperationFailed(PointerOperationFailed,),
	StrumError(strum::ParseError,),
	InvalidMetadataSchema(InvalidMetadataSchema,),
	InvalidPolicy(InvalidPolicy,),
}

#[derive(Debug,)]
pub struct PathNotFound(pub String,);

impl PathNotFound
{
	pub fn new(s: impl Into<String,>,) -> Self
	{
		Self(s.into(),)
	}
}

#[derive(Debug,)]
pub struct HostTupleNotFound;
#[derive(Debug,)]
pub struct InvalidManifest(pub String,);

impl InvalidManifest
{
	pub fn new(s: impl Into<String,>,) -> Self
	{
		Self(s.into(),)
	}
}

#[derive(Debug,)]
pub struct PathIsNotValidUtf8;

#[derive(Debug,)]
pub struct NotObedientPath;

#[derive(Debug,)]
pub struct ProjectRootNotFound;

#[derive(Debug,)]
pub struct InvalidProjectRootFound;

#[derive(Debug,)]
pub struct InvalidCurrentCratePath;

#[derive(Debug,)]
pub struct InvalidHostName(pub String,);

impl InvalidHostName
{
	pub fn new(s: impl Into<String,>,) -> Self
	{
		Self(s.into(),)
	}
}

#[derive(Debug,)]
pub struct YourHostPlatformIsOutOfSupport;

#[derive(Debug,)]
pub struct PointerOperationFailed;

#[derive(Debug,)]
pub struct InvalidMetadataSchema;

#[derive(Debug,)]
pub struct InvalidPolicy;

#[derive(Debug,)]
pub struct CargoError
{
	pub stderr:  String,
	pub context: String,
	pub status:  ExitStatusError,
}

impl CargoError
{
	pub fn new(
		stderr: impl Into<String,>,
		context: impl Into<String,>,
		status: ExitStatusError,
	) -> Self
	{
		Self { stderr: stderr.into(), context: context.into(), status, }
	}
}

#[macro_export]
macro_rules! poison_girl_err {
	($err:expr) => {
		$crate::PoisonGirlError::from($err,)
	};
}
