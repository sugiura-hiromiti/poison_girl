use {
	super::read_le_bytes,
	crate::elf::{
		ElfHeader, elf_container_size::ElfContainerSize,
		elf_context::ElfContext,
	},
	poison_girl_no_std_error::{
		ElfParseError, PoisonGirlB, X, Y, poison_girl_err,
	},
};

pub fn gnu_hash_len(
	binary: &[u8],
	mut offset: usize,
	context: &ElfContext,
) -> PoisonGirlB<usize,>
{
	let buckets_count =
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize;
	let min_chain =
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize;
	let bloom_size =
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize;
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
		let chain =
			read_le_bytes::<u32,>(&mut (buckets_offset + bucket * 4), binary,)
				.unwrap() as usize;
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
		let hash =
			read_le_bytes::<u32,>(&mut chain_offset, binary,).unwrap() as usize;
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
		read_le_bytes::<u64,>(&mut offset, binary,).unwrap() as usize
	} else {
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize
	};
	X(nchain,)
}
