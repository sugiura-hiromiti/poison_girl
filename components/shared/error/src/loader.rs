use crate::OsoError;

#[derive(Debug, Default,)]
pub enum EfiParseError {
	EndOfBinary {
		parser_pos: &'static str,
		stage:      EfiParseStage,
	},
	SizeOverflow {
		stage:    EfiParseStage,
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
	#[default]
	Unknown,
}

#[derive(Debug, Default,)]
pub enum EfiParseStage {
	#[default]
	Header,
	ProgramHeader,
	SectionHeader,
	StringTable,
}

#[derive(Debug, Default,)]
pub enum UefiError {
	#[default]
	CustomStatus,
	ErrorStatus(&'static str,),
	Custom(&'static str,),
}

impl From<OsoError<UefiError,>,> for OsoError<(),> {
	fn from(value: OsoError<UefiError,>,) -> Self {
		OsoError { from: value.from, desc: Some((),), }
	}
}
