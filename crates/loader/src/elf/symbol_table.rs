use {
	crate::elf::{
		elf_container_size::ElfContainerSize, elf_context::ElfContext,
	},
	alloc::vec::Vec,
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, X, Y, poison_girl_err,
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

		if size == 0 {
			return X(SymbolTable {
				bytes: Vec::new(),
				count,
				ctx: context.clone(),
				start: offset,
				end: offset,
			},);
		}

		let Some(end,) = offset.checked_add(size,) else {
			return Y(poison_girl_err!(ElfParseError::SizeOverflow {
				stage:    ElfParseStage::Dynamic,
				name:     0,
				expected: binary.len() as u64,
				base:     offset as u64,
				size:     size as u64,
			}),);
		};
		if end > binary.len() {
			return Y(poison_girl_err!(ElfParseError::SizeOverflow {
				stage:    ElfParseStage::Dynamic,
				name:     0,
				expected: binary.len() as u64,
				base:     offset as u64,
				size:     size as u64,
			}),);
		}

		let bytes = binary[offset..end].to_vec();

		X(SymbolTable {
			bytes,
			count,
			ctx: context.clone(),
			start: offset,
			end,
		},)
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
		alloc::vec::Vec,
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	fn ctx64() -> ElfContext
	{
		ElfContext { container: ElfContainerSize::Big, ..Default::default() }
	}

	#[test]
	fn parses_64_bit_symbols_from_expected_byte_range() -> PoisonGirlTestB
	{
		let ctx = ctx64();
		let mut binary = alloc::vec![0xaa; 3];
		let symbols = (0..SymbolTable::SIZE_OF_SYMBOL_64 * 2)
			.map(|byte| byte as u8,)
			.collect::<Vec<_,>>();
		binary.extend_from_slice(&symbols,);
		binary.extend_from_slice(&[0xbb; 4],);

		let table = SymbolTable::parse(&binary, 3, 2, &ctx,)?;

		assert_eq!(table.bytes, symbols);
		assert_eq!(table.count, 2);
		assert_eq!(table.start, 3);
		assert_eq!(table.end, 3 + SymbolTable::SIZE_OF_SYMBOL_64 * 2);
		assert!(matches!(table.ctx.container, ElfContainerSize::Big));
		success!()
	}

	#[test]
	fn zero_count_returns_empty_table_without_reading_bytes() -> PoisonGirlTestB
	{
		let ctx = ctx64();

		let table = SymbolTable::parse(&[], 123, 0, &ctx,)?;

		assert!(table.bytes.is_empty());
		assert_eq!(table.count, 0);
		assert_eq!(table.start, 123);
		assert_eq!(table.end, 123);
		success!()
	}

	#[test]
	fn count_multiplication_overflow_returns_error()
	{
		let ctx = ctx64();
		let count = usize::MAX / SymbolTable::SIZE_OF_SYMBOL_64 + 1;

		assert!(matches!(SymbolTable::parse(&[], 0, count, &ctx,), Y(_)));
	}

	#[test]
	fn parse_rejects_ranges_outside_binary()
	{
		let ctx = ctx64();
		let binary = alloc::vec![0; SymbolTable::SIZE_OF_SYMBOL_64 - 1];

		assert!(matches!(SymbolTable::parse(&binary, 0, 1, &ctx,), Y(_)));
		assert!(matches!(
			SymbolTable::parse(&binary, usize::MAX, 1, &ctx,),
			Y(_)
		));
	}
}
