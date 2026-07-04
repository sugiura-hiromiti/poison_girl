use {
	crate::elf::{
		ProgramHeader,
		dynamic::{dynamic::Dynamic, dynmc::Dyn},
		vm_to_offset,
	},
	poison_girl_no_std_error::{
		ElfParseError, PoisonGirlB, X, Y, poison_girl_err,
	},
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
	pub fn update(
		&mut self,
		phdrs: &[ProgramHeader],
		dynamic: &Dyn,
	) -> PoisonGirlB<(),>
	{
		match dynamic.tag {
			Dynamic::DT_RELA => {
				self.relocation_addend =
					required_vm_offset(phdrs, dynamic,)? as usize
			}, // .rela.dyn
			Dynamic::DT_RELASZ => {
				self.relocation_addend_size = dynamic.val as usize
			},
			Dynamic::DT_RELAENT => self.relocation_addend_entry = dynamic.val,
			Dynamic::DT_RELACOUNT => {
				self.relocation_addend_entry_count = dynamic.val as usize
			},
			Dynamic::DT_REL => {
				self.relocation = required_vm_offset(phdrs, dynamic,)? as usize
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
					required_vm_offset(phdrs, dynamic,)? as usize
			},
			Dynamic::DT_STRSZ => self.string_table_size = dynamic.val as usize,
			Dynamic::DT_SYMTAB => {
				self.symbol_table =
					required_vm_offset(phdrs, dynamic,)? as usize
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
					required_vm_offset(phdrs, dynamic,)? as usize
			}, /* .rela.plt */
			Dynamic::DT_VERDEF => {
				self.virsion_definition_table_address =
					required_vm_offset(phdrs, dynamic,)?
			},
			Dynamic::DT_VERDEFNUM => {
				self.version_definition_count = dynamic.val
			},
			Dynamic::DT_VERNEED => {
				self.version_need_table_address =
					required_vm_offset(phdrs, dynamic,)?
			},
			Dynamic::DT_VERNEEDNUM => self.version_need_count = dynamic.val,
			Dynamic::DT_VERSYM => {
				self.version_symbol_table_address =
					required_vm_offset(phdrs, dynamic,)?
			},
			Dynamic::DT_INIT => {
				self.init_fn_address = required_vm_offset(phdrs, dynamic,)?
			},
			Dynamic::DT_FINI => {
				self.finalization_fn_address =
					required_vm_offset(phdrs, dynamic,)?
			},
			Dynamic::DT_INIT_ARRAY => {
				self.init_fn_array_address =
					required_vm_offset(phdrs, dynamic,)?
			},
			Dynamic::DT_INIT_ARRAYSZ => {
				self.init_fn_array_len = dynamic.val as usize
			},
			Dynamic::DT_FINI_ARRAY => {
				self.finalization_fn_array_address =
					required_vm_offset(phdrs, dynamic,)?
			},
			Dynamic::DT_FINI_ARRAYSZ => {
				self.finalization_fn_array_len = dynamic.val as usize
			},
			Dynamic::DT_NEEDED => self.required_shared_lib_count += 1,
			Dynamic::DT_FLAGS => self.flags = dynamic.val,
			Dynamic::DT_FLAGS_1 => self.extended_flags = dynamic.val,
			Dynamic::DT_SONAME => {
				self.shared_object_name_offset = dynamic.val as usize
			},
			Dynamic::DT_TEXTREL => self.text_section_relocation = true,
			_ => (),
		}
		X((),)
	}
}

fn required_vm_offset(
	phdrs: &[ProgramHeader],
	dynamic: &Dyn,
) -> PoisonGirlB<u64,>
{
	match vm_to_offset(phdrs, dynamic.val,) {
		Some(offset,) => X(offset,),
		None => Y(poison_girl_err!(ElfParseError::InvalidDynamicAddress {
			tag:     dynamic.tag,
			address: dynamic.val,
		}),),
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::elf::program_header::ProgramHeaderType,
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::{ElfParseError, PoisonGirlErrorKind, Y},
	};

	#[test]
	fn invalid_required_dynamic_address_returns_error()
	{
		let mut info = DynamicInfo::default();
		let phdrs = [ProgramHeader {
			ty:               ProgramHeaderType::Load,
			flags:            0,
			offset:           0x40,
			virtual_address:  0x1000,
			physical_address: 0x1000,
			file_size:        0x100,
			memory_size:      0x100,
			align:            0,
		},];
		let dynamic = Dyn { tag: Dynamic::DT_STRTAB, val: 0x3000, };

		let Y(err,) = info.update(&phdrs, &dynamic,) else {
			panic!("dynamic info accepted an unmapped dynamic address");
		};

		assert!(matches!(
			err.kind(),
			PoisonGirlErrorKind::ElfParse(
				ElfParseError::InvalidDynamicAddress {
					tag:     Dynamic::DT_STRTAB,
					address: 0x3000,
				}
			)
		));
	}

	#[test]
	fn update_maps_dynamic_addresses_and_retains_counts_flags()
	-> PoisonGirlTestB
	{
		let mut info = DynamicInfo::default();
		let phdrs = [ProgramHeader {
			ty:               ProgramHeaderType::Load,
			flags:            0,
			offset:           0x40,
			virtual_address:  0x1000,
			physical_address: 0x1000,
			file_size:        0x400,
			memory_size:      0x400,
			align:            0,
		},];

		for dynamic in [
			Dyn { tag: Dynamic::DT_STRTAB, val: 0x1100, },
			Dyn { tag: Dynamic::DT_STRSZ, val: 0x30, },
			Dyn { tag: Dynamic::DT_SYMTAB, val: 0x1200, },
			Dyn { tag: Dynamic::DT_SYMENT, val: 0x18, },
			Dyn { tag: Dynamic::DT_VERNEEDNUM, val: 7, },
			Dyn { tag: Dynamic::DT_NEEDED, val: 0x01, },
			Dyn { tag: Dynamic::DT_NEEDED, val: 0x10, },
			Dyn { tag: Dynamic::DT_SONAME, val: 0x20, },
			Dyn { tag: Dynamic::DT_FLAGS_1, val: Dynamic::DF_EXTEND_PIE, },
			Dyn { tag: Dynamic::DT_TEXTREL, val: 0, },
		] {
			info.update(&phdrs, &dynamic,)?;
		}

		assert_eq!(info.string_table_address, 0x140);
		assert_eq!(info.string_table_size, 0x30);
		assert_eq!(info.symbol_table, 0x240);
		assert_eq!(info.symbol_table_entry, 0x18);
		assert_eq!(info.version_need_count, 7);
		assert_eq!(info.required_shared_lib_count, 2);
		assert_eq!(info.shared_object_name_offset, 0x20);
		assert_eq!(info.extended_flags, Dynamic::DF_EXTEND_PIE);
		assert!(info.text_section_relocation);
		success!()
	}
}
