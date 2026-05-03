use poison_girl_no_std_error::{
	ElfParseError, PoisonGirlError, poison_girl_err,
};

#[non_exhaustive]
#[derive(Debug, Default, PartialEq, Eq,)]
pub enum TargetOsAbi
{
	SysV,
	#[default]
	Arm,
	Standalone,
}

impl TryFrom<u8,> for TargetOsAbi
{
	type Error = PoisonGirlError;

	fn try_from(value: u8,) -> Result<Self, Self::Error,>
	{
		match value {
			0x0 => Ok(Self::SysV,),
			0x53 => Ok(Self::Arm,),
			0x61 => Ok(Self::Standalone,),
			_ => {
				Err(poison_girl_err!(ElfParseError::OsAbiOutOfSupport(value,)),)
			},
		}
	}
}
