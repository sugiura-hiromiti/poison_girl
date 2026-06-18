use {
	crate::elf::{
		ELF_ABI_VERSION_INDEX, ELF_ENDIANNESS_INDEX, ELF_FILE_CLASS_INDEX,
		ELF_IDENT_SIZE, ELF_MAGIC_NUMBER, ELF_OS_ABI_INDEX, ELF_VERSION_INDEX,
		abi_version::AbiVersion, elf_version::ElfVersion, endian::Endian,
		file_class::FileClass, target_os_abi::TargetOsAbi,
	},
	poison_girl_no_std_error::{
		ElfParseError, PoisonGirlB, X, Y, poison_girl_err,
	},
};

#[derive(Debug, Default, PartialEq, Eq,)]
pub struct ElfHeaderIdent
{
	pub file_class:    FileClass,
	pub endianness:    Endian,
	pub elf_version:   ElfVersion,
	pub target_os_abi: TargetOsAbi,
	pub abi_version:   AbiVersion,
}

impl ElfHeaderIdent
{
	pub fn new(ident: &[u8],) -> PoisonGirlB<Self,>
	{
		if ident.len() != ELF_IDENT_SIZE {
			return Y(poison_girl_err!(ElfParseError::InvalidIdentLen(
				ident.len()
			)),);
		}

		// check magic number
		// size of elf magic number is 4
		if &ident[0..4] != ELF_MAGIC_NUMBER {
			return Y(poison_girl_err!(ElfParseError::BadMagicNumber(
				ident[0], ident[1], ident[2], ident[3]
			)),);
		}

		let file_class = FileClass::try_from(ident[ELF_FILE_CLASS_INDEX],)?;
		let endianness = Endian::try_from(ident[ELF_ENDIANNESS_INDEX],)?;
		let elf_version = ElfVersion(ident[ELF_VERSION_INDEX],);
		let target_os_abi = TargetOsAbi::try_from(ident[ELF_OS_ABI_INDEX],)?;
		let abi_version = AbiVersion(ident[ELF_ABI_VERSION_INDEX],);

		X(Self {
			file_class,
			endianness,
			elf_version,
			target_os_abi,
			abi_version,
		},)
	}

	pub fn is_64(&self,) -> bool
	{
		self.file_class.is_64()
	}

	pub fn is_little_endian(&self,) -> bool
	{
		self.endianness.is_little_endian()
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::elf::{
			abi_version::AbiVersion, elf_version::ElfVersion, endian::Endian,
			file_class::FileClass, target_os_abi::TargetOsAbi,
		},
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	fn valid_ident() -> [u8; ELF_IDENT_SIZE]
	{
		let mut ident = [0; ELF_IDENT_SIZE];
		ident[0..4].copy_from_slice(b"\x7fELF",);
		ident[ELF_FILE_CLASS_INDEX] = 2;
		ident[ELF_ENDIANNESS_INDEX] = 1;
		ident[ELF_VERSION_INDEX] = 1;
		ident[ELF_OS_ABI_INDEX] = 0;
		ident[ELF_ABI_VERSION_INDEX] = 0;
		ident
	}

	#[test]
	fn parses_valid_ident() -> PoisonGirlTestB
	{
		let ident = ElfHeaderIdent::new(&valid_ident(),)?;

		assert_eq!(
			ident,
			ElfHeaderIdent {
				file_class:    FileClass::Bit64,
				endianness:    Endian::Little,
				elf_version:   ElfVersion::ONE,
				target_os_abi: TargetOsAbi::SysV,
				abi_version:   AbiVersion(0,),
			},
		);
		success!()
	}

	#[test]
	fn short_ident_returns_error()
	{
		assert!(matches!(ElfHeaderIdent::new(&[0; ELF_IDENT_SIZE - 1],), Y(_)));
	}

	#[test]
	fn invalid_magic_returns_error()
	{
		let mut ident = valid_ident();
		ident[0] = 0;

		assert!(matches!(ElfHeaderIdent::new(&ident,), Y(_)));
	}

	#[test]
	fn invalid_file_class_returns_error()
	{
		let mut ident = valid_ident();
		ident[ELF_FILE_CLASS_INDEX] = 0xff;

		assert!(matches!(ElfHeaderIdent::new(&ident,), Y(_)));
	}

	#[test]
	fn invalid_endian_returns_error()
	{
		let mut ident = valid_ident();
		ident[ELF_ENDIANNESS_INDEX] = 0xff;

		assert!(matches!(ElfHeaderIdent::new(&ident,), Y(_)));
	}

	#[test]
	fn invalid_os_abi_returns_error()
	{
		let mut ident = valid_ident();
		ident[ELF_OS_ABI_INDEX] = 0xff;

		assert!(matches!(ElfHeaderIdent::new(&ident,), Y(_)));
	}
}
