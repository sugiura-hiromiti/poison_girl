use super::Context;
use super::read_le_bytes;
use crate::Rslt;
use crate::elf::Container;
use crate::elf::ElfHeader;
use oso_error::loader::EfiParseError;
use oso_error::oso_err;

pub fn gnu_hash_len(
	binary: &[u8],
	mut offset: usize,
	context: &Context,
) -> Rslt<usize, EfiParseError,> {
	let buckets_count =
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize;
	let min_chain =
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize;
	let bloom_size =
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize;
	if buckets_count == 0 || min_chain == 0 || bloom_size == 0 {
		return Err(oso_err!(EfiParseError::InvalidGnuHash {
			buckets_count,
			min_chain,
			bloom_size
		}),);
	}

	// find the last bucket
	let buckets_offset = offset
		+ 4 + bloom_size
		* if context.container == Container::Big { 8 } else { 4 };
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
		return Ok(0,);
	}

	// find the last chain within the bucket
	let mut chain_offset =
		buckets_offset + buckets_count * 4 + (max_chain - min_chain) * 4;
	loop {
		let hash =
			read_le_bytes::<u32,>(&mut chain_offset, binary,).unwrap() as usize;
		max_chain += 1;
		if hash & 1 != 0 {
			return Ok(max_chain,);
		}
	}
}

pub fn hash_len(
	binary: &[u8],
	mut offset: usize,
	machine: u16,
	context: &Context,
) -> Rslt<usize, EfiParseError,> {
	offset = offset.saturating_add(4,);
	let nchain = if (machine == ElfHeader::EM_FAKE_ALPHA
		|| machine == ElfHeader::EM_S390)
		&& context.container == Container::Big
	{
		read_le_bytes::<u64,>(&mut offset, binary,).unwrap() as usize
	} else {
		read_le_bytes::<u32,>(&mut offset, binary,).unwrap() as usize
	};
	Ok(nchain,)
}
