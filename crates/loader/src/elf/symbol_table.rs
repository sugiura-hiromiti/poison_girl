use {
	crate::elf::{
		elf_container_size::ElfContainerSize, elf_context::ElfContext,
	},
	alloc::vec::Vec,
	poison_girl_no_std_error::{
		ElfParseError, PoisonGirlB, X, poison_girl_err,
	},
};

#[derive_const(Default)]
pub struct SymbolTable
{
	pub bytes: Vec<u8,>,
	pub count: usize,
	pub ctx:   ElfContext,
	pub start: usize,
	pub end:   usize,
}

impl SymbolTable
{
	/// size of symbol structure in 64bit.
	const SIZE_OF_SYMBOL_64: usize = 4 + 1 + 1 + 2 + 8 + 8;

	pub fn parse(
		binary: &[u8],
		offset: usize,
		count: usize,
		context: &ElfContext,
	) -> PoisonGirlB<Self,>
	{
		let size = count
			.checked_mul(match context.container {
				ElfContainerSize::Little => todo!(),
				ElfContainerSize::Big => Self::SIZE_OF_SYMBOL_64,
			},)
			.ok_or(poison_girl_err!(ElfParseError::TooManySymbolsOffset {
				offset,
				count
			}),)?;

		let bytes = binary[offset..offset + size].to_vec();

		X(SymbolTable {
			bytes,
			count,
			ctx: context.clone(),
			start: offset,
			end: offset + size,
		},)
	}
}
