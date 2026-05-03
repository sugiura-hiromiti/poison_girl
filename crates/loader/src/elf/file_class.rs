use {
	crate::elf::{ELF_32_BIT_OBJECT, ELF_64_BIT_OBJECT},
	poison_girl_no_std_error::{
		ElfParseError, PoisonGirlError, poison_girl_err,
	},
};

#[derive(PartialEq, Eq, Debug, Default,)]
pub enum FileClass
{
	Bit32,
	#[default]
	Bit64,
}

impl FileClass
{
	pub fn is_64(&self,) -> bool
	{
		*self == FileClass::Bit64
	}
}

impl TryFrom<u8,> for FileClass
{
	type Error = PoisonGirlError;

	fn try_from(value: u8,) -> Result<Self, Self::Error,>
	{
		match value {
			ELF_32_BIT_OBJECT => Ok(Self::Bit32,),
			ELF_64_BIT_OBJECT => Ok(Self::Bit64,),
			_ => {
				Err(poison_girl_err!(ElfParseError::InvalidFileClass(value,)),)
			},
		}
	}
}
