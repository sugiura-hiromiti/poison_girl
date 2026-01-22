#![feature(exit_status_error)]

pub use this_is_b::{
	B::{X, Y},
	Container, ReShape,
};
use {
	core::{fmt::Debug, panic::Location},
	std::fmt::Display,
};

pub type PoisonGirlB<T,> = this_is_b::B<T, PoisonGirlError,>;

#[derive(Debug,)]
pub struct PoisonGirlError {
	loc: &'static Location<'static,>,
	src: DevError,
}

impl Display for PoisonGirlError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_,>,) -> std::fmt::Result {
		f.write_fmt(format_args!("at: {}\nsrc: {:?}", self.loc, self.src),)
	}
}

impl From<std::io::Error,> for PoisonGirlError {
	#[track_caller]
	fn from(value: std::io::Error,) -> Self {
		Self { loc: Location::caller(), src: DevError::Io(value,), }
	}
}

impl From<std::process::ExitStatusError,> for PoisonGirlError {
	#[track_caller]
	fn from(value: std::process::ExitStatusError,) -> Self {
		Self { loc: Location::caller(), src: DevError::ExitStatus(value,), }
	}
}

impl From<PathNotFound,> for PoisonGirlError {
	#[track_caller]
	fn from(value: PathNotFound,) -> Self {
		Self { loc: Location::caller(), src: DevError::PathNotFound(value,), }
	}
}

impl From<std::string::FromUtf8Error,> for PoisonGirlError {
	#[track_caller]
	fn from(value: std::string::FromUtf8Error,) -> Self {
		Self { loc: Location::caller(), src: DevError::FromUtf8(value,), }
	}
}

impl From<toml::de::Error,> for PoisonGirlError {
	#[track_caller]
	fn from(value: toml::de::Error,) -> Self {
		Self { loc: Location::caller(), src: DevError::TomlDeError(value,), }
	}
}

impl From<HostTupleNotFound,> for PoisonGirlError {
	#[track_caller]
	fn from(value: HostTupleNotFound,) -> Self {
		Self {
			loc: Location::caller(),
			src: DevError::HostTupleNotFound(value,),
		}
	}
}

impl From<ovmf_prebuilt::Error,> for PoisonGirlError {
	#[track_caller]
	fn from(value: ovmf_prebuilt::Error,) -> Self {
		Self { loc: Location::caller(), src: DevError::OvmfError(value,), }
	}
}

impl From<&str,> for PoisonGirlError {
	#[track_caller]
	fn from(value: &str,) -> Self {
		Self {
			loc: Location::caller(),
			src: DevError::Todo(value.to_string(),),
		}
	}
}

impl From<toml::ser::Error,> for PoisonGirlError {
	#[track_caller]
	fn from(value: toml::ser::Error,) -> Self {
		Self { loc: Location::caller(), src: DevError::TomlSerError(value,), }
	}
}

impl From<InvalidManifest,> for PoisonGirlError {
	#[track_caller]
	fn from(value: InvalidManifest,) -> Self {
		Self {
			loc: Location::caller(), src: DevError::InvalidManifest(value,),
		}
	}
}

#[derive(Debug,)]
pub enum DevError {
	Io(std::io::Error,),
	ExitStatus(std::process::ExitStatusError,),
	FromUtf8(std::string::FromUtf8Error,),
	TomlDeError(toml::de::Error,),
	TomlSerError(toml::ser::Error,),
	OvmfError(ovmf_prebuilt::Error,),
	PathNotFound(PathNotFound,),
	HostTupleNotFound(HostTupleNotFound,),
	InvalidManifest(InvalidManifest,),
	Todo(String,),
}

#[derive(Debug,)]
pub struct PathNotFound(pub String,);

#[derive(Debug,)]
pub struct HostTupleNotFound;
#[derive(Debug,)]
pub struct InvalidManifest(pub String,);

impl InvalidManifest {
	pub fn new(s: impl Into<String,>,) -> Self {
		Self(s.into(),)
	}
}

#[macro_export]
macro_rules! poison_girl_err {
	($err:expr) => {
		$crate::PoisonGirlError::from($err,)
	};
}
