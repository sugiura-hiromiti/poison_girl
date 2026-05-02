use {
	crate::elf::{
		elf_header::ElfHeader,
		hash::{gnu_hash_len, hash_len},
		program_header::ProgramHeader,
		section_header::SectionHeader,
	},
	alloc::{
		string::{String, ToString},
		vec::Vec,
	},
	core::{
		cmp,
		iter::Sum,
		ops::{
			Add, AddAssign, Div, DivAssign, Mul, MulAssign, Shl, Shr, Sub,
			SubAssign,
		},
	},
	poison_girl_no_std_error::{
		Container as _, ElfParseError, ElfParseStage, PoisonGirlB,
		PoisonGirlError, X, Y, poison_girl_err,
	},
	program_header::ProgramHeaderType,
	section_header::{
		SHT_GNU_VERDEF, SHT_GNU_VERNEED, SHT_GNU_VERSYM, SHT_REL, SHT_RELA,
		SHT_SYMTAB, get_string_table,
	},
};

/// defines elf header data structure
pub mod elf_header;
/// main logic of elf parser
pub mod elf_parser;
/// Hash table implementations for symbol lookup
pub mod hash;
/// Program header parsing and types
pub mod program_header;
/// Section header parsing and types
pub mod section_header;

/// ELF magic number signature used to identify ELF files
const ELF_MAGIC_NUMBER: &[u8; ELF_MAGIC_NUMBER_SIZE] = b"\x7fELF";
/// Size of the ELF magic number in bytes
const ELF_MAGIC_NUMBER_SIZE: usize = 4;
/// Size of the ELF identification array in the header
const ELF_IDENT_SIZE: usize = 16;
/// Index of the file class byte in the ELF identification array
const ELF_FILE_CLASS_INDEX: usize = 4;
/// Value indicating a 32-bit ELF object file
const ELF_32_BIT_OBJECT: u8 = 1;
/// Value indicating a 64-bit ELF object file
const ELF_64_BIT_OBJECT: u8 = 2;
/// Index of the data encoding (endianness) byte in the ELF identification array
const ELF_ENDIANNESS_INDEX: usize = 5;
/// Index of the ELF version byte in the identification array
const ELF_VERSION_INDEX: usize = 6;
/// Index of the target OS ABI byte in the identification array
const ELF_OS_ABI_INDEX: usize = 7;
/// Index of the ABI version byte in the identification array
const ELF_ABI_VERSION_INDEX: usize = 8;

pub struct Elf
{
	pub header:                             ElfHeader,
	pub program_headers:                    Vec<ProgramHeader,>,
	pub section_headers:                    Vec<SectionHeader,>,
	pub section_header_string_table:        StringTable,
	pub dynamic_string_table:               StringTable,
	pub dynamic_symbol_table:               SymbolTable,
	pub symbol_table:                       SymbolTable,
	pub string_table_for_symbol_table:      StringTable,
	pub dynamic_info:                       Dynamic,
	pub dynamic_relocation_with_addend:     RelocationSection,
	pub dynamic_relocation:                 RelocationSection,
	pub procedure_linkage_table_relocation: RelocationSection,
	pub section_relocations:                Vec<(usize, RelocationSection,),>,
	pub shared_object_name:                 Option<String,>,
	pub interpreter:                        Interpreter,
	pub libraries:                          Vec<String,>,
	pub runtime_search_path_deprecated:     Vec<String,>,
	pub runtime_search_path:                Vec<String,>,
	pub symbol_version_section:             Option<SymbolVersionSection,>,
	pub version_definition_section:         Option<VersionDefinitionSection,>,
	pub version_needed_section:             Option<VersionNeededSection,>,
	pub is_position_independent_executable: bool,
}

impl Elf
{
	/// Returns whether this ELF file is 64-bit
	///
	/// # Returns
	///
	/// `true` if this is a 64-bit ELF file, `false` if it's 32-bit
	pub fn is_64(&self,) -> bool
	{
		self.header.is_64()
	}

	/// Returns whether this ELF file is a shared library
	///
	/// A file is considered a library if it has the shared object type
	/// but is not a position-independent executable.
	///
	/// # Returns
	///
	/// `true` if this is a shared library, `false` otherwise
	pub fn is_lib(&self,) -> bool
	{
		self.header.is_lib() && !self.is_position_independent_executable
	}

	/// Returns whether this ELF file uses little-endian byte ordering
	///
	/// # Returns
	///
	/// `true` if little-endian, `false` if big-endian
	pub fn is_little_endian(&self,) -> bool
	{
		self.header.is_little_endian()
	}

	/// Returns the entry point address of the ELF file
	///
	/// This is the virtual address where execution should begin
	/// when the program is loaded.
	///
	/// # Returns
	///
	/// The entry point address as a `usize`
	pub fn entry_point_address(&self,) -> usize
	{
		self.header.entry as usize
	}
}

fn read_le_bytes<I: PrimitiveInteger,>(
	offset: &mut usize,
	binary: &[u8],
) -> Option<I,>
where
	for<'a> &'a [u8]: AsInt<I,>,
{
	//let window = &binary[*offset..*offset + size];
	// let val =
	// 	window.iter().enumerate().map(|(i, b,)| Integer::<I,>::cast_int(*b,) <<
	// i * 8,).sum::<I>();
	let size = size_of::<I,>();
	if size + *offset > binary.len() {
		*offset += size;
		return None;
	}

	let val = (&binary[*offset..]).as_int();
	*offset += size;
	Some(val,)
}

#[derive(PartialEq, Eq, Debug, Default,)]
pub enum ElfType
{
	None,
	Relocatable,
	#[default]
	Executable,
	SharedObject,
	Core,
	NumberOfDefined,
	OsSpecificRangeStart,
	OsSpecificRangeEnd,
	ProcessorSpecificRangeStart,
	ProcessorSpecificRangeEnd,
}

impl ElfType
{
	fn is_lib(&self,) -> bool
	{
		*self == Self::SharedObject
	}
}

impl TryFrom<u16,> for ElfType
{
	type Error = PoisonGirlError;

	fn try_from(value: u16,) -> Result<Self, Self::Error,>
	{
		let ty = match value {
			0 => Self::None,
			1 => Self::Relocatable,
			2 => Self::Executable,
			3 => Self::SharedObject,
			4 => Self::Core,
			5 => Self::NumberOfDefined,
			0xfe00 => Self::OsSpecificRangeStart,
			0xfeff => Self::OsSpecificRangeEnd,
			0xff00 => Self::ProcessorSpecificRangeStart,
			0xffff => Self::OsSpecificRangeEnd,
			_ => {
				return Err(poison_girl_err!(ElfParseError::UnknownEfiType(
					value
				)),);
			},
		};
		Ok(ty,)
	}
}

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfHeaderIdent
{
	pub file_class:    FileClass,
	pub endianness:    Endian,
	pub elf_version:   ElfVersion,
	pub target_os_abi: TargetOsAbi,
	pub abi_version:   AbiVersion,
}

impl ElfHeaderIdent
{
	fn new(ident: &[u8],) -> PoisonGirlB<Self,>
	{
		if ident.len() != ELF_IDENT_SIZE {
			return Y(poison_girl_err!(ElfParseError::InvalidIdentLen(
				ident.len()
			)),);
		}

		// check magic number
		// size of elf magic number is 4
		if &ident[0..4] != ELF_MAGIC_NUMBER {
			return Y(poison_girl_err!(ElfParseError::BadMagicNumber(
				ident[0], ident[1], ident[2], ident[3]
			)),);
		}

		let file_class = FileClass::try_from(ident[ELF_FILE_CLASS_INDEX],)?;
		let endianness = Endian::try_from(ident[ELF_ENDIANNESS_INDEX],)?;
		let elf_version = ElfVersion(ident[ELF_VERSION_INDEX],);
		let target_os_abi = TargetOsAbi::try_from(ident[ELF_OS_ABI_INDEX],)?;
		let abi_version = AbiVersion(ident[ELF_ABI_VERSION_INDEX],);

		X(Self {
			file_class,
			endianness,
			elf_version,
			target_os_abi,
			abi_version,
		},)
	}

	fn is_64(&self,) -> bool
	{
		self.file_class.is_64()
	}

	fn is_little_endian(&self,) -> bool
	{
		self.endianness.is_little_endian()
	}
}

pub struct Interpreter(Option<Vec<u8,>,>,);

#[derive(PartialEq, Eq, Debug, Default,)]
pub enum FileClass
{
	Bit32,
	#[default]
	Bit64,
}

impl FileClass
{
	fn is_64(&self,) -> bool
	{
		*self == FileClass::Bit64
	}
}

impl TryFrom<u8,> for FileClass
{
	type Error = PoisonGirlError;

	fn try_from(value: u8,) -> Result<Self, Self::Error,>
	{
		match value {
			ELF_32_BIT_OBJECT => Ok(Self::Bit32,),
			ELF_64_BIT_OBJECT => Ok(Self::Bit64,),
			_ => {
				Err(poison_girl_err!(ElfParseError::InvalidFileClass(value,)),)
			},
		}
	}
}

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfVersion(pub u8,);

impl ElfVersion
{
	pub const ONE: Self = Self(1,);
}

#[non_exhaustive]
#[derive(Debug, Default, PartialEq, Eq,)]
pub enum TargetOsAbi
{
	SysV,
	#[default]
	Arm,
	Standalone,
}

impl TryFrom<u8,> for TargetOsAbi
{
	type Error = PoisonGirlError;

	fn try_from(value: u8,) -> Result<Self, Self::Error,>
	{
		match value {
			0x0 => Ok(Self::SysV,),
			0x53 => Ok(Self::Arm,),
			0x61 => Ok(Self::Standalone,),
			_ => {
				Err(poison_girl_err!(ElfParseError::OsAbiOutOfSupport(value,)),)
			},
		}
	}
}

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct AbiVersion(pub u8,);
impl AbiVersion
{
	pub const ONE: Self = Self(0,);
}

#[derive_const(Default)]
pub struct StringTable
{
	pub delimitor: StringContext,
	pub bytes:     Vec<u8,>,
	pub strings:   Vec<(usize, String,),>,
}

impl StringTable
{
	/// # Params
	///
	/// - bytes
	///
	/// bytes expected to be entire elf file
	pub fn parse(
		binary: &[u8],
		offset: usize,
		len: usize,
		delimiter: u8,
	) -> PoisonGirlB<Self,>
	{
		let (end, overflow,) = offset.overflowing_add(len,);
		if overflow || end > binary.len() {
			return Y(poison_girl_err!(ElfParseError::SizeOverflow {
				stage:    ElfParseStage::StringTable,
				name:     0,
				expected: binary.len() as u64,
				base:     offset as u64,
				size:     len as u64,
			}),);
		}

		let mut rslt =
			Self::from_slice(&binary[offset..offset + len], delimiter,);
		let mut i = 0;
		while i < rslt.bytes.len() {
			let s = rslt.delimitor.read_bytes(&binary[i..],)?.to_vec();
			let s = String::from_utf8_lossy_owned(s,);
			let len = s.len();
			rslt.strings.push((i, s,),);
			i += len + 1;
		}

		X(rslt,)
	}

	fn from_slice(bytes: &[u8], delimiter: u8,) -> Self
	{
		Self {
			delimitor: StringContext::Delimiter(delimiter,),
			bytes:     bytes.to_vec(),
			strings:   alloc::vec![],
		}
	}

	fn get_at(&self, offset: usize,) -> Option<String,>
	{
		match self.strings.binary_search_by_key(&offset, |(key, _value,)| *key,)
		{
			Ok(index,) => Some(self.strings[index].1.clone(),),
			Err(index,) => {
				if index == 0 {
					return None;
				}
				let (string_begin_offset, entire_string,) =
					&self.strings[index - 1];
				entire_string
					.get(offset - string_begin_offset..,)
					.map(|s| s.to_string(),)
			},
		}
	}
}

pub enum StringContext
{
	Delimiter(u8,),
	DelimiterUntil(u8, usize,),
	Length(usize,),
}

impl StringContext
{
	fn read_bytes<'a,>(&self, bytes: &'a [u8],) -> PoisonGirlB<&'a [u8],>
	{
		let bytes = match self {
			StringContext::Delimiter(delimiter,) => {
				let mut i = 0;
				while let a = &bytes[i..=i]
					&& a != [*delimiter,]
				{
					i += 1;
					if i >= bytes.len() {
						return Y(poison_girl_err!(
							ElfParseError::DelimiterNotFound(*delimiter)
						),);
					}
				}

				&bytes[..i]
			},
			StringContext::DelimiterUntil(..,) => todo!(),
			StringContext::Length(l,) => &bytes[..*l],
		};

		X(bytes,)
	}
}

impl const Default for StringContext
{
	fn default() -> Self
	{
		// null delimiter
		Self::Delimiter(0,)
	}
}

#[derive_const(Default)]
pub struct SymbolTable
{
	pub bytes: Vec<u8,>,
	pub count: usize,
	pub ctx:   Context,
	pub start: usize,
	pub end:   usize,
}

impl SymbolTable
{
	/// size of symbol structure in 64bit.
	const SIZE_OF_SYMBOL_64: usize = 4 + 1 + 1 + 2 + 8 + 8;

	fn parse(
		binary: &[u8],
		offset: usize,
		count: usize,
		context: &Context,
	) -> PoisonGirlB<Self,>
	{
		let size = count
			.checked_mul(match context.container {
				Container::Little => todo!(),
				Container::Big => Self::SIZE_OF_SYMBOL_64,
			},)
			.ok_or(poison_girl_err!(ElfParseError::TooManySymbolsOffset {
				offset,
				count
			}),)?;

		let bytes = binary[offset..offset + size].to_vec();

		X(SymbolTable {
			bytes,
			count,
			ctx: context.clone(),
			start: offset,
			end: offset + size,
		},)
	}
}

#[derive(Clone,)]
#[derive_const(Default)]
pub struct Context
{
	pub container: Container,
	pub le:        Endian,
}

/// the size of a binary container
#[derive(PartialEq, Eq, Clone,)]
pub enum Container
{
	Little,
	Big,
}

impl const Default for Container
{
	fn default() -> Self
	{
		Self::Big
	}
}

#[derive(Debug, PartialEq, Eq, Clone,)]
pub enum Endian
{
	Little,
	Big,
}

impl const Default for Endian
{
	fn default() -> Self
	{
		Self::Big
	}
}

impl Endian
{
	fn is_little_endian(&self,) -> bool
	{
		*self == Self::Little
	}
}

impl TryFrom<u8,> for Endian
{
	type Error = PoisonGirlError;

	fn try_from(value: u8,) -> Result<Self, Self::Error,>
	{
		match value {
			1 => Ok(Self::Little,),
			2 => Ok(Self::Big,),
			_ => {
				Err(poison_girl_err!(ElfParseError::InvalidEndianFlag(value,)),)
			},
		}
	}
}

struct DynamicInner
{
	pub dyns: Vec<Dyn,>,
	pub info: DynamicInfo,
}

pub struct Dynamic(Option<DynamicInner,>,);

impl AsRef<Option<DynamicInner,>,> for Dynamic
{
	fn as_ref(&self,) -> &Option<DynamicInner,>
	{
		&self.0
	}
}

impl Dynamic
{
	/// No lazy binding for this object.
	pub const DF_BIND_NOW: u64 = 0x0000_0008;
	/// Configuration alternative created.
	pub const DF_EXTEND_CONFALT: u64 = 0x0000_2000;
	/// Direct binding enabled.
	pub const DF_EXTEND_DIRECT: u64 = 0x0000_0100;
	/// Disp reloc applied at build time.
	pub const DF_EXTEND_DISPRELDNE: u64 = 0x0000_8000;
	/// Disp reloc applied at run-time.
	pub const DF_EXTEND_DISPRELPND: u64 = 0x0001_0000;
	/// Object is modified after built.
	pub const DF_EXTEND_EDITED: u64 = 0x0020_0000;
	/// Filtee terminates filters search.
	pub const DF_EXTEND_ENDFILTEE: u64 = 0x0000_4000;
	/// Set RTLD_GLOBAL for this object.
	pub const DF_EXTEND_GLOBAL: u64 = 0x0000_0002;
	/// Global auditing required.
	pub const DF_EXTEND_GLOBAUDIT: u64 = 0x0100_0000;
	/// Set RTLD_GROUP for this object.
	pub const DF_EXTEND_GROUP: u64 = 0x0000_0004;
	pub const DF_EXTEND_IGNMULDEF: u64 = 0x0004_0000;
	/// Set RTLD_INITFIRST for this object.
	pub const DF_EXTEND_INITFIRST: u64 = 0x0000_0020;
	/// Object is used to interpose.
	pub const DF_EXTEND_INTERPOSE: u64 = 0x0000_0400;
	/// Trigger filtee loading at runtime.
	pub const DF_EXTEND_LOADFLTR: u64 = 0x0000_0010;
	/// Ignore default lib search path.
	pub const DF_EXTEND_NODEFLIB: u64 = 0x0000_0800;
	/// Set RTLD_NODELETE for this object.
	pub const DF_EXTEND_NODELETE: u64 = 0x0000_0008;
	/// Object has no-direct binding.
	pub const DF_EXTEND_NODIRECT: u64 = 0x0002_0000;
	/// Object can't be dldump'ed.
	pub const DF_EXTEND_NODUMP: u64 = 0x0000_1000;
	pub const DF_EXTEND_NOHDR: u64 = 0x0010_0000;
	pub const DF_EXTEND_NOKSYMS: u64 = 0x0008_0000;
	/// Set RTLD_NOOPEN for this object.
	pub const DF_EXTEND_NOOPEN: u64 = 0x0000_0040;
	pub const DF_EXTEND_NORELOC: u64 = 0x0040_0000;
	/// === State flags ===
	/// selectable in the `d_un.d_val` element of the DT_FLAGS_1 entry in the
	/// dynamic section.
	///
	/// Set RTLD_NOW for this object.
	pub const DF_EXTEND_NOW: u64 = 0x0000_0001;
	/// $ORIGIN must be handled.
	pub const DF_EXTEND_ORIGIN: u64 = 0x0000_0080;
	/// Object is a Position Independent Executable (PIE).
	pub const DF_EXTEND_PIE: u64 = 0x0800_0000;
	/// Singleton dyn are used.
	pub const DF_EXTEND_SINGLETON: u64 = 0x0200_0000;
	/// Object has individual interposers.
	pub const DF_EXTEND_SYMINTPOSE: u64 = 0x0080_0000;
	pub const DF_EXTEND_TRANS: u64 = 0x0000_0200;
	// Values of `d_un.d_val` in the DT_FLAGS entry
	/// Object may use DF_ORIGIN.
	pub const DF_ORIGIN: u64 = 0x0000_0001;
	/// Module uses the static TLS model.
	pub const DF_STATIC_TLS: u64 = 0x0000_0010;
	/// Symbol resolutions starts here.
	pub const DF_SYMBOLIC: u64 = 0x0000_0002;
	/// Object contains text relocations.
	pub const DF_TEXTREL: u64 = 0x0000_0004;
	//DT_ADDRTAGIDX(tag)	(DT_ADDRRNGHI - (tag))	/* Reverse order! */
	pub const DT_ADDRNUM: u64 = 11;
	/// --
	pub const DT_ADDRRNGHI: u64 = 0x6fff_feff;
	/// DT_* entries which fall between DT_ADDRRNGHI & DT_ADDRRNGLO use the
	/// Dyn.d_un.d_ptr field of the Elf*_Dyn structure.
	///
	/// If any adjustment is made to the ELF object after it has been
	/// built these entries will need to be adjusted.
	pub const DT_ADDRRNGLO: u64 = 0x6fff_fe00;
	/// Object auditing
	pub const DT_AUDIT: u64 = 0x6fff_fefc;
	/// Process relocations of object
	pub const DT_BIND_NOW: u64 = 24;
	/// Configuration information
	pub const DT_CONFIG: u64 = 0x6fff_fefa;
	/// For debugging; unspecified
	pub const DT_DEBUG: u64 = 21;
	/// Dependency auditing
	pub const DT_DEPAUDIT: u64 = 0x6fff_fefb;
	/// Start of encoded range
	pub const DT_ENCODING: u64 = 32;
	/// Address of termination function
	pub const DT_FINI: u64 = 13;
	/// Array with addresses of fini fct
	pub const DT_FINI_ARRAY: u64 = 26;
	/// Size in bytes of DT_FINI_ARRAY
	pub const DT_FINI_ARRAYSZ: u64 = 28;
	/// Flags for the object being loaded
	pub const DT_FLAGS: u64 = 30;
	/// State flags, see DF_1_* below
	pub const DT_FLAGS_1: u64 = 0x6fff_fffb;
	/// Start of conflict section
	pub const DT_GNU_CONFLICT: u64 = 0x6fff_fef8;
	/// GNU-style hash table
	pub const DT_GNU_HASH: u64 = 0x6fff_fef5;
	/// Library list
	pub const DT_GNU_LIBLIST: u64 = 0x6fff_fef9;
	/// Address of symbol hash table
	pub const DT_HASH: u64 = 4;
	/// End of OS-specific
	pub const DT_HIOS: u64 = 0x6fff_f000;
	/// End of processor-specific
	pub const DT_HIPROC: u64 = 0x7fff_ffff;
	/// Address of init function
	pub const DT_INIT: u64 = 12;
	/// Array with addresses of init fct
	pub const DT_INIT_ARRAY: u64 = 25;
	/// Size in bytes of DT_INIT_ARRAY
	pub const DT_INIT_ARRAYSZ: u64 = 27;
	/// Address of PLT relocs
	pub const DT_JMPREL: u64 = 23;
	/// Start of OS-specific
	pub const DT_LOOS: u64 = 0x6000_000d;
	/// Start of processor-specific
	pub const DT_LOPROC: u64 = 0x7000_0000;
	/// Move table
	pub const DT_MOVETAB: u64 = 0x6fff_fefe;
	/// Name of needed library
	pub const DT_NEEDED: u64 = 1;
	/// Marks end of dynamic section
	pub const DT_NULL: u64 = 0;
	/// Number used
	pub const DT_NUM: u64 = 34;
	/// Processor defined value
	pub const DT_PLTGOT: u64 = 3;
	/// PLT padding
	pub const DT_PLTPAD: u64 = 0x6fff_fefd;
	/// Type of reloc in PLT
	pub const DT_PLTREL: u64 = 20;
	/// Size in bytes of PLT relocs
	pub const DT_PLTRELSZ: u64 = 2;
	/// Array with addresses of preinit fct
	pub const DT_PREINIT_ARRAY: u64 = 32;
	/// size in bytes of DT_PREINIT_ARRAY
	pub const DT_PREINIT_ARRAYSZ: u64 = 33;
	/// Address of Rel relocs
	pub const DT_REL: u64 = 17;
	/// Address of Rela relocs
	pub const DT_RELA: u64 = 7;
	pub const DT_RELACOUNT: u64 = 0x6fff_fff9;
	/// Size of one Rela reloc
	pub const DT_RELAENT: u64 = 9;
	/// Total size of Rela relocs
	pub const DT_RELASZ: u64 = 8;
	pub const DT_RELCOUNT: u64 = 0x6fff_fffa;
	/// Size of one Rel reloc
	pub const DT_RELENT: u64 = 19;
	/// Total size of Rel relocs
	pub const DT_RELSZ: u64 = 18;
	/// Library search path (deprecated)
	pub const DT_RPATH: u64 = 15;
	/// Library search path
	pub const DT_RUNPATH: u64 = 29;
	/// Name of shared object
	pub const DT_SONAME: u64 = 14;
	/// Size of string table
	pub const DT_STRSZ: u64 = 10;
	/// Address of string table
	pub const DT_STRTAB: u64 = 5;
	/// Start symbol search here
	pub const DT_SYMBOLIC: u64 = 16;
	/// Size of one symbol table entry
	pub const DT_SYMENT: u64 = 11;
	/// Syminfo table
	pub const DT_SYMINFO: u64 = 0x6fff_feff;
	/// Address of symbol table
	pub const DT_SYMTAB: u64 = 6;
	/// Reloc might modify .text
	pub const DT_TEXTREL: u64 = 22;
	/// --
	pub const DT_TLSDESC_GOT: u64 = 0x6fff_fef7;
	/// --
	pub const DT_TLSDESC_PLT: u64 = 0x6fff_fef6;
	/// Address of version definition table
	pub const DT_VERDEF: u64 = 0x6fff_fffc;
	/// Number of version definitions
	pub const DT_VERDEFNUM: u64 = 0x6fff_fffd;
	/// Address of table with needed versions
	pub const DT_VERNEED: u64 = 0x6fff_fffe;
	/// Number of needed versions
	pub const DT_VERNEEDNUM: u64 = 0x6fff_ffff;
	/// The versioning entry types. The next are defined as part of the GNU
	/// extension
	pub const DT_VERSYM: u64 = 0x6fff_fff0;
	/// デフォルト値
	const DYNAMIC_RELOCATION: RelocationSection = RelocationSection::default();
	/// デフォルト値
	const DYNAMIC_RELOCATION_WITH_ADDEND: RelocationSection =
		RelocationSection::default();
	/// デフォルト値
	const DYNAMIC_STRING_TABLE: StringTable = StringTable::default();
	/// デフォルト値
	const DYNAMIC_SYMBOL_TABLE: SymbolTable = SymbolTable::default();
	/// デフォルト値
	const IS_POSITION_INDEPENDENT_EXECUTABLE: bool = false;
	/// デフォルト値
	const LIBRARIES: Vec<String,> = alloc::vec![];
	/// デフォルト値
	const PROCEDURE_LINKAGE_TABLE_RELOCATION: RelocationSection =
		RelocationSection::default();
	/// デフォルト値
	const RUNTIME_SEARCH_PATH: Vec<String,> = alloc::vec![];
	/// デフォルト値
	const RUNTIME_SEARCH_PATH_DEPRECATED: Vec<String,> = alloc::vec![];
	/// デフォルト値
	const SHARED_OBJECT_NAME: Option<String,> = None;

	fn parse(
		binary: &[u8],
		program_headers: &Vec<ProgramHeader,>,
	) -> PoisonGirlB<Self,>
	{
		for program_header in program_headers {
			if program_header.ty == ProgramHeaderType::Dynamic {
				let offset = program_header.offset as usize;
				let file_size = program_header.file_size as usize;
				let bytes = if file_size > 0 {
					&binary[offset..offset + file_size]
				} else {
					&[]
				};
				let size = Dyn::size_of(&Context {
					container: Container::Big,
					..Default::default()
				},);
				let count = file_size / size;
				let mut dyns = Vec::with_capacity(count,);
				let offset = &mut 0;
				for _ in 0..count {
					let dynamic = Dyn::parse(bytes, offset,);
					let tag = dynamic.tag;
					dyns.push(dynamic,);
					if tag == Self::DT_NULL {
						break;
					}
				}

				let mut info = DynamicInfo::default();
				for dynamic in &dyns {
					info.update(program_headers, dynamic,);
				}

				return X(Dynamic(Some(DynamicInner { dyns, info, },),),);
			}
		}

		X(Dynamic(None,),)
	}

	fn get_libraries(&self, string_table: &StringTable,) -> Vec<String,>
	{
		let Some(ref inner,) = self.0 else {
			return Vec::new();
		};

		let count =
			inner.dyns.len().min(inner.info.version_need_count as usize,);
		let mut needed = Vec::with_capacity(count,);
		for dynamic in &inner.dyns {
			if dynamic.tag == Self::DT_NEEDED
				&& let Some(lib,) = string_table.get_at(dynamic.val as usize,)
			{
				needed.push(lib,);
			}
		}
		needed
	}

	fn is_position_independent_executable(&self,) -> bool
	{
		let Some(ref inner,) = self.0 else {
			return Self::IS_POSITION_INDEPENDENT_EXECUTABLE;
		};

		inner.info.extended_flags & Self::DF_EXTEND_PIE != 0
	}

	fn dynamic_string_table(&self, binary: &[u8],)
	-> PoisonGirlB<StringTable,>
	{
		let Some(ref inner,) = self.0 else {
			return X(Self::DYNAMIC_STRING_TABLE,);
		};

		let info = &inner.info;
		StringTable::parse(
			binary,
			info.string_table_address,
			info.string_table_size,
			0x0,
		)
	}

	fn shared_object_name(
		&self, dyn_str_table: &StringTable,
	) -> Option<String,>
	{
		if let Some(ref inner,) = self.0
			&& inner.info.shared_object_name_offset != 0
		{
			dyn_str_table.get_at(inner.info.shared_object_name_offset,)
		} else {
			Self::SHARED_OBJECT_NAME
		}
	}

	fn libraries(&self, dyn_str_table: &StringTable,) -> Vec<String,>
	{
		if let Some(ref inner,) = self.0
			&& inner.info.version_need_count > 0
		{
			self.get_libraries(dyn_str_table,)
		} else {
			Self::LIBRARIES
		}
	}

	fn runtime_search_path_detection(
		&self,
		dyn_str_table: &StringTable,
		tag: u64,
	) -> Vec<String,>
	{
		let Some(ref inner,) = self.0 else {
			return Self::RUNTIME_SEARCH_PATH_DEPRECATED;
		};

		inner
			.dyns
			.iter()
			.filter_map(|dynamic| {
				if dynamic.tag == tag
					&& let Some(path,) =
						dyn_str_table.get_at(dynamic.val as usize,)
				{
					Some(path,)
				} else {
					None
				}
			},)
			.collect()
	}

	fn runtime_search_path_deprecated(
		&self,
		dyn_str_table: &StringTable,
	) -> Vec<String,>
	{
		self.runtime_search_path_detection(dyn_str_table, Self::DT_RPATH,)
	}

	fn runtime_search_path(&self, dyn_str_table: &StringTable,)
	-> Vec<String,>
	{
		self.runtime_search_path_detection(dyn_str_table, Self::DT_RUNPATH,)
	}

	/// # Return
	/// (dynamic_relocation_with_addend, dynamic_relocation,
	/// procedure_linkage_table_relocation)
	fn dynamic_relocations(
		&self,
		ctx: &Context,
		binary: &[u8],
	) -> PoisonGirlB<(RelocationSection, RelocationSection, RelocationSection,),>
	{
		let Some(ref inner,) = self.0 else {
			return X((
				Self::DYNAMIC_RELOCATION_WITH_ADDEND,
				Self::DYNAMIC_RELOCATION,
				Self::PROCEDURE_LINKAGE_TABLE_RELOCATION,
			),);
		};

		let dynamic_relocation_with_addend = RelocationSection::parse(
			binary,
			inner.info.relocation_addend,
			inner.info.relocation_addend_size,
			true,
			ctx,
		)?;
		let dynamic_relocation = RelocationSection::parse(
			binary,
			inner.info.relocation,
			inner.info.relocation_size,
			false,
			ctx,
		)?;
		let is_relocation_addrend =
			inner.info.plt_relocation_type == Self::DT_RELA;
		let procedure_linkage_table_relocation = RelocationSection::parse(
			binary,
			inner.info.jmp_relocation_address,
			inner.info.plt_relocation_size,
			is_relocation_addrend,
			ctx,
		)?;

		X((
			dynamic_relocation_with_addend,
			dynamic_relocation,
			procedure_linkage_table_relocation,
		),)
	}

	fn dynamic_relocations_and_symbol_table(
		&self,
		ctx: &Context,
		binary: &[u8],
		machine: u16,
	) -> PoisonGirlB<(
		RelocationSection,
		RelocationSection,
		RelocationSection,
		SymbolTable,
	),>
	{
		let Some(ref inner,) = self.0 else {
			return X((
				Self::DYNAMIC_RELOCATION_WITH_ADDEND,
				Self::DYNAMIC_RELOCATION,
				Self::PROCEDURE_LINKAGE_TABLE_RELOCATION,
				Self::DYNAMIC_SYMBOL_TABLE,
			),);
		};

		let (
			dynamic_relocation_with_addend,
			dynamic_relocation,
			procedure_linkage_table_relocation,
		) = self.dynamic_relocations(ctx, binary,)?;

		let mut symbols_count = if let Some(gnu_hash,) = inner.info.gnu_hash {
			gnu_hash_len(binary, gnu_hash as usize, ctx,)?
		} else if let Some(hash,) = inner.info.hash {
			hash_len(binary, hash as usize, machine, ctx,)?
		} else {
			0
		};

		let max_relocation_symbol = dynamic_relocation_with_addend
			.iter()
			.chain(dynamic_relocation.iter(),)
			.chain(procedure_linkage_table_relocation.iter(),)
			.fold(0, |count, relocation| {
				cmp::max(count, relocation.symbol_index,)
			},);

		if max_relocation_symbol != 0 {
			symbols_count = cmp::max(symbols_count, max_relocation_symbol + 1,);
		}

		let dynamic_symbol_table = SymbolTable::parse(
			binary,
			inner.info.symbol_table,
			symbols_count,
			ctx,
		)?;

		X((
			dynamic_relocation_with_addend,
			dynamic_relocation,
			procedure_linkage_table_relocation,
			dynamic_symbol_table,
		),)
	}
}

pub struct Dyn
{
	pub tag: u64,
	pub val: u64,
}

impl Dyn
{
	const SIZE_OF_DYN_32: usize = 8;
	const SIZE_OF_DYN_64: usize = 16;

	fn size_of(Context { container, .. }: &Context,) -> usize
	{
		match container {
			Container::Little => Self::SIZE_OF_DYN_32,
			Container::Big => Self::SIZE_OF_DYN_64,
		}
	}

	fn parse(bytes: &[u8], offset: &mut usize,) -> Self
	{
		let tag = read_le_bytes(offset, bytes,).unwrap();
		let val = read_le_bytes(offset, bytes,).unwrap();
		Self { tag, val, }
	}
}

#[derive(Default,)]
pub struct DynamicInfo
{
	/// An addend is an extra constant value used in a relocation to help
	/// compute the correct final address. It adjusts the value that gets
	/// written into the relocated memory.
	pub relocation_addend:                usize,
	pub relocation_addend_size:           usize,
	pub relocation_addend_entry:          u64,
	pub relocation_addend_entry_count:    usize,
	pub relocation:                       usize,
	pub relocation_size:                  usize,
	pub relocation_entry:                 u64,
	pub relocation_entry_count:           usize,
	pub gnu_hash:                         Option<u64,>,
	pub hash:                             Option<u64,>,
	pub string_table_address:             usize,
	pub string_table_size:                usize,
	pub symbol_table:                     usize,
	pub symbol_table_entry:               usize,
	pub plt_got_address:                  Option<u64,>,
	pub plt_relocation_size:              usize,
	pub plt_relocation_type:              u64,
	pub jmp_relocation_address:           usize,
	pub virsion_definition_table_address: u64,
	pub version_definition_count:         u64,
	pub version_need_table_address:       u64,
	pub version_need_count:               u64,
	pub version_symbol_table_address:     u64,
	pub init_fn_address:                  u64,
	pub finalization_fn_address:          u64,
	pub init_fn_array_address:            u64,
	pub init_fn_array_len:                usize,
	pub finalization_fn_array_address:    u64,
	pub finalization_fn_array_len:        usize,
	pub required_shared_lib_count:        usize,
	pub flags:                            u64,
	pub extended_flags:                   u64,
	pub shared_object_name_offset:        usize,
	pub text_section_relocation:          bool,
}

impl DynamicInfo
{
	pub fn update(&mut self, phdrs: &[ProgramHeader], dynamic: &Dyn,)
	{
		match dynamic.tag {
			Dynamic::DT_RELA => {
				self.relocation_addend =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,) as usize
			}, // .rela.dyn
			Dynamic::DT_RELASZ => {
				self.relocation_addend_size = dynamic.val as usize
			},
			Dynamic::DT_RELAENT => self.relocation_addend_entry = dynamic.val,
			Dynamic::DT_RELACOUNT => {
				self.relocation_addend_entry_count = dynamic.val as usize
			},
			Dynamic::DT_REL => {
				self.relocation =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,) as usize
			}, /* .rel.dyn */
			Dynamic::DT_RELSZ => self.relocation_size = dynamic.val as usize,
			Dynamic::DT_RELENT => self.relocation_entry = dynamic.val,
			Dynamic::DT_RELCOUNT => {
				self.relocation_entry_count = dynamic.val as usize
			},
			Dynamic::DT_GNU_HASH => {
				self.gnu_hash = vm_to_offset(phdrs, dynamic.val,)
			},
			Dynamic::DT_HASH => self.hash = vm_to_offset(phdrs, dynamic.val,),
			Dynamic::DT_STRTAB => {
				self.string_table_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,) as usize
			},
			Dynamic::DT_STRSZ => self.string_table_size = dynamic.val as usize,
			Dynamic::DT_SYMTAB => {
				self.symbol_table =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,) as usize
			},
			Dynamic::DT_SYMENT => {
				self.symbol_table_entry = dynamic.val as usize
			},
			Dynamic::DT_PLTGOT => {
				self.plt_got_address = vm_to_offset(phdrs, dynamic.val,)
			},
			Dynamic::DT_PLTRELSZ => {
				self.plt_relocation_size = dynamic.val as usize
			},
			Dynamic::DT_PLTREL => self.plt_relocation_type = dynamic.val,
			Dynamic::DT_JMPREL => {
				self.jmp_relocation_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,) as usize
			}, /* .rela.plt */
			Dynamic::DT_VERDEF => {
				self.version_definition_count =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_VERDEFNUM => {
				self.version_definition_count =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_VERNEED => {
				self.version_need_table_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_VERNEEDNUM => self.version_need_count = dynamic.val,
			Dynamic::DT_VERSYM => {
				self.version_symbol_table_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_INIT => {
				self.init_fn_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_FINI => {
				self.finalization_fn_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_INIT_ARRAY => {
				self.init_fn_array_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_INIT_ARRAYSZ => {
				self.init_fn_array_len = dynamic.val as usize
			},
			Dynamic::DT_FINI_ARRAY => {
				self.finalization_fn_array_address =
					vm_to_offset(phdrs, dynamic.val,).unwrap_or(0,)
			},
			Dynamic::DT_FINI_ARRAYSZ => {
				self.finalization_fn_array_len = dynamic.val as usize
			},
			Dynamic::DT_NEEDED => self.version_need_count += 1,
			Dynamic::DT_FLAGS => self.flags = dynamic.val,
			Dynamic::DT_FLAGS_1 => self.extended_flags = dynamic.val,
			Dynamic::DT_SONAME => {
				self.shared_object_name_offset = dynamic.val as usize
			},
			Dynamic::DT_TEXTREL => self.text_section_relocation = true,
			_ => (),
		}
	}
}

fn vm_to_offset(
	program_headers: &[ProgramHeader], address: u64,
) -> Option<u64,>
{
	for program_header in program_headers {
		if program_header.ty == ProgramHeaderType::Load
			&& address >= program_header.virtual_address
		{
			let offset = address - program_header.virtual_address;
			if offset < program_header.memory_size {
				return program_header.offset.checked_add(offset,);
			}
		}
	}
	None
}

#[derive_const(Default)]
pub struct RelocationSection
{
	pub bytes:   Vec<u8,>,
	pub count:   usize,
	pub context: RelocationContext,
	pub start:   usize,
	pub end:     usize,
}

impl RelocationSection
{
	const SIZE_OF_RELOCATION_32: usize = 8;
	const SIZE_OF_RELOCATION_64: usize = 16;
	const SIZE_OF_RELOCATION_ADDEND_32: usize = 12;
	const SIZE_OF_RELOCATION_ADDEND_64: usize = 24;

	fn parse(
		binary: &[u8],
		offset: usize,
		size: usize,
		is_addend: bool,
		ctx: &Context,
	) -> PoisonGirlB<Self,>
	{
		let bytes =
			if size != 0 { &binary[offset..offset + size] } else { &[] }
				.to_vec();

		X(Self {
			bytes,
			count: size / Self::size(is_addend, ctx,),
			context: RelocationContext(is_addend, ctx.clone(),),
			start: offset,
			end: offset + size,
		},)
	}

	fn size(
		is_relocation_addrend: bool,
		Context { container, .. }: &Context,
	) -> usize
	{
		match (is_relocation_addrend, container,) {
			(true, Container::Little,) => Self::SIZE_OF_RELOCATION_ADDEND_32,
			(true, Container::Big,) => Self::SIZE_OF_RELOCATION_ADDEND_64,
			(false, Container::Little,) => Self::SIZE_OF_RELOCATION_32,
			(false, Container::Big,) => Self::SIZE_OF_RELOCATION_64,
		}
	}

	fn iter(&self,) -> RelocationIterator
	{
		self.into_iter()
	}
}

impl IntoIterator for &RelocationSection
{
	type IntoIter = RelocationIterator;
	type Item = <RelocationIterator as Iterator>::Item;

	fn into_iter(self,) -> Self::IntoIter
	{
		todo!()
	}
}

pub struct RelocationIterator
{
	bytes:   Vec<u8,>,
	offset:  usize,
	index:   usize,
	count:   usize,
	context: RelocationContext,
}

impl Iterator for RelocationIterator
{
	type Item = Relocation;

	fn next(&mut self,) -> Option<Self::Item,>
	{
		if self.index >= self.count {
			None
		} else {
			self.index += 1;
			Some(Relocation::parse(&self.bytes, &mut self.offset, &self.context,).unwrap(),)
		}
	}
}

#[derive_const(Default)]
pub struct RelocationContext(bool, Context,);

pub struct Relocation
{
	/// address
	pub offset:       u64,
	/// addend
	pub addend:       Option<i64,>,
	/// the index into the corresponding symbol table - either dynamic or
	/// regular
	pub symbol_index: usize,
	/// the relocation type
	pub ty:           u32,
}

impl Relocation
{
	fn parse(
		bytes: &[u8],
		offset: &mut usize,
		RelocationContext(is_relocation_addrend, context,): &RelocationContext,
	) -> PoisonGirlB<Self,>
	{
		let relocation = match (is_relocation_addrend, &context.container,) {
			(true, Container::Little,) => todo!(),
			(true, Container::Big,) => {
				RelocAddend::parse(bytes, offset,).into()
			},
			(false, Container::Little,) => todo!(),
			(false, Container::Big,) => Reloc::parse(bytes, offset,).into(),
		};
		X(relocation,)
	}
}

pub struct RelocAddend
{
	pub offset: u64,
	pub info:   u64,
	pub addend: i64,
}

impl RelocAddend
{
	fn parse(binary: &[u8], offset: &mut usize,) -> Self
	{
		let reloc_offset: u64 = read_le_bytes(offset, binary,).unwrap();
		let info: u64 = read_le_bytes(offset, binary,).unwrap();
		let addend: i64 = read_le_bytes(offset, binary,).unwrap();
		Self { offset: reloc_offset, info, addend, }
	}
}

impl From<RelocAddend,> for Relocation
{
	fn from(value: RelocAddend,) -> Self
	{
		Self {
			offset:       value.offset,
			addend:       Some(value.addend,),
			symbol_index: relocation_symbol_index(value.info,) as usize,
			ty:           relocation_type(value.info,),
		}
	}
}

fn relocation_symbol_index(info: u64,) -> u32
{
	(info >> 32) as u32
}

fn relocation_type(info: u64,) -> u32
{
	(info & 0xffff_ffff) as u32
}

// fn relocation_info(symbol: u64, ty: u64,) -> u64 {
// 	(symbol << 32) + ty
// }

pub struct Reloc
{
	pub offset: u64,
	pub info:   u64,
}

impl Reloc
{
	fn parse(binary: &[u8], offset: &mut usize,) -> Self
	{
		let reloc_offset: u64 = read_le_bytes(offset, binary,).unwrap();
		let info: u64 = read_le_bytes(offset, binary,).unwrap();
		Self { offset: reloc_offset, info, }
	}
}

impl From<Reloc,> for Relocation
{
	fn from(value: Reloc,) -> Self
	{
		Self {
			offset:       value.offset,
			addend:       None,
			symbol_index: relocation_symbol_index(value.info,) as usize,
			ty:           relocation_type(value.info,),
		}
	}
}

pub struct SymbolVersionSection
{
	pub bytes:   Vec<u8,>,
	pub context: Context,
}

impl SymbolVersionSection
{
	fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &Context,
	) -> PoisonGirlB<Option<Self,>,>
	{
		let (offset, size,) = if let Some(section_header,) = section_headers
			.iter()
			.find(|section_header| section_header.ty == SHT_GNU_VERSYM,)
		{
			(section_header.offset as usize, section_header.size as usize,)
		} else {
			return X(None,);
		};
		let bytes = binary[offset..offset + size].to_vec();
		X(Some(Self { bytes, context: ctx.clone(), },),)
	}
}

pub struct VersionDefinitionSection
{
	pub bytes:   Vec<u8,>,
	pub count:   usize,
	pub context: Context,
}

impl VersionDefinitionSection
{
	fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &Context,
	) -> PoisonGirlB<Option<Self,>,>
	{
		let (offset, size, count,) = if let Some(section_header,) =
			section_headers
				.iter()
				.find(|section_header| section_header.ty == SHT_GNU_VERDEF,)
		{
			(
				section_header.offset as usize,
				section_header.size as usize,
				section_header.info as usize,
			)
		} else {
			return X(None,);
		};
		let bytes = binary[offset..offset + size].to_vec();
		X(Some(Self { bytes, count, context: ctx.clone(), },),)
	}
}

pub struct VersionNeededSection
{
	pub bytes:   Vec<u8,>,
	pub count:   usize,
	pub context: Context,
}

impl VersionNeededSection
{
	fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &Context,
	) -> PoisonGirlB<Option<Self,>,>
	{
		let (offset, size, count,) = if let Some(section_header,) =
			section_headers
				.iter()
				.find(|section_header| section_header.ty == SHT_GNU_VERNEED,)
		{
			(
				section_header.offset as usize,
				section_header.size as usize,
				section_header.info as usize,
			)
		} else {
			return X(None,);
		};
		let bytes = binary[offset..offset + size].to_vec();
		X(Some(Self { bytes, count, context: ctx.clone(), },),)
	}
}

#[allow(dead_code)]
trait Integer<T: PrimitiveInteger,>:
	Add
	+ AddAssign
	+ Sub
	+ SubAssign
	+ Mul
	+ MulAssign
	+ Div
	+ DivAssign
	+ Shl
	+ Shr
	+ Clone
	+ Sum
	+ Sized
{
	fn cast_int(self,) -> T;
}

trait PrimitiveInteger:
	Add
	+ AddAssign
	+ Sub
	+ SubAssign
	+ Mul
	+ MulAssign
	+ Div
	+ DivAssign
	+ Shl<usize, Output: Sum,>
	+ Shr
	+ Clone
	+ Sum
	+ Sized
{
}

impl PrimitiveInteger for u8
{
}
impl PrimitiveInteger for u16
{
}
impl PrimitiveInteger for u32
{
}
impl PrimitiveInteger for u64
{
}
impl PrimitiveInteger for u128
{
}
impl PrimitiveInteger for usize
{
}
impl PrimitiveInteger for i8
{
}
impl PrimitiveInteger for i16
{
}
impl PrimitiveInteger for i32
{
}
impl PrimitiveInteger for i64
{
}
impl PrimitiveInteger for i128
{
}
impl PrimitiveInteger for isize
{
}

impl Integer<u8,> for u8
{
	fn cast_int(self,) -> u8
	{
		self
	}
}

impl Integer<u16,> for u8
{
	fn cast_int(self,) -> u16
	{
		self as u16
	}
}

impl Integer<u32,> for u8
{
	fn cast_int(self,) -> u32
	{
		self as u32
	}
}

impl Integer<u64,> for u8
{
	fn cast_int(self,) -> u64
	{
		self as u64
	}
}

impl Integer<u128,> for u8
{
	fn cast_int(self,) -> u128
	{
		self as u128
	}
}

impl Integer<usize,> for u8
{
	fn cast_int(self,) -> usize
	{
		self as usize
	}
}

impl Integer<i8,> for u8
{
	fn cast_int(self,) -> i8
	{
		self as i8
	}
}

impl Integer<i16,> for u8
{
	fn cast_int(self,) -> i16
	{
		self as i16
	}
}

impl Integer<i32,> for u8
{
	fn cast_int(self,) -> i32
	{
		self as i32
	}
}

impl Integer<i64,> for u8
{
	fn cast_int(self,) -> i64
	{
		self as i64
	}
}

impl Integer<i128,> for u8
{
	fn cast_int(self,) -> i128
	{
		self as i128
	}
}

impl Integer<isize,> for u8
{
	fn cast_int(self,) -> isize
	{
		self as isize
	}
}

trait AsInt<T: PrimitiveInteger,>
{
	fn as_int(&self,) -> T;
}

impl AsInt<u8,> for &[u8]
{
	fn as_int(&self,) -> u8
	{
		*self.first().unwrap()
	}
}

impl AsInt<u16,> for &[u8]
{
	fn as_int(&self,) -> u16
	{
		// unsafe { *(&self[..2] as *const _ as *const u16) }
		let mut rslt = 0;
		for i in (0..size_of::<u16,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u16;
		}

		rslt
	}
}

impl AsInt<u32,> for &[u8]
{
	fn as_int(&self,) -> u32
	{
		// unsafe { *(&self[..4] as *const _ as *const u32) }
		let mut rslt = 0;
		for i in (0..size_of::<u32,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u32;
		}

		rslt
	}
}

impl AsInt<u64,> for &[u8]
{
	fn as_int(&self,) -> u64
	{
		// unsafe { *(&self[..8] as *const _ as *const u64) }
		let mut rslt = 0;
		for i in (0..size_of::<u64,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u64;
		}

		rslt
	}
}

impl AsInt<u128,> for &[u8]
{
	fn as_int(&self,) -> u128
	{
		// unsafe { *(&self[..16] as *const _ as *const u128) }
		let mut rslt = 0;
		for i in (0..size_of::<u128,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u128;
		}

		rslt
	}
}

impl AsInt<usize,> for &[u8]
{
	fn as_int(&self,) -> usize
	{
		// unsafe { *(&self[..8] as *const _ as *const usize) }
		let mut rslt = 0;
		for i in (0..size_of::<usize,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as usize;
		}

		rslt
	}
}

impl AsInt<i8,> for &[u8]
{
	fn as_int(&self,) -> i8
	{
		*self.first().unwrap() as i8
	}
}

impl AsInt<i16,> for &[u8]
{
	fn as_int(&self,) -> i16
	{
		// unsafe { *(&self[..2] as *const _ as *const u16) }
		let mut rslt = 0;
		for i in (0..size_of::<i16,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i16;
		}

		rslt
	}
}

impl AsInt<i32,> for &[u8]
{
	fn as_int(&self,) -> i32
	{
		// unsafe { *(&self[..4] as *const _ as *const u32) }
		let mut rslt = 0;
		for i in (0..size_of::<i32,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i32;
		}

		rslt
	}
}

impl AsInt<i64,> for &[u8]
{
	fn as_int(&self,) -> i64
	{
		// unsafe { *(&self[..8] as *const _ as *const u64) }
		let mut rslt = 0;
		for i in (0..size_of::<i64,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i64;
		}

		rslt
	}
}

impl AsInt<i128,> for &[u8]
{
	fn as_int(&self,) -> i128
	{
		// unsafe { *(&self[..16] as *const _ as *const u128) }
		let mut rslt = 0;
		for i in (0..size_of::<i128,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i128;
		}

		rslt
	}
}

impl AsInt<isize,> for &[u8]
{
	fn as_int(&self,) -> isize
	{
		// unsafe { *(&self[..8] as *const _ as *const usize) }
		let mut rslt = 0;
		for i in (0..size_of::<isize,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as isize;
		}

		rslt
	}
}
