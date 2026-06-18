use {
	crate::elf::{
		elf_container_size::ElfContainerSize, elf_context::ElfContext,
		read_le_bytes_or,
	},
	alloc::vec::Vec,
	poison_girl_no_std_error::{ElfParseStage, PoisonGirlB, X},
};

#[derive_const(Default)]
pub struct RelocationSection
{
	pub bytes:   Vec<u8,>,
	pub count:   usize,
	pub context: RelocationContext,
	pub start:   usize,
	pub end:     usize,
}

impl RelocationSection
{
	const SIZE_OF_RELOCATION_32: usize = 8;
	const SIZE_OF_RELOCATION_64: usize = 16;
	const SIZE_OF_RELOCATION_ADDEND_32: usize = 12;
	const SIZE_OF_RELOCATION_ADDEND_64: usize = 24;

	pub fn parse(
		binary: &[u8],
		offset: usize,
		size: usize,
		is_addend: bool,
		ctx: &ElfContext,
	) -> PoisonGirlB<Self,>
	{
		let bytes =
			if size != 0 { &binary[offset..offset + size] } else { &[] }
				.to_vec();

		X(Self {
			bytes,
			count: size / Self::size(is_addend, ctx,),
			context: RelocationContext(is_addend, ctx.clone(),),
			start: offset,
			end: offset + size,
		},)
	}

	fn size(
		is_relocation_addrend: bool,
		ElfContext { container, .. }: &ElfContext,
	) -> usize
	{
		match (is_relocation_addrend, container,) {
			(true, ElfContainerSize::Little,) => {
				Self::SIZE_OF_RELOCATION_ADDEND_32
			},
			(true, ElfContainerSize::Big,) => {
				Self::SIZE_OF_RELOCATION_ADDEND_64
			},
			(false, ElfContainerSize::Little,) => Self::SIZE_OF_RELOCATION_32,
			(false, ElfContainerSize::Big,) => Self::SIZE_OF_RELOCATION_64,
		}
	}

	pub fn iter(&self,) -> RelocationIterator
	{
		self.into_iter()
	}
}

impl IntoIterator for &RelocationSection
{
	type IntoIter = RelocationIterator;
	type Item = <RelocationIterator as Iterator>::Item;

	fn into_iter(self,) -> Self::IntoIter
	{
		RelocationIterator {
			bytes:   self.bytes.clone(),
			offset:  0,
			index:   0,
			count:   self.count,
			context: RelocationContext(self.context.0, self.context.1.clone(),),
		}
	}
}

pub struct RelocationIterator
{
	bytes:   Vec<u8,>,
	offset:  usize,
	index:   usize,
	count:   usize,
	context: RelocationContext,
}

impl Iterator for RelocationIterator
{
	type Item = PoisonGirlB<Relocation,>;

	fn next(&mut self,) -> Option<Self::Item,>
	{
		if self.index >= self.count {
			None
		} else {
			self.index += 1;
			Some(Relocation::parse(
				&self.bytes,
				&mut self.offset,
				&self.context,
			),)
		}
	}
}

#[derive_const(Default)]
pub struct RelocationContext(bool, ElfContext,);

pub struct Relocation
{
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

impl Relocation
{
	fn parse(
		bytes: &[u8],
		offset: &mut usize,
		RelocationContext(is_relocation_addrend, context,): &RelocationContext,
	) -> PoisonGirlB<Self,>
	{
		let relocation = match (is_relocation_addrend, &context.container,) {
			(true, ElfContainerSize::Little,) => todo!(),
			(true, ElfContainerSize::Big,) => {
				RelocAddend::parse(bytes, offset,)?.into()
			},
			(false, ElfContainerSize::Little,) => todo!(),
			(false, ElfContainerSize::Big,) => {
				Reloc::parse(bytes, offset,)?.into()
			},
		};
		X(relocation,)
	}
}

pub struct RelocAddend
{
	pub offset: u64,
	pub info:   u64,
	pub addend: i64,
}

impl RelocAddend
{
	fn parse(binary: &[u8], offset: &mut usize,) -> PoisonGirlB<Self,>
	{
		let reloc_offset: u64 = read_le_bytes_or(
			offset,
			binary,
			"relocation addend offset",
			ElfParseStage::Relocation,
		)?;
		let info: u64 = read_le_bytes_or(
			offset,
			binary,
			"relocation addend info",
			ElfParseStage::Relocation,
		)?;
		let addend: i64 = read_le_bytes_or(
			offset,
			binary,
			"relocation addend",
			ElfParseStage::Relocation,
		)?;
		X(Self { offset: reloc_offset, info, addend, },)
	}
}

impl From<RelocAddend,> for Relocation
{
	fn from(value: RelocAddend,) -> Self
	{
		Self {
			offset:       value.offset,
			addend:       Some(value.addend,),
			symbol_index: relocation_symbol_index(value.info,) as usize,
			ty:           relocation_type(value.info,),
		}
	}
}

fn relocation_symbol_index(info: u64,) -> u32
{
	(info >> 32) as u32
}

fn relocation_type(info: u64,) -> u32
{
	(info & 0xffff_ffff) as u32
}

// fn relocation_info(symbol: u64, ty: u64,) -> u64 {
// 	(symbol << 32) + ty
// }

pub struct Reloc
{
	pub offset: u64,
	pub info:   u64,
}

impl Reloc
{
	fn parse(binary: &[u8], offset: &mut usize,) -> PoisonGirlB<Self,>
	{
		let reloc_offset: u64 = read_le_bytes_or(
			offset,
			binary,
			"relocation offset",
			ElfParseStage::Relocation,
		)?;
		let info: u64 = read_le_bytes_or(
			offset,
			binary,
			"relocation info",
			ElfParseStage::Relocation,
		)?;
		X(Self { offset: reloc_offset, info, },)
	}
}

impl From<Reloc,> for Relocation
{
	fn from(value: Reloc,) -> Self
	{
		Self {
			offset:       value.offset,
			addend:       None,
			symbol_index: relocation_symbol_index(value.info,) as usize,
			ty:           relocation_type(value.info,),
		}
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::elf::{
			elf_container_size::ElfContainerSize, elf_context::ElfContext,
		},
		poison_girl_dev_test::{PoisonGirlTestB, success},
	};

	#[test]
	fn iterates_64_bit_relocations_with_and_without_addends() -> PoisonGirlTestB
	{
		let ctx = ElfContext {
			container: ElfContainerSize::Big,
			..Default::default()
		};

		let mut rela_binary = alloc::vec![0xaa; 4];
		rela_binary.extend_from_slice(&0x1010_u64.to_le_bytes(),);
		rela_binary.extend_from_slice(&((3_u64 << 32) | 7).to_le_bytes(),);
		rela_binary.extend_from_slice(&(-5_i64).to_le_bytes(),);
		let rela = RelocationSection::parse(
			&rela_binary,
			4,
			RelocationSection::SIZE_OF_RELOCATION_ADDEND_64,
			true,
			&ctx,
		)?;
		let mut rela_iter = rela.iter();
		let relocation = rela_iter.next().expect("missing rela entry",)?;

		assert_eq!(rela.count, 1);
		assert_eq!(rela.start, 4);
		assert_eq!(rela.end, 28);
		assert_eq!(relocation.offset, 0x1010);
		assert_eq!(relocation.addend, Some(-5));
		assert_eq!(relocation.symbol_index, 3);
		assert_eq!(relocation.ty, 7);
		assert!(rela_iter.next().is_none());

		let mut rel_binary = alloc::vec![0xbb; 2];
		rel_binary.extend_from_slice(&0x2020_u64.to_le_bytes(),);
		rel_binary.extend_from_slice(&((4_u64 << 32) | 9).to_le_bytes(),);
		let rel = RelocationSection::parse(
			&rel_binary,
			2,
			RelocationSection::SIZE_OF_RELOCATION_64,
			false,
			&ctx,
		)?;
		let mut rel_iter = rel.iter();
		let relocation = rel_iter.next().expect("missing rel entry",)?;

		assert_eq!(rel.count, 1);
		assert_eq!(rel.start, 2);
		assert_eq!(rel.end, 18);
		assert_eq!(relocation.offset, 0x2020);
		assert_eq!(relocation.addend, None);
		assert_eq!(relocation.symbol_index, 4);
		assert_eq!(relocation.ty, 9);
		assert!(rel_iter.next().is_none());
		success!()
	}
}
