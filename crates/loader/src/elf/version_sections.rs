use {
	crate::elf::{
		SectionHeader,
		elf_context::ElfContext,
		section_header::{SHT_GNU_VERDEF, SHT_GNU_VERNEED, SHT_GNU_VERSYM},
	},
	alloc::vec::Vec,
	poison_girl_no_std_error::{PoisonGirlB, X},
};

pub struct SymbolVersionSection
{
	pub bytes:   Vec<u8,>,
	pub context: ElfContext,
}

impl SymbolVersionSection
{
	pub fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &ElfContext,
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
	pub context: ElfContext,
}

impl VersionDefinitionSection
{
	pub fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &ElfContext,
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
	pub context: ElfContext,
}

impl VersionNeededSection
{
	pub fn parse(
		binary: &[u8],
		section_headers: &[SectionHeader],
		ctx: &ElfContext,
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
