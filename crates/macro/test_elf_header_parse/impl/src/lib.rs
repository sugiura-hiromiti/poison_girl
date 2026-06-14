mod codegen;
mod readelf;

pub use {
	codegen::{elf_header_info, test_elf_header_parse},
	readelf::{ReadElfH, readelf_h},
};

#[cfg(test)]
pub(crate) use codegen::{
	parse_abi_version, parse_elf_version, parse_endianness, parse_entry,
	parse_file_class, parse_flags, parse_machine, parse_program_header_offset,
	parse_section_header_offset, parse_target_os_abi, parse_ty, parse_version,
};
#[cfg(test)] pub(crate) use readelf::Property;

#[cfg(test)] mod tests;
