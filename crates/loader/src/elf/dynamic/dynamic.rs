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
	poison_girl_no_std_error::{PoisonGirlB, X},
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
				let bytes = if file_size > 0 {
					&binary[offset..offset + file_size]
				} else {
					&[]
				};
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

	pub(in crate::elf) fn is_position_independent_executable(&self,) -> bool
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
