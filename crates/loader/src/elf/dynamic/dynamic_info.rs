use crate::elf::{
	ProgramHeader,
	dynamic::{dynamic::Dynamic, dynmc::Dyn},
	vm_to_offset,
};

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
