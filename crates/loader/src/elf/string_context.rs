use poison_girl_no_std_error::{
	ElfParseError, PoisonGirlB, X, Y, poison_girl_err,
};

pub enum StringContext
{
	Delimiter(u8,),
	DelimiterUntil(u8, usize,),
	Length(usize,),
}

impl StringContext
{
	pub fn read_bytes<'a,>(&self, bytes: &'a [u8],) -> PoisonGirlB<&'a [u8],>
	{
		let bytes = match self {
			StringContext::Delimiter(delimiter,) => {
				let mut i = 0;
				while let a = &bytes[i..=i]
					&& a != [*delimiter,]
				{
					i += 1;
					if i >= bytes.len() {
						return Y(poison_girl_err!(
							ElfParseError::DelimiterNotFound(*delimiter)
						),);
					}
				}

				&bytes[..i]
			},
			StringContext::DelimiterUntil(..,) => todo!(),
			StringContext::Length(l,) => &bytes[..*l],
		};

		X(bytes,)
	}
}

const impl Default for StringContext
{
	fn default() -> Self
	{
		// null delimiter
		Self::Delimiter(0,)
	}
}
