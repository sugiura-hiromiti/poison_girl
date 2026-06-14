use poison_girl_no_std_error::{
	ElfParseError, PoisonGirlError, poison_girl_err,
};

#[derive(Debug, PartialEq, Eq, Clone,)]
pub enum Endian
{
	Little,
	Big,
}

const impl Default for Endian
{
	fn default() -> Self
	{
		Self::Big
	}
}

impl Endian
{
	pub fn is_little_endian(&self,) -> bool
	{
		*self == Self::Little
	}
}

impl TryFrom<u8,> for Endian
{
	type Error = PoisonGirlError;

	fn try_from(value: u8,) -> Result<Self, Self::Error,>
	{
		match value {
			1 => Ok(Self::Little,),
			2 => Ok(Self::Big,),
			_ => {
				Err(poison_girl_err!(ElfParseError::InvalidEndianFlag(value,)),)
			},
		}
	}
}
