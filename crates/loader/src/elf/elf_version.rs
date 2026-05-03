#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfVersion(pub u8,);

impl ElfVersion
{
	pub const ONE: Self = Self(1,);
}
