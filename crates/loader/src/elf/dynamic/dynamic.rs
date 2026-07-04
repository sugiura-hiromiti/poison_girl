use {
	crate::elf::{
		ProgramHeader, ProgramHeaderType,
		dynamic::{dynamic_info::DynamicInfo, dynmc::Dyn},
		elf_container_size::ElfContainerSize,
		elf_context::ElfContext,
		hash::{gnu_hash_len, hash_len},
		relocation::RelocationSection,
		string_table::StringTable,
		symbol_table::SymbolTable,
	},
	alloc::{string::String, vec::Vec},
	core::cmp,
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, X, Y, poison_girl_err,
	},
};

pub mod dynamic_consts;

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
	pub fn parse(
		binary: &[u8],
		program_headers: &Vec<ProgramHeader,>,
	) -> PoisonGirlB<Self,>
	{
		for program_header in program_headers {
			if program_header.ty == ProgramHeaderType::Dynamic {
				let offset = program_header.offset as usize;
				let file_size = program_header.file_size as usize;
				let Some(end,) = offset.checked_add(file_size,) else {
					return Y(poison_girl_err!(ElfParseError::SizeOverflow {
						stage:    ElfParseStage::Dynamic,
						name:     0,
						expected: binary.len() as u64,
						base:     offset as u64,
						size:     file_size as u64,
					}),);
				};
				if end > binary.len() {
					return Y(poison_girl_err!(ElfParseError::SizeOverflow {
						stage:    ElfParseStage::Dynamic,
						name:     0,
						expected: binary.len() as u64,
						base:     offset as u64,
						size:     file_size as u64,
					}),);
				}
				let bytes =
					if file_size > 0 { &binary[offset..end] } else { &[] };
				let size = Dyn::size_of(&ElfContext {
					container: ElfContainerSize::Big,
					..Default::default()
				},);
				let count = file_size / size;
				let mut dyns = Vec::with_capacity(count,);
				let offset = &mut 0;
				for _ in 0..count {
					let dynamic = Dyn::parse(bytes, offset,)?;
					let tag = dynamic.tag;
					dyns.push(dynamic,);
					if tag == Self::DT_NULL {
						break;
					}
				}

				let mut info = DynamicInfo::default();
				for dynamic in &dyns {
					info.update(program_headers, dynamic,)?;
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

		let count = inner.dyns.len().min(inner.info.required_shared_lib_count,);
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

	pub(in crate::elf) fn is_position_independent_executable(&self,) -> bool
	{
		let Some(ref inner,) = self.0 else {
			return Self::IS_POSITION_INDEPENDENT_EXECUTABLE;
		};

		inner.info.extended_flags & Self::DF_EXTEND_PIE != 0
	}

	pub(in crate::elf) fn dynamic_string_table(
		&self,
		binary: &[u8],
	) -> PoisonGirlB<StringTable,>
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

	pub(in crate::elf) fn shared_object_name(
		&self,
		dyn_str_table: &StringTable,
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

	pub(in crate::elf) fn libraries(
		&self,
		dyn_str_table: &StringTable,
	) -> Vec<String,>
	{
		if self.0.is_some() {
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

	pub(in crate::elf) fn runtime_search_path_deprecated(
		&self,
		dyn_str_table: &StringTable,
	) -> Vec<String,>
	{
		self.runtime_search_path_detection(dyn_str_table, Self::DT_RPATH,)
	}

	pub(in crate::elf) fn runtime_search_path(
		&self,
		dyn_str_table: &StringTable,
	) -> Vec<String,>
	{
		self.runtime_search_path_detection(dyn_str_table, Self::DT_RUNPATH,)
	}

	/// # Return
	/// (dynamic_relocation_with_addend, dynamic_relocation,
	/// procedure_linkage_table_relocation)
	fn dynamic_relocations(
		&self,
		ctx: &ElfContext,
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

	pub(in crate::elf) fn dynamic_relocations_and_symbol_table(
		&self,
		ctx: &ElfContext,
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
				let X(relocation,) = relocation else {
					return count;
				};
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

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::elf::{
			program_header::{ProgramHeader, ProgramHeaderType},
			test_helpers,
		},
		alloc::vec,
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::{
			ElfParseError, ElfParseStage, PoisonGirlErrorKind, Y,
		},
	};

	fn dynamic_program_header(offset: u64, file_size: u64,) -> ProgramHeader
	{
		ProgramHeader {
			ty: ProgramHeaderType::Dynamic,
			flags: 0,
			offset,
			virtual_address: 0,
			physical_address: 0,
			file_size,
			memory_size: file_size,
			align: 8,
		}
	}

	#[test]
	fn dt_null_stops_dynamic_parsing() -> PoisonGirlTestB
	{
		let mut binary = test_helpers::dynamic_entry(Dynamic::DT_NULL, 0,);
		binary.extend_from_slice(&test_helpers::dynamic_entry(
			Dynamic::DT_STRTAB,
			0xdead,
		),);
		let program_headers =
			vec![dynamic_program_header(0, binary.len() as u64,)];

		let dynamic = Dynamic::parse(&binary, &program_headers,)?;
		let Some(inner,) = dynamic.as_ref().as_ref() else {
			return PoisonGirlTestB::y("dynamic section was not parsed",);
		};

		assert_eq!(inner.dyns.len(), 1);
		assert_eq!(inner.dyns[0].tag, Dynamic::DT_NULL);
		success!()
	}

	#[test]
	fn dynamic_segment_outside_binary_returns_error()
	{
		let binary = vec![0; 16];
		let program_headers = vec![dynamic_program_header(8, 16,)];

		let Y(err,) = Dynamic::parse(&binary, &program_headers,) else {
			panic!("dynamic parser accepted segment outside binary");
		};

		assert!(matches!(
			err.kind(),
			PoisonGirlErrorKind::ElfParse(ElfParseError::SizeOverflow {
				stage: ElfParseStage::Dynamic,
				expected: 16,
				base: 8,
				size: 16,
				..
			})
		));
	}
}
