#![feature(log_syntax)]
#![feature(str_as_str)]
#![feature(iter_array_chunks)]
#![feature(associated_type_defaults)]
#![feature(iterator_try_collect)]
#![feature(string_remove_matches)]

pub mod features;
/// Font data processing and bitmap conversion utilities
pub mod font;
pub mod from_path_buf;
/// Trait implementation generation for integer types
pub mod impl_int;
/// UEFI status code parsing from HTML specifications
pub mod status;
/// ELF header parsing and analysis utilities
pub mod test_elf_header_parse;
/// ELF program header parsing utilities
pub mod test_program_headers_parse;
/// Function wrapper generation utilities
pub mod wrapper;
