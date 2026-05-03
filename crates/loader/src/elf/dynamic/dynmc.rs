use crate::elf::{
	elf_container_size::ElfContainerSize, elf_context::ElfContext,
	read_le_bytes,
};

pub struct Dyn
{
	pub tag: u64,
	pub val: u64,
}

impl Dyn
{
	const SIZE_OF_DYN_32: usize = 8;
	const SIZE_OF_DYN_64: usize = 16;

	pub fn size_of(ElfContext { container, .. }: &ElfContext,) -> usize
	{
		match container {
			ElfContainerSize::Little => Self::SIZE_OF_DYN_32,
			ElfContainerSize::Big => Self::SIZE_OF_DYN_64,
		}
	}

	pub fn parse(bytes: &[u8], offset: &mut usize,) -> Self
	{
		let tag = read_le_bytes(offset, bytes,).unwrap();
		let val = read_le_bytes(offset, bytes,).unwrap();
		Self { tag, val, }
	}
}
