use poison_girl_no_std_error::{
	ElfParseError, PoisonGirlError, poison_girl_err,
};

#[derive(PartialEq, Eq, Debug, Default,)]
pub enum ElfType
{
	None,
	Relocatable,
	#[default]
	Executable,
	SharedObject,
	Core,
	NumberOfDefined,
	OsSpecificRangeStart,
	OsSpecificRangeEnd,
	ProcessorSpecificRangeStart,
	ProcessorSpecificRangeEnd,
}

impl ElfType
{
	pub fn is_lib(&self,) -> bool
	{
		*self == Self::SharedObject
	}
}

impl TryFrom<u16,> for ElfType
{
	type Error = PoisonGirlError;

	fn try_from(value: u16,) -> Result<Self, Self::Error,>
	{
		let ty = match value {
			0 => Self::None,
			1 => Self::Relocatable,
			2 => Self::Executable,
			3 => Self::SharedObject,
			4 => Self::Core,
			5 => Self::NumberOfDefined,
			0xfe00 => Self::OsSpecificRangeStart,
			0xfeff => Self::OsSpecificRangeEnd,
			0xff00 => Self::ProcessorSpecificRangeStart,
			0xffff => Self::OsSpecificRangeEnd,
			_ => {
				return Err(poison_girl_err!(ElfParseError::UnknownEfiType(
					value
				)),);
			},
		};
		Ok(ty,)
	}
}
