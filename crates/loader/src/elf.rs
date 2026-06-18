use {
	crate::elf::{
		dynamic::dynamic::Dynamic,
		elf_header::ElfHeader,
		program_header::ProgramHeader,
		relocation::RelocationSection,
		section_header::SectionHeader,
		string_table::StringTable,
		symbol_table::SymbolTable,
		version_sections::{
			SymbolVersionSection, VersionDefinitionSection,
			VersionNeededSection,
		},
	},
	alloc::{string::String, vec::Vec},
	core::{
		iter::Sum,
		mem::size_of,
		ops::{
			Add, AddAssign, Div, DivAssign, Mul, MulAssign, Shl, Shr, Sub,
			SubAssign,
		},
	},
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, X, Y, poison_girl_err,
	},
	program_header::ProgramHeaderType,
};

/// define abi version data
pub mod abi_version;
/// defines dynamic data
pub mod dynamic;
/// defines elf container size;
pub mod elf_container_size;
/// defines elf context data
pub mod elf_context;
/// defines elf header data structure
pub mod elf_header;
/// defines ElfHeaderIdent data
pub mod elf_header_ident;
/// main logic of elf parser
pub mod elf_parser;
/// defines ElfType data
pub mod elf_type;
/// defines ElfVersion data
pub mod elf_version;
/// defines endian data
pub mod endian;
/// define FileClass data
pub mod file_class;
/// Hash table implementations for symbol lookup
pub mod hash;
/// Program header parsing and types
pub mod program_header;
/// define relocation related data
pub mod relocation;
/// Section header parsing and types
pub mod section_header;
/// defines string context data
pub mod string_context;
/// define string table data
pub mod string_table;
/// define symbol table data
pub mod symbol_table;
/// define TargetOsAbi data
pub mod target_os_abi;
#[cfg(test)] pub(crate) mod test_helpers;
/// defines version sections definition data
pub mod version_sections;

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

fn read_le_bytes_or<I: PrimitiveInteger,>(
	offset: &mut usize,
	binary: &[u8],
	parser_pos: &'static str,
	stage: ElfParseStage,
) -> PoisonGirlB<I,>
where
	for<'a> &'a [u8]: AsInt<I,>,
{
	match read_le_bytes(offset, binary,) {
		Some(value,) => X(value,),
		None => Y(poison_girl_err!(ElfParseError::EndOfBinary {
			parser_pos,
			stage
		}),),
	}
}

pub struct Interpreter(Option<Vec<u8,>,>,);

impl AsRef<Option<Vec<u8,>,>,> for Interpreter
{
	fn as_ref(&self,) -> &Option<Vec<u8,>,>
	{
		&self.0
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

macro_rules! impl_as_int {
	($ty:ty) => {
		impl AsInt<$ty,> for &[u8]
		{
			fn as_int(&self,) -> $ty
			{
				let mut bytes = [0; size_of::<$ty,>()];
				for (dst, src,) in bytes.iter_mut().zip(self.iter().copied(),) {
					*dst = src;
				}
				<$ty>::from_le_bytes(bytes,)
			}
		}
	};
}

impl_as_int!(u8);
impl_as_int!(u16);
impl_as_int!(u32);
impl_as_int!(u64);
impl_as_int!(u128);
impl_as_int!(usize);
impl_as_int!(i8);
impl_as_int!(i16);
impl_as_int!(i32);
impl_as_int!(i64);
impl_as_int!(i128);
impl_as_int!(isize);
