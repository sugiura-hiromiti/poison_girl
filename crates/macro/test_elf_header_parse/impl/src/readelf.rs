use {poison_girl_macro_error::rslt::Rslt, std::process::Command};

#[derive(Default, Debug,)]
pub struct ReadElfH
{
	/// ELF file class (32-bit or 64-bit)
	pub file_class: String,
	/// Data encoding (little-endian or big-endian)
	pub endianness: String,
	/// ELF version number
	pub elf_version: String,
	/// Target OS/ABI identification
	pub target_os_abi: String,
	/// ABI version number
	pub abi_version: String,
	/// Object file type (executable, shared object, etc.)
	pub ty: String,
	/// Target machine architecture
	pub machine: String,
	/// Object file version
	pub version: String,
	/// Entry point virtual address
	pub entry: String,
	/// Program header table file offset
	pub program_header_offset: String,
	/// Section header table file offset
	pub section_header_offset: String,
	/// Processor-specific flags
	pub flags: String,
	/// ELF header size in bytes
	pub elf_header_size: String,
	/// Program header table entry size
	pub program_header_entry_size: String,
	/// Number of program header table entries
	pub program_header_count: String,
	/// Section header table entry size
	pub section_header_entry_size: String,
	/// Number of section header table entries
	pub section_header_count: String,
	/// Section header string table index
	pub section_header_index_of_section_name_string_table: String,
}

impl ReadElfH
{
	pub(crate) fn fix(&mut self,) -> Rslt<(),>
	{
		// Extract first word from each field (split on whitespace)
		self.file_class = self
			.file_class
			.split_whitespace()
			.next()
			.unwrap_or("",)
			.to_string();
		self.endianness = self
			.endianness
			.split_whitespace()
			.next()
			.unwrap_or("",)
			.to_string();
		self.elf_version = self
			.elf_version
			.split_whitespace()
			.next()
			.unwrap_or("",)
			.to_string();
		// Note: target_os_abi is intentionally not processed as it may contain
		// spaces
		self.abi_version = self.abi_version.split(" ",).nth(0,)?.to_string();
		self.ty = self.ty.split(" ",).nth(0,)?.to_string();
		self.machine = self.machine.split(" ",).nth(0,)?.to_string();
		self.version = self.version.split(" ",).nth(0,)?.to_string();
		self.entry = self.entry.split(" ",).nth(0,)?.to_string();
		self.program_header_offset =
			self.program_header_offset.split(" ",).nth(0,)?.to_string();
		self.section_header_offset =
			self.section_header_offset.split(" ",).nth(0,)?.to_string();
		self.flags = self.flags.split(" ",).nth(0,)?.to_string();
		self.elf_header_size =
			self.elf_header_size.split(" ",).nth(0,)?.to_string();
		self.program_header_entry_size =
			self.program_header_entry_size.split(" ",).nth(0,)?.to_string();
		self.program_header_count =
			self.program_header_count.split(" ",).nth(0,)?.to_string();
		self.section_header_entry_size =
			self.section_header_entry_size.split(" ",).nth(0,)?.to_string();
		self.section_header_count =
			self.section_header_count.split(" ",).nth(0,)?.to_string();
		self.section_header_index_of_section_name_string_table = self
			.section_header_index_of_section_name_string_table
			.split(" ",)
			.nth(0,)?
			.to_string();
		Rslt::new((),)
	}
}

pub(crate) trait Property
{
	fn is_peoperty_of(&self, key: &str,) -> bool;
}

impl Property for Vec<&str,>
{
	fn is_peoperty_of(&self, key: &str,) -> bool
	{
		// Check if the first element (index 0) matches the key
		self.first().is_some_and(|s| *s == key,)
	}
}

pub fn readelf_h() -> Rslt<ReadElfH,>
{
	// Execute readelf command to get header information
	let header_info = Command::new("readelf",)
		.args(["-h", "target/poison_girl_kernel.elf",],)
		.output()?
		.stdout;

	// Convert command output to string
	let header_info = String::from_utf8(header_info,)?;

	// Initialize default header struct
	let mut header = ReadElfH::default();

	// Parse each line of readelf output
	header_info.lines().try_for_each(|line| {
		// Split each line on ':' to get key-value pairs
		let key_value: Vec<_,> = line.split(':',).map(|s| s.trim(),).collect();

		// Parse each field based on the key name
		if key_value.is_peoperty_of("Class",) {
			header.file_class = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Data",) {
			// Extract endianness from "2's complement, little endian" format
			header.endianness = key_value[1].split(" ",).nth(2,)?.to_string();
		}
		if key_value.is_peoperty_of("Version",) {
			// Handle both ELF version and object version fields
			if key_value[1].contains("0x",) {
				header.version = key_value[1].to_string();
			} else {
				header.elf_version =
					key_value[1].split(" ",).next()?.to_string();
			}
		}
		if key_value.is_peoperty_of("OS/ABI",) {
			header.target_os_abi = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("ABI Version",) {
			header.abi_version = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Type",) {
			header.ty = key_value[1].split(" ",).next()?.to_string();
		}
		if key_value.is_peoperty_of("Machine",) {
			header.machine = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Entry point address",) {
			header.entry = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Start of program headers",) {
			header.program_header_offset =
				key_value[1].split(" ",).next()?.to_string();
		}
		if key_value.is_peoperty_of("Start of section headers",) {
			header.section_header_offset = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Flags",) {
			header.flags = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Size of this header",) {
			header.elf_header_size =
				key_value[1].split(" ",).next()?.to_string();
		}
		if key_value.is_peoperty_of("Size of program headers",) {
			header.program_header_entry_size =
				key_value[1].split(" ",).next()?.to_string();
		}
		if key_value.is_peoperty_of("Number of program headers",) {
			header.program_header_count = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Size of section headers",) {
			header.section_header_entry_size = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Number of section headers",) {
			header.section_header_count = key_value[1].to_string();
		}
		if key_value.is_peoperty_of("Section header string table index",) {
			header.section_header_index_of_section_name_string_table =
				key_value[1].to_string();
		}
		Rslt::new((),)
	},)?;

	// Clean up the parsed fields by removing extra whitespace and comments
	header.fix();

	Rslt::new(header,)
}
