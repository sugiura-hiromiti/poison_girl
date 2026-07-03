#![feature(exit_status_error)]

pub use poison_girl_this_is_b_wrapper_dev::{
	B::{X, Y},
	Container, ReShape,
};
use {
	core::{fmt::Debug, panic::Location},
	poison_girl_this_is_b_wrapper_dev::B,
	std::fmt::Display,
};

/// X/Y はResultの別名ではなく、分岐値 B の左右である
/// PoisonGirlB<T> は error-specialized B である
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

impl From<std::io::Error,> for PoisonGirlError
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

impl From<std::process::ExitStatusError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: std::process::ExitStatusError,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::ExitStatus(value,), }
	}
}

impl From<PathNotFound,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: PathNotFound,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::PathNotFound(value,), }
	}
}

impl From<std::string::FromUtf8Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: std::string::FromUtf8Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::FromUtf8(value,), }
	}
}

impl From<toml::de::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: toml::de::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::TomlDeError(value,), }
	}
}

impl From<HostTupleNotFound,> for PoisonGirlError
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

impl From<ovmf_prebuilt::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: ovmf_prebuilt::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::OvmfError(value,), }
	}
}

impl From<&str,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: &str,) -> Self
	{
		Self {
			loc: Location::caller(),
			src: DevError::Todo(value.to_string(),),
		}
	}
}

impl From<String,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: String,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::Todo(value,), }
	}
}

impl From<toml::ser::Error,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: toml::ser::Error,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::TomlSerError(value,), }
	}
}

impl From<InvalidManifest,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidManifest,) -> Self
	{
		Self {
			loc: Location::caller(), src: DevError::InvalidManifest(value,),
		}
	}
}

impl From<PathIsNotValidUtf8,> for PoisonGirlError
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

impl From<NotObedientPath,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: NotObedientPath,) -> Self
	{
		Self {
			loc: Location::caller(), src: DevError::NotObedientPath(value,),
		}
	}
}

impl From<hadris_fat::error::FatError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: hadris_fat::error::FatError,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::FatError(value,), }
	}
}

impl From<ProjectRootNotFound,> for PoisonGirlError
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

impl From<InvalidProjectRootFound,> for PoisonGirlError
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

impl From<InvalidCurrentCratePath,> for PoisonGirlError
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

impl From<InvalidHostName,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidHostName,) -> Self
	{
		Self {
			loc: Location::caller(), src: DevError::InvalidHostName(value,),
		}
	}
}

impl From<YourHostPlatformIsOutOfSupport,> for PoisonGirlError
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

impl From<PointerOperationFailed,> for PoisonGirlError
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

impl From<strum::ParseError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: strum::ParseError,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::StrumError(value,), }
	}
}

impl From<InvalidMetadataSchema,> for PoisonGirlError
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

impl From<InvalidPolicy,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: InvalidPolicy,) -> Self
	{
		Self { loc: Location::caller(), src: DevError::InvalidPolicy(value,), }
	}
}

#[allow(dead_code)]
#[derive(Debug,)]
enum DevError
{
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
	FatError(hadris_fat::error::FatError,),
	Todo(String,),
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

#[macro_export]
macro_rules! poison_girl_err {
	($err:expr) => {
		$crate::PoisonGirlError::from($err,)
	};
}
