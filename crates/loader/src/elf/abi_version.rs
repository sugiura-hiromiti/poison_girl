#[derive(Debug, Default, PartialEq, Eq,)]
pub struct AbiVersion(pub u8,);
impl AbiVersion
{
	pub const ONE: Self = Self(0,);
}
