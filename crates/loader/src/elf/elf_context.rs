use crate::elf::{elf_container_size::ElfContainerSize, endian::Endian};

#[derive(Clone,)]
#[derive_const(Default)]
pub struct ElfContext
{
	pub container: ElfContainerSize,
	pub le:        Endian,
}
