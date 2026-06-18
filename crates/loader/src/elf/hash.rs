use {
	super::read_le_bytes_or,
	crate::elf::{
		ElfHeader, elf_container_size::ElfContainerSize,
		elf_context::ElfContext,
	},
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, X, Y, poison_girl_err,
	},
};

pub fn gnu_hash_len(
	binary: &[u8],
	mut offset: usize,
	context: &ElfContext,
) -> PoisonGirlB<usize,>
{
	let buckets_count = read_le_bytes_or::<u32,>(
		&mut offset,
		binary,
		"gnu hash bucket count",
		ElfParseStage::Hash,
	)? as usize;
	let min_chain = read_le_bytes_or::<u32,>(
		&mut offset,
		binary,
		"gnu hash minimum chain",
		ElfParseStage::Hash,
	)? as usize;
	let bloom_size = read_le_bytes_or::<u32,>(
		&mut offset,
		binary,
		"gnu hash bloom size",
		ElfParseStage::Hash,
	)? as usize;
	if buckets_count == 0 || min_chain == 0 || bloom_size == 0 {
		return Y(poison_girl_err!(ElfParseError::InvalidGnuHash {
			buckets_count,
			min_chain,
			bloom_size
		}),);
	}

	// find the last bucket
	let buckets_offset = offset
		+ 4 + bloom_size
		* if context.container == ElfContainerSize::Big { 8 } else { 4 };
	let mut max_chain = 0;
	for bucket in 0..buckets_count {
		let chain = read_le_bytes_or::<u32,>(
			&mut (buckets_offset + bucket * 4),
			binary,
			"gnu hash bucket chain",
			ElfParseStage::Hash,
		)? as usize;
		if max_chain < chain {
			max_chain = chain;
		}
	}

	if max_chain < min_chain {
		return X(0,);
	}

	// find the last chain within the bucket
	let mut chain_offset =
		buckets_offset + buckets_count * 4 + (max_chain - min_chain) * 4;
	loop {
		let hash = read_le_bytes_or::<u32,>(
			&mut chain_offset,
			binary,
			"gnu hash chain",
			ElfParseStage::Hash,
		)? as usize;
		max_chain += 1;
		if hash & 1 != 0 {
			return X(max_chain,);
		}
	}
}

pub fn hash_len(
	binary: &[u8],
	mut offset: usize,
	machine: u16,
	context: &ElfContext,
) -> PoisonGirlB<usize,>
{
	offset = offset.saturating_add(4,);
	let nchain = if (machine == ElfHeader::EM_FAKE_ALPHA
		|| machine == ElfHeader::EM_S390)
		&& context.container == ElfContainerSize::Big
	{
		read_le_bytes_or::<u64,>(
			&mut offset,
			binary,
			"sysv hash chain count",
			ElfParseStage::Hash,
		)? as usize
	} else {
		read_le_bytes_or::<u32,>(
			&mut offset,
			binary,
			"sysv hash chain count",
			ElfParseStage::Hash,
		)? as usize
	};
	X(nchain,)
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

	fn push_u32(binary: &mut Vec<u8,>, value: u32,)
	{
		binary.extend_from_slice(&value.to_le_bytes(),);
	}

	fn push_u64(binary: &mut Vec<u8,>, value: u64,)
	{
		binary.extend_from_slice(&value.to_le_bytes(),);
	}

	fn gnu_hash_header(
		buckets_count: u32,
		min_chain: u32,
		bloom_size: u32,
	) -> Vec<u8,>
	{
		let mut binary = Vec::new();
		push_u32(&mut binary, buckets_count,);
		push_u32(&mut binary, min_chain,);
		push_u32(&mut binary, bloom_size,);
		push_u32(&mut binary, 0,);
		binary
	}

	#[test]
	fn gnu_hash_all_buckets_before_min_chain_returns_zero() -> PoisonGirlTestB
	{
		let ctx = ctx64();
		let mut binary = gnu_hash_header(2, 5, 1,);
		push_u64(&mut binary, 0,);
		push_u32(&mut binary, 0,);
		push_u32(&mut binary, 4,);

		let len = gnu_hash_len(&binary, 0, &ctx,)?;

		assert_eq!(len, 0);
		success!()
	}

	#[test]
	fn gnu_hash_counts_until_low_bit_chain_marker() -> PoisonGirlTestB
	{
		let ctx = ctx64();
		let mut binary = gnu_hash_header(1, 2, 1,);
		push_u64(&mut binary, 0,);
		push_u32(&mut binary, 4,);
		push_u32(&mut binary, 0,);
		push_u32(&mut binary, 0,);
		push_u32(&mut binary, 1,);

		let len = gnu_hash_len(&binary, 0, &ctx,)?;

		assert_eq!(len, 5);
		success!()
	}

	#[test]
	fn gnu_hash_rejects_zero_dimensions()
	{
		let ctx = ctx64();

		for (buckets_count, min_chain, bloom_size,) in
			[(0, 1, 1,), (1, 0, 1,), (1, 1, 0,),]
		{
			let binary = gnu_hash_header(buckets_count, min_chain, bloom_size,);

			assert!(matches!(gnu_hash_len(&binary, 0, &ctx,), Y(_)));
		}
	}

	#[test]
	fn sysv_hash_reads_32_bit_chain_count_for_regular_64_bit_context()
	-> PoisonGirlTestB
	{
		let ctx = ctx64();
		let mut binary = Vec::new();
		push_u32(&mut binary, 7,);
		push_u32(&mut binary, 0x1234_5678,);
		push_u32(&mut binary, 0xdead_beef,);

		let len = hash_len(&binary, 0, ElfHeader::EM_AARCH64, &ctx,)?;

		assert_eq!(len, 0x1234_5678);
		success!()
	}

	#[test]
	fn sysv_hash_reads_64_bit_chain_count_for_alpha_and_s390() -> PoisonGirlTestB
	{
		let ctx = ctx64();
		let nchain = 0x0000_0001_0000_0002_u64;
		let mut binary = Vec::new();
		push_u32(&mut binary, 7,);
		push_u64(&mut binary, nchain,);

		for machine in [ElfHeader::EM_FAKE_ALPHA, ElfHeader::EM_S390,] {
			let len = hash_len(&binary, 0, machine, &ctx,)?;

			assert_eq!(len, nchain as usize);
		}
		success!()
	}
}
