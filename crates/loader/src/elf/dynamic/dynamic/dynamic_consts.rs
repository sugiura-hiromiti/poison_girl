use {
	crate::elf::{
		dynamic::dynamic::Dynamic, relocation::RelocationSection,
		string_table::StringTable, symbol_table::SymbolTable,
	},
	alloc::{string::String, vec::Vec},
};

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
	pub const DYNAMIC_RELOCATION: RelocationSection =
		RelocationSection::default();
	/// デフォルト値
	pub const DYNAMIC_RELOCATION_WITH_ADDEND: RelocationSection =
		RelocationSection::default();
	/// デフォルト値
	pub const DYNAMIC_STRING_TABLE: StringTable = StringTable::default();
	/// デフォルト値
	pub const DYNAMIC_SYMBOL_TABLE: SymbolTable = SymbolTable::default();
	/// デフォルト値
	pub const IS_POSITION_INDEPENDENT_EXECUTABLE: bool = false;
	/// デフォルト値
	pub const LIBRARIES: Vec<String,> = alloc::vec![];
	/// デフォルト値
	pub const PROCEDURE_LINKAGE_TABLE_RELOCATION: RelocationSection =
		RelocationSection::default();
	/// デフォルト値
	pub const RUNTIME_SEARCH_PATH: Vec<String,> = alloc::vec![];
	/// デフォルト値
	pub const RUNTIME_SEARCH_PATH_DEPRECATED: Vec<String,> = alloc::vec![];
	/// デフォルト値
	pub const SHARED_OBJECT_NAME: Option<String,> = None;
}
