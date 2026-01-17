#![no_std]

use core::{fmt::Debug, panic::Location};
pub use this_is_b::{
	B::{X, Y},
	Container,
};

pub type PoisonGirlB<T,> = this_is_b::B<T, PoisonGirlError,>;

#[derive(Debug,)]
pub struct PoisonGirlError {
	_loc: &'static Location<'static,>,
	_src: PoisonGirlErrorKind,
}

impl From<ElfParseError,> for PoisonGirlError {
	#[track_caller]
	fn from(value: ElfParseError,) -> Self {
		Self {
			_loc: Location::caller(),
			_src: PoisonGirlErrorKind::ElfParse(value,),
		}
	}
}

impl From<ParserError,> for PoisonGirlError {
	#[track_caller]
	fn from(value: ParserError,) -> Self {
		Self {
			_loc: Location::caller(),
			_src: PoisonGirlErrorKind::Parser(value,),
		}
	}
}

#[derive(Debug,)]
pub enum PoisonGirlErrorKind {
	ElfParse(ElfParseError,),
	Parser(ParserError,),
}

#[derive(Debug,)]
pub enum ElfParseError {
	EndOfBinary {
		parser_pos: &'static str,
		stage:      ElfParseStage,
	},
	SizeOverflow {
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
	TooManySymbolsOffset {
		offset: usize,
		count:  usize,
	},
	InvalidEndianFlag(u8,),
	InvalidProgramHeaderType(u32,),
	InvalidGnuHash {
		buckets_count: usize,
		min_chain:     usize,
		bloom_size:    usize,
	},
}

#[derive(Debug,)]
pub enum ElfParseStage {
	Header,
	ProgramHeader,
	SectionHeader,
	StringTable,
}

#[derive(Debug,)]
pub enum ParserError {}
