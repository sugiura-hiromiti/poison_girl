#![no_std]

pub use poison_girl_this_is_b_wrapper_no_std::{
	B::{X, Y},
	Container,
};
use {
	core::{
		fmt::{Debug, Display},
		panic::Location,
	},
	poison_girl_this_is_b_wrapper_no_std::B,
};

/// X/Y はResultの別名ではなく、分岐値 B の左右である
/// PoisonGirlB<T> は error-specialized B である
/// no_std/stdをまたぐ統一的な失敗伝播モデルである
pub type PoisonGirlB<T,> = B<T, PoisonGirlError,>;

#[derive(Debug,)]
pub struct PoisonGirlError
{
	_loc: &'static Location<'static,>,
	_src: PoisonGirlErrorKind,
}

impl core::error::Error for PoisonGirlError
{
}

impl Display for PoisonGirlError
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_,>,) -> core::fmt::Result
	{
		f.write_fmt(format_args!("{self:?}"),)
	}
}

impl From<ElfParseError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: ElfParseError,) -> Self
	{
		Self {
			_loc: Location::caller(),
			_src: PoisonGirlErrorKind::ElfParse(value,),
		}
	}
}

impl From<ParserError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: ParserError,) -> Self
	{
		Self {
			_loc: Location::caller(),
			_src: PoisonGirlErrorKind::Parser(value,),
		}
	}
}

impl From<GraphicError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: GraphicError,) -> Self
	{
		Self {
			_loc: Location::caller(),
			_src: PoisonGirlErrorKind::Graphic(value,),
		}
	}
}

impl From<UefiError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: UefiError,) -> Self
	{
		Self {
			_loc: Location::caller(),
			_src: PoisonGirlErrorKind::Uefi(value,),
		}
	}
}

impl From<GuidError,> for PoisonGirlError
{
	#[track_caller]
	fn from(value: GuidError,) -> Self
	{
		Self {
			_loc: Location::caller(),
			_src: PoisonGirlErrorKind::Uefi(UefiError::Guid(value,),),
		}
	}
}

#[derive(Debug,)]
pub enum PoisonGirlErrorKind
{
	Uefi(UefiError,),
	ElfParse(ElfParseError,),
	Parser(ParserError,),
	Graphic(GraphicError,),
}

#[derive(Debug,)]
pub enum UefiError
{
	CustomStatus(usize,),
	Status(&'static str,),
	Custom(&'static str,),
	Guid(GuidError,),
}

#[derive(Debug,)]
pub enum GuidError
{
	InvalidHexChar,
}

#[derive(Debug,)]
pub enum ElfParseError
{
	EndOfBinary
	{
		parser_pos: &'static str,
		stage:      ElfParseStage,
	},
	SizeOverflow
	{
		stage:    ElfParseStage,
		name:     u64,
		expected: u64,
		base:     u64,
		size:     u64,
	},
	UnknownEfiType(u16,),
	InvalidIdentLen(usize,),
	BadMagicNumber(u8, u8, u8, u8,),
	InvalidFileClass(u8,),
	OsAbiOutOfSupport(u8,),
	/// string context
	DelimiterNotFound(u8,),
	TooManySymbolsOffset
	{
		offset: usize,
		count:  usize,
	},
	InvalidEndianFlag(u8,),
	InvalidProgramHeaderType(u32,),
	InvalidGnuHash
	{
		buckets_count: usize,
		min_chain:     usize,
		bloom_size:    usize,
	},
}

#[derive(Debug,)]
pub enum ElfParseStage
{
	Header,
	ProgramHeader,
	SectionHeader,
	StringTable,
}

#[derive(Debug,)]
pub enum ParserError {}

#[derive(Debug,)]
pub enum GraphicError
{
	InvalidCoordinate,
}

#[macro_export]
macro_rules! poison_girl_err {
	($err:expr) => {
		$crate::PoisonGirlError::from($err,)
	};
}
