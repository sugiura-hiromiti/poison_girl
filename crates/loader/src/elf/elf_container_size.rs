/// the size of a binary container
/// TODO: rename to ElfContainerSize
#[derive(PartialEq, Eq, Clone,)]
pub enum ElfContainerSize
{
	Little,
	Big,
}

impl const Default for ElfContainerSize
{
	fn default() -> Self
	{
		Self::Big
	}
}
