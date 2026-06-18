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
			let s = rslt.delimitor.read_bytes(&rslt.bytes[i..],)?.to_vec();
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

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	#[test]
	fn parses_table_at_nonzero_file_offset() -> PoisonGirlTestB
	{
		let prefix = b"ignored\0";
		let table = b"alpha\0beta\0";
		let mut binary = prefix.to_vec();
		binary.extend_from_slice(table,);
		binary.extend_from_slice(b"trailing\0",);

		let string_table =
			StringTable::parse(&binary, prefix.len(), table.len(), 0,)?;

		assert_eq!(string_table.bytes, table);
		assert_eq!(string_table.get_at(0,).as_deref(), Some("alpha"));
		assert_eq!(string_table.get_at(2,).as_deref(), Some("pha"));
		assert_eq!(string_table.get_at(6,).as_deref(), Some("beta"));
		assert_eq!(string_table.get_at(7,).as_deref(), Some("eta"));
		assert_eq!(string_table.get_at(table.len(),), None);
		success!()
	}

	#[test]
	fn parse_rejects_ranges_outside_binary()
	{
		let binary = b"alpha\0";

		assert!(matches!(StringTable::parse(binary, 4, 8, 0,), Y(_)));
		assert!(matches!(StringTable::parse(binary, usize::MAX, 1, 0,), Y(_)));
	}
}
