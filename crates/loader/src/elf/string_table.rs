use {
	crate::elf::string_context::StringContext,
	alloc::{
		string::{String, ToString},
		vec::Vec,
	},
	poison_girl_no_std_error::{
		ElfParseError, ElfParseStage, PoisonGirlB, X, Y, poison_girl_err,
	},
};

#[derive_const(Default)]
pub struct StringTable
{
	pub delimitor: StringContext,
	pub bytes:     Vec<u8,>,
	pub strings:   Vec<(usize, String,),>,
}

impl StringTable
{
	/// # Params
	///
	/// - bytes
	///
	/// bytes expected to be entire elf file
	pub fn parse(
		binary: &[u8],
		offset: usize,
		len: usize,
		delimiter: u8,
	) -> PoisonGirlB<Self,>
	{
		let (end, overflow,) = offset.overflowing_add(len,);
		if overflow || end > binary.len() {
			return Y(poison_girl_err!(ElfParseError::SizeOverflow {
				stage:    ElfParseStage::StringTable,
				name:     0,
				expected: binary.len() as u64,
				base:     offset as u64,
				size:     len as u64,
			}),);
		}

		let mut rslt =
			Self::from_slice(&binary[offset..offset + len], delimiter,);
		let mut i = 0;
		while i < rslt.bytes.len() {
			let s = rslt.delimitor.read_bytes(&binary[i..],)?.to_vec();
			let s = String::from_utf8_lossy_owned(s,);
			let len = s.len();
			rslt.strings.push((i, s,),);
			i += len + 1;
		}

		X(rslt,)
	}

	fn from_slice(bytes: &[u8], delimiter: u8,) -> Self
	{
		Self {
			delimitor: StringContext::Delimiter(delimiter,),
			bytes:     bytes.to_vec(),
			strings:   alloc::vec![],
		}
	}

	pub fn get_at(&self, offset: usize,) -> Option<String,>
	{
		match self.strings.binary_search_by_key(&offset, |(key, _value,)| *key,)
		{
			Ok(index,) => Some(self.strings[index].1.clone(),),
			Err(index,) => {
				if index == 0 {
					return None;
				}
				let (string_begin_offset, entire_string,) =
					&self.strings[index - 1];
				entire_string
					.get(offset - string_begin_offset..,)
					.map(|s| s.to_string(),)
			},
		}
	}
}
