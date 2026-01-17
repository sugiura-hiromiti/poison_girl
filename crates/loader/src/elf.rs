//! # ELF File Parsing and Loading Module
//!
//! This module provides comprehensive ELF (Executable and Linkable Format) file
//! parsing capabilities for the OSO bootloader. It supports parsing ELF
//! headers, program headers, section headers, and various ELF structures needed
//! for kernel loading.
//!
//! ## Features
//!
//! - **ELF Header Parsing**: Validates and parses ELF file headers
//! - **Program Header Processing**: Handles loadable segments and program
//!   information
//! - **Section Header Analysis**: Processes section headers for symbol tables
//!   and relocations
//! - **Dynamic Linking Support**: Handles dynamic symbols, relocations, and
//!   version information
//! - **Multi-architecture Support**: Works with 32-bit and 64-bit ELF files
//! - **Endianness Handling**: Supports both little-endian and big-endian
//!   formats
//!
//! ## ELF Structure Support
//!
//! The module supports parsing of:
//! - Symbol tables (static and dynamic)
//! - String tables
//! - Relocation sections (REL and RELA)
//! - Version information (definitions and requirements)
//! - Dynamic linking information
//! - Hash tables for symbol lookup
//!
//! ## Constants
//!
//! Various ELF format constants are defined for validation and parsing.

use crate::Rslt;
use crate::elf::hash::gnu_hash_len;
use crate::elf::hash::hash_len;
use crate::elf::program_header::ProgramHeader;
use crate::elf::section_header::SectionHeader;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;
use core::iter::Sum;
use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Div;
use core::ops::DivAssign;
use core::ops::Mul;
use core::ops::MulAssign;
use core::ops::Shl;
use core::ops::Shr;
use core::ops::Sub;
use core::ops::SubAssign;
use oso_error::OsoError;
use oso_error::loader::EfiParseError;
use oso_error::loader::EfiParseStage;
use oso_error::oso_err;
use program_header::ProgramHeaderType;
use section_header::SHT_GNU_VERDEF;
use section_header::SHT_GNU_VERNEED;
use section_header::SHT_GNU_VERSYM;
use section_header::SHT_REL;
use section_header::SHT_RELA;
use section_header::SHT_SYMTAB;
use section_header::get_string_table;

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

/// Represents a parsed ELF file with all its components
///
/// This structure contains all the parsed information from an ELF file,
/// including headers, tables, and metadata necessary for loading and
/// executing the binary.
///
/// # Fields
///
/// ## Core Structure
/// - `header`: Main ELF header containing file metadata
/// - `program_headers`: Array of program headers describing segments
/// - `section_headers`: Array of section headers describing sections
///
/// ## String and Symbol Tables
/// - `section_header_string_table`: Names of sections
/// - `dynamic_string_table`: Strings used by dynamic linking
/// - `dynamic_symbol_table`: Symbols for dynamic linking
/// - `symbol_table`: Static symbol table
/// - `string_table_for_symbol_table`: Strings for static symbols
///
/// ## Dynamic Linking Information
/// - `dynamic_info`: Dynamic section information
/// - `shared_object_name`: Name of shared object (if applicable)
/// - `interpreter`: Path to dynamic linker/interpreter
/// - `libraries`: List of required shared libraries
/// - `runtime_search_path_deprecated`: Deprecated runtime search paths
/// - `runtime_search_path`: Runtime library search paths
///
/// ## Relocation Information
/// - `dynamic_relocation_with_addend`: RELA relocations
/// - `dynamic_relocation`: REL relocations
/// - `procedure_linkage_table_relocation`: PLT relocations
/// - `section_relocations`: Per-section relocations
///
/// ## Version Information
/// - `symbol_version_section`: Symbol version information
/// - `version_definition_section`: Version definitions
/// - `version_needed_section`: Required versions
///
/// ## Flags
/// - `is_position_independent_executable`: Whether this is a PIE binary
pub struct Elf {
	pub header:                             ElfHeader,
	pub program_headers:                    Vec<ProgramHeader,>,
	pub section_headers:                    Vec<SectionHeader,>,
	pub section_header_string_table:        StringTable,
	pub dynamic_string_table:               StringTable,
	pub dynamic_symbol_table:               SymbolTable,
	pub symbol_table:                       SymbolTable,
	pub string_table_for_symbol_table:      StringTable,
	pub dynamic_info:                       Option<Dynamic,>,
	pub dynamic_relocation_with_addend:     RelocationSection,
	pub dynamic_relocation:                 RelocationSection,
	pub procedure_linkage_table_relocation: RelocationSection,
	pub section_relocations:                Vec<(usize, RelocationSection,),>,
	pub shared_object_name:                 Option<String,>,
	pub interpreter:                        Option<String,>,
	pub libraries:                          Vec<String,>,
	pub runtime_search_path_deprecated:     Vec<String,>,
	pub runtime_search_path:                Vec<String,>,
	pub symbol_version_section:             Option<SymbolVersionSection,>,
	pub version_definition_section:         Option<VersionDefinitionSection,>,
	pub version_needed_section:             Option<VersionNeededSection,>,
	pub is_position_independent_executable: bool,
}

impl Elf {
	/// Parses an ELF file from binary data
	///
	/// This method performs comprehensive parsing of an ELF file, extracting
	/// all relevant information needed for loading and execution. The parsing
	/// process includes validation of the ELF format and extraction of all
	/// supported ELF structures.
	///
	/// # Arguments
	///
	/// * `binary` - Raw bytes of the ELF file to parse
	///
	/// # Returns
	///
	/// * `Ok(Elf)` - Successfully parsed ELF structure
	/// * `Err(EfiParseError)` - If parsing fails due to invalid format or
	///   unsupported features
	///
	/// # Errors
	///
	/// This method can fail if:
	/// - The file is not a valid ELF file (invalid magic number)
	/// - The ELF format is unsupported (wrong architecture, endianness, etc.)
	/// - Required sections or headers are malformed
	/// - Memory allocation fails during parsing
	///
	/// # Parsing Process
	///
	/// 1. **Header Parsing**: Validates and parses the main ELF header
	/// 2. **Program Headers**: Extracts segment information for loading
	/// 3. **Section Headers**: Processes section metadata
	/// 4. **String Tables**: Extracts various string tables
	/// 5. **Symbol Tables**: Processes static and dynamic symbol information
	/// 6. **Dynamic Information**: Handles dynamic linking metadata
	/// 7. **Relocations**: Processes relocation entries
	/// 8. **Version Information**: Extracts symbol versioning data
	pub fn parse(binary: &[u8],) -> Rslt<Self, EfiParseError,> {
		let header = ElfHeader::parse(binary,)?;
		oso_proc_macro::test_elf_header_parse!(header);

		let mut offset = header.program_header_offset as usize;
		let program_headers = ProgramHeader::parse(
			binary,
			&mut offset,
			header.program_header_count as usize,
		)?;

		let mut interpreter = None;
		for program_header in &program_headers {
			if program_header.ty == ProgramHeaderType::Interp
				&& program_header.file_size != 0
			{
				let count = program_header.file_size as usize - 1;
				let offset = program_header.offset as usize;

				interpreter = Some(
					StringContext::Length(count,)
						.read_bytes(&binary[offset..],)?,
				);
			}
		}

		offset = header.section_header_offset as usize;
		let section_headers = SectionHeader::parse(
			binary,
			&mut offset,
			header.section_header_count as usize,
		)?;

		let string_table_index =
			header.section_header_index_of_section_name_string_table as usize;
		let section_header_string_table =
			get_string_table(&section_headers, string_table_index, binary,)?;

		let ctx = &Context::default();
		let mut symbol_table = SymbolTable::default();
		let mut string_table_for_symbol_table = StringTable::default();
		if let Some(section_header,) = section_headers
			.iter()
			.rfind(|section_header| section_header.ty == SHT_SYMTAB,)
		{
			let size = section_header.entry_size;
			let count = if size == 0 { 0 } else { section_header.size / size };
			symbol_table = SymbolTable::parse(
				binary,
				section_header.offset as usize,
				count as usize,
				ctx,
			)?;
			string_table_for_symbol_table = get_string_table(
				&section_headers,
				section_header.link as usize,
				binary,
			)?;
		}

		let mut is_position_independent_executable = false;
		let mut shared_object_name = None;
		let mut libraries = vec![];
		let mut runtime_search_path_deprecated = vec![];
		let mut runtime_search_path = vec![];
		let mut dynamic_symbol_table = SymbolTable::default();
		let mut dynamic_relocation_with_addend = RelocationSection::default();
		let mut dynamic_relocation = RelocationSection::default();
		let mut procedure_linkage_table_relocation =
			RelocationSection::default();
		let mut dynamic_string_table = StringTable::default();

		let dynamic_info = Dynamic::parse(binary, &program_headers,)?;
		if let Some(ref dynamic,) = dynamic_info {
			let dyn_info = &dynamic.info;
			is_position_independent_executable =
				dyn_info.extended_flags & Dynamic::DF_EXTEND_PIE != 0;
			dynamic_string_table = StringTable::parse(
				binary,
				dyn_info.string_table_address,
				dyn_info.string_table_size,
				0x0,
			)?;

			if dyn_info.shared_object_name_offset != 0 {
				shared_object_name = dynamic_string_table
					.get_at(dyn_info.shared_object_name_offset,);
			}
			if dyn_info.version_need_count > 0 {
				libraries = dynamic.get_libraries(&dynamic_string_table,);
			}

			for dynamic in &dynamic.dyns {
				if dynamic.tag == Dynamic::DT_RPATH {
					if let Some(path,) =
						dynamic_string_table.get_at(dynamic.val as usize,)
					{
						runtime_search_path_deprecated.push(path,);
					}
				} else if dynamic.tag == Dynamic::DT_RUNPATH
					&& let Some(path,) =
						dynamic_string_table.get_at(dynamic.val as usize,)
				{
					runtime_search_path.push(path,);
				}
			}

			dynamic_relocation_with_addend = RelocationSection::parse(
				binary,
				dyn_info.relocation_addend,
				dyn_info.relocation_addend_size,
				true,
				ctx,
			)?;
			dynamic_relocation = RelocationSection::parse(
				binary,
				dyn_info.relocation,
				dyn_info.relocation_size,
				false,
				ctx,
			)?;
			let is_relocation_addrend =
				dyn_info.plt_relocation_type == Dynamic::DT_RELA;
			procedure_linkage_table_relocation = RelocationSection::parse(
				binary,
				dyn_info.jmp_relocation_address,
				dyn_info.plt_relocation_size,
				is_relocation_addrend,
				ctx,
			)?;

			let mut symbols_count = if let Some(gnu_hash,) = dyn_info.gnu_hash {
				gnu_hash_len(binary, gnu_hash as usize, ctx,)?
			} else if let Some(hash,) = dyn_info.hash {
				hash_len(binary, hash as usize, header.machine, ctx,)?
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
				symbols_count =
					cmp::max(symbols_count, max_relocation_symbol + 1,);
			}
			dynamic_symbol_table = SymbolTable::parse(
				binary,
				dyn_info.symbol_table,
				symbols_count,
				ctx,
			)?;
		}

		let mut section_relocations = vec![];
		for (index, section,) in section_headers.iter().enumerate() {
			let is_relocation_addrend = section.ty == SHT_RELA;
			if is_relocation_addrend || section.ty == SHT_REL {
				section.check_size(binary.len(),)?;
				let section_header_relocation_section =
					RelocationSection::parse(
						binary,
						section.offset as usize,
						section.size as usize,
						is_relocation_addrend,
						ctx,
					)?;
				section_relocations
					.push((index, section_header_relocation_section,),);
			}
		}

		let symbol_version_section =
			SymbolVersionSection::parse(binary, &section_headers, ctx,)?;
		let version_definition_section =
			VersionDefinitionSection::parse(binary, &section_headers, ctx,)?;
		let version_needed_section =
			VersionNeededSection::parse(binary, &section_headers, ctx,)?;

		Ok(Self {
			header,
			program_headers,
			section_headers,
			section_header_string_table,
			dynamic_string_table,
			dynamic_symbol_table,
			symbol_table,
			string_table_for_symbol_table,
			dynamic_info,
			dynamic_relocation_with_addend,
			dynamic_relocation,
			procedure_linkage_table_relocation,
			section_relocations,
			shared_object_name,
			interpreter,
			libraries,
			runtime_search_path_deprecated,
			runtime_search_path,
			symbol_version_section,
			version_definition_section,
			version_needed_section,
			is_position_independent_executable,
		},)
	}

	/// Returns whether this ELF file is 64-bit
	///
	/// # Returns
	///
	/// `true` if this is a 64-bit ELF file, `false` if it's 32-bit
	pub fn is_64(&self,) -> bool {
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
	pub fn is_lib(&self,) -> bool {
		self.header.is_lib() && !self.is_position_independent_executable
	}

	/// Returns whether this ELF file uses little-endian byte ordering
	///
	/// # Returns
	///
	/// `true` if little-endian, `false` if big-endian
	pub fn is_little_endian(&self,) -> bool {
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
	pub fn entry_point_address(&self,) -> usize {
		self.header.entry as usize
	}
}

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfHeader {
	pub ident: ElfHeaderIdent,
	pub ty: ElfType,
	pub machine: u16,
	pub version: u32,
	pub entry: u64,
	pub program_header_offset: u64,
	pub section_header_offset: u64,
	pub flags: u32,
	pub elf_header_size: u16,
	pub program_header_entry_size: u16,
	pub program_header_count: u16,
	pub section_header_entry_size: u16,
	pub section_header_count: u16,
	pub section_header_index_of_section_name_string_table: u16,
}

impl ElfHeader {
	/// Intel 80386
	pub const EM_386: u16 = 3;
	/// Freescale 56800EX DSC
	pub const EM_56800EX: u16 = 200;
	/// Motorola MC68HC05 microcontroller
	pub const EM_68HC05: u16 = 72;
	/// Motorola MC68HC08 microcontroller
	pub const EM_68HC08: u16 = 71;
	/// Motorola MC68HC11 microcontroller
	pub const EM_68HC11: u16 = 70;
	/// Motorola M68HC12
	pub const EM_68HC12: u16 = 53;
	/// Motorola MC68HC16 microcontroller
	pub const EM_68HC16: u16 = 69;
	/// Motorola m68k family
	pub const EM_68K: u16 = 4;
	/// Renesas 78KOR
	pub const EM_78KOR: u16 = 199;
	/// Intel 8051 and variants
	pub const EM_8051: u16 = 165;
	/// Intel 80860
	pub const EM_860: u16 = 7;
	/// Motorola m88k family
	pub const EM_88K: u16 = 5;
	/// Intel 80960
	pub const EM_960: u16 = 19;
	// reserved 182
	/// ARM AARCH64
	pub const EM_AARCH64: u16 = 183;
	/// Altera Nios II
	pub const EM_ALTERA_NIOS2: u16 = 113;
	/// AMD GPU
	pub const EM_AMDGPU: u16 = 224;
	/// Argonaut RISC Core
	pub const EM_ARC: u16 = 45;
	/// Arca RISC
	pub const EM_ARCA: u16 = 109;
	/// ARC International ARCompact
	pub const EM_ARC_COMPACT: u16 = 93;
	/// Synopsys ARCompact V2
	pub const EM_ARC_COMPACT2: u16 = 195;
	/// ARM
	pub const EM_ARM: u16 = 40;
	/// Atmel AVR 8-bit microcontroller
	pub const EM_AVR: u16 = 83;
	// reserved 184
	/// Amtel 32-bit microprocessor
	pub const EM_AVR32: u16 = 185;
	/// Beyond BA1
	pub const EM_BA1: u16 = 201;
	/// Beyond BA2
	pub const EM_BA2: u16 = 202;
	/// Analog Devices Blackfin DSP
	pub const EM_BLACKFIN: u16 = 106;
	/// Linux BPF -- in-kernel virtual machine
	pub const EM_BPF: u16 = 247;
	/// Infineon C16x/XC16x
	pub const EM_C166: u16 = 116;
	/// Paneve CDP
	pub const EM_CDP: u16 = 215;
	/// Freescale Communication Engine RISC
	pub const EM_CE: u16 = 119;
	/// CloudShield
	pub const EM_CLOUDSHIELD: u16 = 192;
	/// Cognitive Smart Memory Processor
	pub const EM_COGE: u16 = 216;
	/// Motorola Coldfire
	pub const EM_COLDFIRE: u16 = 52;
	/// Bluechip CoolEngine
	pub const EM_COOL: u16 = 217;
	/// KIPO-KAIST Core-A 1st gen.
	pub const EM_COREA_1ST: u16 = 193;
	/// KIPO-KAIST Core-A 2nd gen.
	pub const EM_COREA_2ND: u16 = 194;
	/// National Semi. CompactRISC
	pub const EM_CR: u16 = 103;
	/// National Semi. CompactRISC CR16
	pub const EM_CR16: u16 = 177;
	/// Cray NV2 vector architecture
	pub const EM_CRAYNV2: u16 = 172;
	/// Axis Communications 32-bit emb.proc
	pub const EM_CRIS: u16 = 76;
	/// National Semi. CompactRISC CRX
	pub const EM_CRX: u16 = 114;
	/// C-SKY
	pub const EM_CSKY: u16 = 252;
	/// CSR Kalimba
	pub const EM_CSR_KALIMBA: u16 = 219;
	/// NVIDIA CUDA
	pub const EM_CUDA: u16 = 190;
	/// Cypress M8C
	pub const EM_CYPRESS_M8C: u16 = 161;
	/// Mitsubishi D10V
	pub const EM_D10V: u16 = 85;
	/// Mitsubishi D30V
	pub const EM_D30V: u16 = 86;
	/// New Japan Radio (NJR) 24-bit DSP
	pub const EM_DSP24: u16 = 136;
	/// Microchip Technology dsPIC30F
	pub const EM_DSPIC30F: u16 = 118;
	/// Icera Semi. Deep Execution Processor
	pub const EM_DXP: u16 = 112;
	/// Cyan Technology eCOG16
	pub const EM_ECOG16: u16 = 176;
	/// Cyan Technology eCOG1X
	pub const EM_ECOG1X: u16 = 168;
	/// Cyan Technology eCOG2
	pub const EM_ECOG2: u16 = 134;
	/// KM211 KMX16
	pub const EM_EMX16: u16 = 212;
	/// KM211 KMX8
	pub const EM_EMX8: u16 = 213;
	/// Freescale Extended Time Processing Unit
	pub const EM_ETPU: u16 = 178;
	/// eXcess configurable cpu
	pub const EM_EXCESS: u16 = 111;
	/// Fujitsu F2MC16
	pub const EM_F2MC16: u16 = 104;
	/// Digital Alpha
	pub const EM_FAKE_ALPHA: u16 = 41;
	/// Element 14 64-bit DSP Processor
	pub const EM_FIREPATH: u16 = 78;
	/// Fujitsu FR20
	pub const EM_FR20: u16 = 37;
	/// Fujitsu FR30
	pub const EM_FR30: u16 = 84;
	/// FTDI Chip FT32
	pub const EM_FT32: u16 = 222;
	/// Siemens FX66 microcontroller
	pub const EM_FX66: u16 = 66;
	/// Hitachi H8S
	pub const EM_H8S: u16 = 48;
	/// Hitachi H8/300
	pub const EM_H8_300: u16 = 46;
	/// Hitachi H8/300H
	pub const EM_H8_300H: u16 = 47;
	/// Hitachi H8/500
	pub const EM_H8_500: u16 = 49;
	/// Harvard University machine-independent object files
	pub const EM_HUANY: u16 = 81;
	/// Intel MCU
	pub const EM_IAMCU: u16 = 6;
	/// Intel Merced
	pub const EM_IA_64: u16 = 50;
	/// Intel Graphics Technology
	pub const EM_INTELGT: u16 = 205;
	/// Ubicom IP2xxx
	pub const EM_IP2K: u16 = 101;
	/// Infineon Technologies 32-bit emb.proc
	pub const EM_JAVELIN: u16 = 77;
	/// Intel K10M
	pub const EM_K10M: u16 = 181;
	// reserved 206-209
	/// KM211 KM32
	pub const EM_KM32: u16 = 210;
	/// KM211 KMX32
	pub const EM_KMX32: u16 = 211;
	/// KM211 KVARC
	pub const EM_KVARC: u16 = 214;
	/// Intel L10M
	pub const EM_L10M: u16 = 180;
	/// RISC for Lattice FPGA
	pub const EM_LATTICEMICO32: u16 = 138;
	// Loongarch 64
	pub const EM_LOONGARCH: u16 = 258;
	/// Renesas M16C
	pub const EM_M16C: u16 = 117;
	/// AT&T WE 32100
	pub const EM_M32: u16 = 1;
	/// Renesas M32C
	pub const EM_M32C: u16 = 120;
	/// Mitsubishi M32R
	pub const EM_M32R: u16 = 88;
	/// M2000 Reconfigurable RISC
	pub const EM_MANIK: u16 = 171;
	/// MAX processor
	pub const EM_MAX: u16 = 102;
	/// Dallas Semi. MAXQ30 mc
	pub const EM_MAXQ30: u16 = 169;
	/// Microchip 8-bit PIC(r)
	pub const EM_MCHP_PIC: u16 = 204;
	/// MCST Elbrus
	pub const EM_MCST_ELBRUS: u16 = 175;
	/// Toyota ME16 processor
	pub const EM_ME16: u16 = 59;
	/// Imagination Tech. META
	pub const EM_METAG: u16 = 174;
	/// Xilinx MicroBlaze
	pub const EM_MICROBLAZE: u16 = 189;
	/// MIPS R3000 big-endian
	pub const EM_MIPS: u16 = 8;
	/// MIPS R3000 little-endian
	pub const EM_MIPS_RS3_LE: u16 = 10;
	/// Stanford MIPS-X
	pub const EM_MIPS_X: u16 = 51;
	/// Fujitsu MMA Multimedia Accelerator
	pub const EM_MMA: u16 = 54;
	// reserved 145-159
	/// STMicroelectronics 64bit VLIW DSP
	pub const EM_MMDSP_PLUS: u16 = 160;
	/// Donald Knuth's educational 64-bit proc
	pub const EM_MMIX: u16 = 80;
	/// Matsushita MN10200
	pub const EM_MN10200: u16 = 90;
	/// Matsushita MN10300
	pub const EM_MN10300: u16 = 89;
	/// Moxie processor
	pub const EM_MOXIE: u16 = 223;
	/// Texas Instruments msp430
	pub const EM_MSP430: u16 = 105;
	/// Sony nCPU embeeded RISC
	pub const EM_NCPU: u16 = 56;
	/// Denso NDR1 microprocessor
	pub const EM_NDR1: u16 = 57;
	/// Andes Tech. compact code emb. RISC
	pub const EM_NDS32: u16 = 167;
	/// TODO: use Enum with explicit discriminant and get debug printer for
	/// free? No machine
	pub const EM_NONE: u16 = 0;
	/// Nanoradio Optimized RISC
	pub const EM_NORC: u16 = 218;
	/// National Semi. 32000
	pub const EM_NS32K: u16 = 97;
	pub const EM_NUM: u16 = 248;
	/// Open8 RISC
	pub const EM_OPEN8: u16 = 196;
	/// OpenRISC 32-bit embedded processor
	pub const EM_OPENRISC: u16 = 92;
	// reserved 11-14
	/// HPPA
	pub const EM_PARISC: u16 = 15;
	/// Siemens PCP
	pub const EM_PCP: u16 = 55;
	/// Digital PDP-10
	pub const EM_PDP10: u16 = 64;
	/// Digital PDP-11
	pub const EM_PDP11: u16 = 65;
	/// Sony DSP Processor
	pub const EM_PDSP: u16 = 63;
	/// picoJava
	pub const EM_PJ: u16 = 91;
	/// PowerPC
	pub const EM_PPC: u16 = 20;
	/// PowerPC 64-bit
	pub const EM_PPC64: u16 = 21;
	/// SiTera Prism
	pub const EM_PRISM: u16 = 82;
	/// QUALCOMM DSP6
	pub const EM_QDSP6: u16 = 164;
	/// Renesas R32C
	pub const EM_R32C: u16 = 162;
	/// Motorola RCE
	pub const EM_RCE: u16 = 39;
	/// TRW RH-32
	pub const EM_RH32: u16 = 38;
	// reserved 225-242
	/// RISC-V
	pub const EM_RISCV: u16 = 243;
	/// Renesas RL78
	pub const EM_RL78: u16 = 197;
	/// Freescale RS08
	pub const EM_RS08: u16 = 132;
	/// Renesas RX
	pub const EM_RX: u16 = 173;
	/// IBM System/370
	pub const EM_S370: u16 = 9;
	/// IBM S390
	pub const EM_S390: u16 = 22;
	/// Sunplus S+core7 RISC
	pub const EM_SCORE7: u16 = 135;
	/// Sharp embedded microprocessor
	pub const EM_SEP: u16 = 108;
	/// Seiko Epson C17
	pub const EM_SE_C17: u16 = 139;
	/// Seiko Epson S1C33 family
	pub const EM_SE_C33: u16 = 107;
	/// Hitachi SH
	pub const EM_SH: u16 = 42;
	/// Analog Devices SHARC family
	pub const EM_SHARC: u16 = 133;
	/// Infineon Tech. SLE9X
	pub const EM_SLE9X: u16 = 179;
	/// Trebia SNP 1000
	pub const EM_SNP1K: u16 = 99;
	/// SUN SPARC
	pub const EM_SPARC: u16 = 2;
	/// Sun's "v8plus"
	pub const EM_SPARC32PLUS: u16 = 18;
	/// SPARC v9 64-bit
	pub const EM_SPARCV9: u16 = 43;
	/// IBM SPU/SPC
	pub const EM_SPU: u16 = 23;
	/// STMicroelectronic ST100 processor
	pub const EM_ST100: u16 = 60;
	/// STMicroelectronics ST19 8 bit mc
	pub const EM_ST19: u16 = 74;
	/// STMicroelectronics ST200
	pub const EM_ST200: u16 = 100;
	/// STmicroelectronics ST7 8 bit mc
	pub const EM_ST7: u16 = 68;
	/// STMicroelectronics ST9+ 8/16 mc
	pub const EM_ST9PLUS: u16 = 67;
	/// Motorola Start*Core processor
	pub const EM_STARCORE: u16 = 58;
	/// STMicroelectronics STM8
	pub const EM_STM8: u16 = 186;
	/// STMicroelectronics STxP7x
	pub const EM_STXP7X: u16 = 166;
	/// Silicon Graphics SVx
	pub const EM_SVX: u16 = 73;
	/// Tileta TILE64
	pub const EM_TILE64: u16 = 187;
	/// Tilera TILE-Gx
	pub const EM_TILEGX: u16 = 191;
	/// Tilera TILEPro
	pub const EM_TILEPRO: u16 = 188;
	/// Advanced Logic Corp. Tinyj emb.fam
	pub const EM_TINYJ: u16 = 61;
	/// Texas Instruments App. Specific RISC
	pub const EM_TI_ARP32: u16 = 143;
	/// Texas Instruments TMS320C2000 DSP
	pub const EM_TI_C2000: u16 = 141;
	/// Texas Instruments TMS320C55x DSP
	pub const EM_TI_C5500: u16 = 142;
	/// Texas Instruments TMS320C6000 DSP
	pub const EM_TI_C6000: u16 = 140;
	/// Texas Instruments Prog. Realtime Unit
	pub const EM_TI_PRU: u16 = 144;
	/// Thompson Multimedia General Purpose Proc
	pub const EM_TMM_GPP: u16 = 96;
	/// Tenor Network TPC
	pub const EM_TPC: u16 = 98;
	/// Siemens Tricore
	pub const EM_TRICORE: u16 = 44;
	/// NXP Semi. TriMedia
	pub const EM_TRIMEDIA: u16 = 163;
	// reserved 121-130
	/// Altium TSK3000
	pub const EM_TSK3000: u16 = 131;
	/// PKU-Unity & MPRC Peking Uni. mc series
	pub const EM_UNICORE: u16 = 110;
	// reserved 24-35
	/// NEC V800 series
	pub const EM_V800: u16 = 36;
	/// NEC v850
	pub const EM_V850: u16 = 87;
	/// Digital VAX
	pub const EM_VAX: u16 = 75;
	/// Alphamosaic VideoCore
	pub const EM_VIDEOCORE: u16 = 95;
	/// Broadcom VideoCore III
	pub const EM_VIDEOCORE3: u16 = 137;
	/// Broadcom VideoCore V
	pub const EM_VIDEOCORE5: u16 = 198;
	/// Controls and Data Services VISIUMcore
	pub const EM_VISIUM: u16 = 221;
	// reserved 16
	/// Fujitsu VPP500
	pub const EM_VPP500: u16 = 17;
	/// AMD x86-64 architecture
	pub const EM_X86_64: u16 = 62;
	/// XMOS xCORE
	pub const EM_XCORE: u16 = 203;
	/// Motorola XGATE
	pub const EM_XGATE: u16 = 115;
	/// New Japan Radio (NJR) 16-bit DSP
	pub const EM_XIMO16: u16 = 170;
	/// Tensilica Xtensa Architecture
	pub const EM_XTENSA: u16 = 94;
	/// Zilog Z80
	pub const EM_Z80: u16 = 220;
	/// LSI Logic 16-bit DSP Processor
	pub const EM_ZSP: u16 = 79;

	pub fn parse(binary: &[u8],) -> Rslt<Self, EfiParseError,> {
		let ident = &binary[..ELF_IDENT_SIZE];
		let ident = ElfHeaderIdent::new(ident,)?;
		let remain = &binary[ELF_IDENT_SIZE..];
		header_flag_fields(ident, remain,)
	}

	fn is_64(&self,) -> bool {
		self.ident.is_64()
	}

	fn is_lib(&self,) -> bool {
		self.ty.is_lib()
	}

	fn is_little_endian(&self,) -> bool {
		self.ident.is_little_endian()
	}
}

fn header_flag_fields(
	ident: ElfHeaderIdent,
	ident_remain: &[u8],
) -> Rslt<ElfHeader, EfiParseError,> {
	let offset = &mut 0;

	macro_rules! fields {
		($field:ident) => {
			let $field =
				read_le_bytes(offset, ident_remain,).ok_or_else(|| {
					let field = stringify!($field);
					oso_error::oso_err!(oso_error::loader::EfiParseError::EndOfBinary{
						parser_pos: field,
						stage: oso_error::loader::EfiParseStage::Header
					})
				})?;
		};
		($($fields:ident,)*)=>{
			$(
				fields!($fields);
			)*
		};
	}

	let ty: u16 = read_le_bytes(offset, ident_remain,).ok_or(oso_err!(
		EfiParseError::EndOfBinary {
			parser_pos: "ty",
			stage:      oso_error::loader::EfiParseStage::Header,
		}
	),)?;
	let ty = ElfType::try_from(ty,)?;
	fields!(
		machine,
		version,
		entry,
		program_header_offset,
		section_header_offset,
		flags,
		elf_header_size,
		program_header_entry_size,
		program_header_count,
		section_header_entry_size,
		section_header_count,
		section_header_index_of_section_name_string_table,
	);

	Ok(ElfHeader {
		ident,
		ty,
		machine,
		version,
		entry,
		program_header_offset,
		section_header_offset,
		flags,
		elf_header_size,
		program_header_entry_size,
		program_header_count,
		section_header_entry_size,
		section_header_count,
		section_header_index_of_section_name_string_table,
	},)
}

//fn read_le_bytes<I: PrimitiveInteger + Sum<<I as Shl<usize,>>::Output,>,>(
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
pub enum ElfType {
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

impl ElfType {
	fn is_lib(&self,) -> bool {
		*self == Self::SharedObject
	}
}

impl TryFrom<u16,> for ElfType {
	type Error = OsoError<EfiParseError,>;

	fn try_from(value: u16,) -> Result<Self, Self::Error,> {
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
			_ => return Err(oso_err!(EfiParseError::UnknownEfiType(value)),),
		};
		Ok(ty,)
	}
}

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfHeaderIdent {
	pub file_class:    FileClass,
	pub endianness:    Endian,
	pub elf_version:   ElfVersion,
	pub target_os_abi: TargetOsAbi,
	pub abi_version:   AbiVersion,
}

impl ElfHeaderIdent {
	fn new(ident: &[u8],) -> Rslt<Self, EfiParseError,> {
		if ident.len() != ELF_IDENT_SIZE {
			return Err(oso_err!(EfiParseError::InvalidIdentLen(ident.len())),);
		}

		// check magic number
		// size of elf magic number is 4
		if &ident[0..4] != ELF_MAGIC_NUMBER {
			return Err(oso_err!(EfiParseError::BadMagicNumber(
				ident[0], ident[1], ident[2], ident[3]
			)),);
		}

		let file_class = FileClass::try_from(ident[ELF_FILE_CLASS_INDEX],)?;
		let endianness = Endian::try_from(ident[ELF_ENDIANNESS_INDEX],)?;
		let elf_version = ElfVersion(ident[ELF_VERSION_INDEX],);
		let target_os_abi = TargetOsAbi::try_from(ident[ELF_OS_ABI_INDEX],)?;
		let abi_version = AbiVersion(ident[ELF_ABI_VERSION_INDEX],);

		Ok(Self {
			file_class,
			endianness,
			elf_version,
			target_os_abi,
			abi_version,
		},)
	}

	fn is_64(&self,) -> bool {
		self.file_class.is_64()
	}

	fn is_little_endian(&self,) -> bool {
		self.endianness.is_little_endian()
	}
}

#[derive(PartialEq, Eq, Debug, Default,)]
pub enum FileClass {
	Bit32,
	#[default]
	Bit64,
}

impl FileClass {
	fn is_64(&self,) -> bool {
		*self == FileClass::Bit64
	}
}

impl TryFrom<u8,> for FileClass {
	type Error = OsoError<EfiParseError,>;

	fn try_from(value: u8,) -> Result<Self, Self::Error,> {
		match value {
			ELF_32_BIT_OBJECT => Ok(Self::Bit32,),
			ELF_64_BIT_OBJECT => Ok(Self::Bit64,),
			_ => Err(oso_err!(EfiParseError::InvalidFileClass(value)),),
		}
	}
}

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfVersion(pub u8,);

impl ElfVersion {
	pub const ONE: Self = Self(1,);
}

#[non_exhaustive]
#[derive(Debug, Default, PartialEq, Eq,)]
pub enum TargetOsAbi {
	SysV,
	#[default]
	Arm,
	Standalone,
}

impl TryFrom<u8,> for TargetOsAbi {
	type Error = OsoError<EfiParseError,>;

	fn try_from(value: u8,) -> Result<Self, Self::Error,> {
		match value {
			0x0 => Ok(Self::SysV,),
			0x53 => Ok(Self::Arm,),
			0x61 => Ok(Self::Standalone,),
			_ => Err(oso_err!(EfiParseError::OsAbiOutOfSupport(value)),),
		}
	}
}

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct AbiVersion(pub u8,);
impl AbiVersion {
	pub const ONE: Self = Self(0,);
}

#[derive(Default,)]
pub struct StringTable {
	pub delimitor: StringContext,
	pub bytes:     Vec<u8,>,
	pub strings:   Vec<(usize, String,),>,
}

impl StringTable {
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
	) -> Rslt<Self, EfiParseError,> {
		let (end, overflow,) = offset.overflowing_add(len,);
		if overflow || end > binary.len() {
			return Err(oso_err!(EfiParseError::SizeOverflow {
				stage:    EfiParseStage::StringTable,
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
			let s = rslt.delimitor.read_bytes(&binary[i..],)?;
			let len = s.len();
			rslt.strings.push((i, s,),);
			i += len + 1;
		}

		Ok(rslt,)
	}

	fn from_slice(bytes: &[u8], delimiter: u8,) -> Self {
		Self {
			delimitor: StringContext::Delimiter(delimiter,),
			bytes:     bytes.to_vec(),
			strings:   vec![],
		}
	}

	fn get_at(&self, offset: usize,) -> Option<String,> {
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

pub enum StringContext {
	Delimiter(u8,),
	DelimiterUntil(u8, usize,),
	Length(usize,),
}

impl StringContext {
	fn read_bytes(&self, bytes: &[u8],) -> Rslt<String, EfiParseError,> {
		let bytes = match self {
			StringContext::Delimiter(delimiter,) => {
				let mut i = 0;
				while let a = &bytes[i..=i]
					&& a != [*delimiter,]
				{
					i += 1;
					if i >= bytes.len() {
						return Err(oso_err!(
							EfiParseError::DelimiterNotFound(*delimiter)
						),);
					}
				}

				&bytes[..i]
			},
			StringContext::DelimiterUntil(..,) => todo!(),
			StringContext::Length(l,) => &bytes[..*l],
		};

		let rslt = String::from_utf8_lossy(bytes,);
		Ok(rslt.to_string(),)
	}
}

impl Default for StringContext {
	fn default() -> Self {
		// null delimiter
		Self::Delimiter(0,)
	}
}

#[derive(Default,)]
pub struct SymbolTable {
	pub bytes: Vec<u8,>,
	pub count: usize,
	pub ctx:   Context,
	pub start: usize,
	pub end:   usize,
}

impl SymbolTable {
	/// size of symbol structure in 64bit.
	const SIZE_OF_SYMBOL_64: usize = 4 + 1 + 1 + 2 + 8 + 8;

	fn parse(
		binary: &[u8],
		offset: usize,
		count: usize,
		context: &Context,
	) -> Rslt<Self, EfiParseError,> {
		let size = count
			.checked_mul(match context.container {
				Container::Little => todo!(),
				Container::Big => Self::SIZE_OF_SYMBOL_64,
			},)
			.ok_or(oso_err!(EfiParseError::TooManySymbolsOffset {
				offset,
				count
			}),)?;

		let bytes = binary[offset..offset + size].to_vec();

		Ok(SymbolTable {
			bytes,
			count,
			ctx: context.clone(),
			start: offset,
			end: offset + size,
		},)
	}
}

#[derive(Clone, Default,)]
pub struct Context {
	pub container: Container,
	pub le:        Endian,
}

/// the size of a binary container
#[derive(PartialEq, Eq, Clone,)]
pub enum Container {
	Little,
	Big,
}

impl Default for Container {
	fn default() -> Self {
		Self::Big
	}
}

#[derive(Debug, PartialEq, Eq, Clone,)]
pub enum Endian {
	Little,
	Big,
}

impl Default for Endian {
	fn default() -> Self {
		Self::Big
	}
}

impl Endian {
	fn is_little_endian(&self,) -> bool {
		*self == Self::Little
	}
}

impl TryFrom<u8,> for Endian {
	type Error = OsoError<EfiParseError,>;

	fn try_from(value: u8,) -> Result<Self, Self::Error,> {
		match value {
			1 => Ok(Self::Little,),
			2 => Ok(Self::Big,),
			_ => Err(oso_err!(EfiParseError::InvalidEndianFlag(value)),),
		}
	}
}

pub struct Dynamic {
	pub dyns: Vec<Dyn,>,
	pub info: DynamicInfo,
}

impl Dynamic {
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

	fn parse(
		binary: &[u8],
		program_headers: &Vec<ProgramHeader,>,
	) -> Rslt<Option<Self,>, EfiParseError,> {
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

				return Ok(Some(Dynamic { dyns, info, },),);
			}
		}

		Ok(None,)
	}

	fn get_libraries(&self, string_table: &StringTable,) -> Vec<String,> {
		let count = self.dyns.len().min(self.info.version_need_count as usize,);
		let mut needed = Vec::with_capacity(count,);
		for dynamic in &self.dyns {
			if dynamic.tag == Self::DT_NEEDED
				&& let Some(lib,) = string_table.get_at(dynamic.val as usize,)
			{
				needed.push(lib,);
			}
		}
		needed
	}
}

pub struct Dyn {
	pub tag: u64,
	pub val: u64,
}

impl Dyn {
	const SIZE_OF_DYN_32: usize = 8;
	const SIZE_OF_DYN_64: usize = 16;

	fn size_of(Context { container, .. }: &Context,) -> usize {
		match container {
			Container::Little => Self::SIZE_OF_DYN_32,
			Container::Big => Self::SIZE_OF_DYN_64,
		}
	}

	fn parse(bytes: &[u8], offset: &mut usize,) -> Self {
		let tag = read_le_bytes(offset, bytes,).unwrap();
		let val = read_le_bytes(offset, bytes,).unwrap();
		Self { tag, val, }
	}
}

#[derive(Default,)]
pub struct DynamicInfo {
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

impl DynamicInfo {
	pub fn update(&mut self, phdrs: &[ProgramHeader], dynamic: &Dyn,) {
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
	program_headers: &[ProgramHeader],
	address: u64,
) -> Option<u64,> {
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

#[derive(Default,)]
pub struct RelocationSection {
	pub bytes:   Vec<u8,>,
	pub count:   usize,
	pub context: RelocationContext,
	pub start:   usize,
	pub end:     usize,
}

impl RelocationSection {
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
	) -> Rslt<Self, EfiParseError,> {
		let bytes =
			if size != 0 { &binary[offset..offset + size] } else { &[] }
				.to_vec();

		Ok(Self {
			bytes,
			count: size / Self::size(is_addend, ctx,),
			context: (is_addend, ctx.clone(),),
			start: offset,
			end: offset + size,
		},)
	}

	fn size(
		is_relocation_addrend: bool,
		Context { container, .. }: &Context,
	) -> usize {
		match (is_relocation_addrend, container,) {
			(true, Container::Little,) => Self::SIZE_OF_RELOCATION_ADDEND_32,
			(true, Container::Big,) => Self::SIZE_OF_RELOCATION_ADDEND_64,
			(false, Container::Little,) => Self::SIZE_OF_RELOCATION_32,
			(false, Container::Big,) => Self::SIZE_OF_RELOCATION_64,
		}
	}

	fn iter(&self,) -> RelocationIterator {
		self.into_iter()
	}
}

impl IntoIterator for &RelocationSection {
	type IntoIter = RelocationIterator;
	type Item = <RelocationIterator as Iterator>::Item;

	fn into_iter(self,) -> Self::IntoIter {
		todo!()
	}
}

pub struct RelocationIterator {
	bytes:   Vec<u8,>,
	offset:  usize,
	index:   usize,
	count:   usize,
	context: RelocationContext,
}

impl Iterator for RelocationIterator {
	type Item = Relocation;

	fn next(&mut self,) -> Option<Self::Item,> {
		if self.index >= self.count {
			None
		} else {
			self.index += 1;
			Some(Relocation::parse(&self.bytes, &mut self.offset, &self.context,).unwrap(),)
		}
	}
}

pub type RelocationContext = (bool, Context,);

pub struct Relocation {
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

impl Relocation {
	fn parse(
		bytes: &[u8],
		offset: &mut usize,
		(is_relocation_addrend, context,): &RelocationContext,
	) -> Rslt<Self,> {
		let relocation = match (is_relocation_addrend, &context.container,) {
			(true, Container::Little,) => todo!(),
			(true, Container::Big,) => {
				RelocAddend::parse(bytes, offset,).into()
			},
			(false, Container::Little,) => todo!(),
			(false, Container::Big,) => Reloc::parse(bytes, offset,).into(),
		};
		Ok(relocation,)
	}
}

pub struct RelocAddend {
	pub offset: u64,
	pub info:   u64,
	pub addend: i64,
}

impl RelocAddend {
	fn parse(binary: &[u8], offset: &mut usize,) -> Self {
		let reloc_offset: u64 = read_le_bytes(offset, binary,).unwrap();
		let info: u64 = read_le_bytes(offset, binary,).unwrap();
		let addend: i64 = read_le_bytes(offset, binary,).unwrap();
		Self { offset: reloc_offset, info, addend, }
	}
}

impl From<RelocAddend,> for Relocation {
	fn from(value: RelocAddend,) -> Self {
		Self {
			offset:       value.offset,
			addend:       Some(value.addend,),
			symbol_index: relocation_symbol_index(value.info,) as usize,
			ty:           relocation_type(value.info,),
		}
	}
}

fn relocation_symbol_index(info: u64,) -> u32 {
	(info >> 32) as u32
}

fn relocation_type(info: u64,) -> u32 {
	(info & 0xffff_ffff) as u32
}

// fn relocation_info(symbol: u64, ty: u64,) -> u64 {
// 	(symbol << 32) + ty
// }

pub struct Reloc {
	pub offset: u64,
	pub info:   u64,
}

impl Reloc {
	fn parse(binary: &[u8], offset: &mut usize,) -> Self {
		let reloc_offset: u64 = read_le_bytes(offset, binary,).unwrap();
		let info: u64 = read_le_bytes(offset, binary,).unwrap();
		Self { offset: reloc_offset, info, }
	}
}

impl From<Reloc,> for Relocation {
	fn from(value: Reloc,) -> Self {
		Self {
			offset:       value.offset,
			addend:       None,
			symbol_index: relocation_symbol_index(value.info,) as usize,
			ty:           relocation_type(value.info,),
		}
	}
}

pub struct SymbolVersionSection {
	pub bytes:   Vec<u8,>,
	pub context: Context,
}

impl SymbolVersionSection {
	fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &Context,
	) -> Rslt<Option<Self,>, EfiParseError,> {
		let (offset, size,) = if let Some(section_header,) = section_headers
			.iter()
			.find(|section_header| section_header.ty == SHT_GNU_VERSYM,)
		{
			(section_header.offset as usize, section_header.size as usize,)
		} else {
			return Ok(None,);
		};
		let bytes = binary[offset..offset + size].to_vec();
		Ok(Some(Self { bytes, context: ctx.clone(), },),)
	}
}

pub struct VersionDefinitionSection {
	pub bytes:   Vec<u8,>,
	pub count:   usize,
	pub context: Context,
}

impl VersionDefinitionSection {
	fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &Context,
	) -> Rslt<Option<Self,>, EfiParseError,> {
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
			return Ok(None,);
		};
		let bytes = binary[offset..offset + size].to_vec();
		Ok(Some(Self { bytes, count, context: ctx.clone(), },),)
	}
}

pub struct VersionNeededSection {
	pub bytes:   Vec<u8,>,
	pub count:   usize,
	pub context: Context,
}

impl VersionNeededSection {
	fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &Context,
	) -> Rslt<Option<Self,>, EfiParseError,> {
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
			return Ok(None,);
		};
		let bytes = binary[offset..offset + size].to_vec();
		Ok(Some(Self { bytes, count, context: ctx.clone(), },),)
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

impl PrimitiveInteger for u8 {}
impl PrimitiveInteger for u16 {}
impl PrimitiveInteger for u32 {}
impl PrimitiveInteger for u64 {}
impl PrimitiveInteger for u128 {}
impl PrimitiveInteger for usize {}
impl PrimitiveInteger for i8 {}
impl PrimitiveInteger for i16 {}
impl PrimitiveInteger for i32 {}
impl PrimitiveInteger for i64 {}
impl PrimitiveInteger for i128 {}
impl PrimitiveInteger for isize {}

impl Integer<u8,> for u8 {
	fn cast_int(self,) -> u8 {
		self
	}
}

impl Integer<u16,> for u8 {
	fn cast_int(self,) -> u16 {
		self as u16
	}
}

impl Integer<u32,> for u8 {
	fn cast_int(self,) -> u32 {
		self as u32
	}
}

impl Integer<u64,> for u8 {
	fn cast_int(self,) -> u64 {
		self as u64
	}
}

impl Integer<u128,> for u8 {
	fn cast_int(self,) -> u128 {
		self as u128
	}
}

impl Integer<usize,> for u8 {
	fn cast_int(self,) -> usize {
		self as usize
	}
}

impl Integer<i8,> for u8 {
	fn cast_int(self,) -> i8 {
		self as i8
	}
}

impl Integer<i16,> for u8 {
	fn cast_int(self,) -> i16 {
		self as i16
	}
}

impl Integer<i32,> for u8 {
	fn cast_int(self,) -> i32 {
		self as i32
	}
}

impl Integer<i64,> for u8 {
	fn cast_int(self,) -> i64 {
		self as i64
	}
}

impl Integer<i128,> for u8 {
	fn cast_int(self,) -> i128 {
		self as i128
	}
}

impl Integer<isize,> for u8 {
	fn cast_int(self,) -> isize {
		self as isize
	}
}

trait AsInt<T: PrimitiveInteger,> {
	fn as_int(&self,) -> T;
}

impl AsInt<u8,> for &[u8] {
	fn as_int(&self,) -> u8 {
		*self.first().unwrap()
	}
}

impl AsInt<u16,> for &[u8] {
	fn as_int(&self,) -> u16 {
		// unsafe { *(&self[..2] as *const _ as *const u16) }
		let mut rslt = 0;
		for i in (0..size_of::<u16,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u16;
		}

		rslt
	}
}

impl AsInt<u32,> for &[u8] {
	fn as_int(&self,) -> u32 {
		// unsafe { *(&self[..4] as *const _ as *const u32) }
		let mut rslt = 0;
		for i in (0..size_of::<u32,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u32;
		}

		rslt
	}
}

impl AsInt<u64,> for &[u8] {
	fn as_int(&self,) -> u64 {
		// unsafe { *(&self[..8] as *const _ as *const u64) }
		let mut rslt = 0;
		for i in (0..size_of::<u64,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u64;
		}

		rslt
	}
}

impl AsInt<u128,> for &[u8] {
	fn as_int(&self,) -> u128 {
		// unsafe { *(&self[..16] as *const _ as *const u128) }
		let mut rslt = 0;
		for i in (0..size_of::<u128,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as u128;
		}

		rslt
	}
}

impl AsInt<usize,> for &[u8] {
	fn as_int(&self,) -> usize {
		// unsafe { *(&self[..8] as *const _ as *const usize) }
		let mut rslt = 0;
		for i in (0..size_of::<usize,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as usize;
		}

		rslt
	}
}

impl AsInt<i8,> for &[u8] {
	fn as_int(&self,) -> i8 {
		*self.first().unwrap() as i8
	}
}

impl AsInt<i16,> for &[u8] {
	fn as_int(&self,) -> i16 {
		// unsafe { *(&self[..2] as *const _ as *const u16) }
		let mut rslt = 0;
		for i in (0..size_of::<i16,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i16;
		}

		rslt
	}
}

impl AsInt<i32,> for &[u8] {
	fn as_int(&self,) -> i32 {
		// unsafe { *(&self[..4] as *const _ as *const u32) }
		let mut rslt = 0;
		for i in (0..size_of::<i32,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i32;
		}

		rslt
	}
}

impl AsInt<i64,> for &[u8] {
	fn as_int(&self,) -> i64 {
		// unsafe { *(&self[..8] as *const _ as *const u64) }
		let mut rslt = 0;
		for i in (0..size_of::<i64,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i64;
		}

		rslt
	}
}

impl AsInt<i128,> for &[u8] {
	fn as_int(&self,) -> i128 {
		// unsafe { *(&self[..16] as *const _ as *const u128) }
		let mut rslt = 0;
		for i in (0..size_of::<i128,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as i128;
		}

		rslt
	}
}

impl AsInt<isize,> for &[u8] {
	fn as_int(&self,) -> isize {
		// unsafe { *(&self[..8] as *const _ as *const usize) }
		let mut rslt = 0;
		for i in (0..size_of::<isize,>()).rev() {
			rslt <<= 8;
			rslt |= *self.get(i,).unwrap() as isize;
		}

		rslt
	}
}
