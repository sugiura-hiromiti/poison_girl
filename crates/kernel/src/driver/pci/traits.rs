use super::{BinaryParser, MemoryReserveEntryData, StructureToken};

pub trait DeviceTree:
	DeviceTreeHeader + DeviceTreeMemoryReservation + DeviceTreeStructure
{
	/// Returns a reference to the memory reservation parser
	///
	/// The memory reservation parser handles the memory reservation block of
	/// the device tree, which contains information about memory regions that
	/// should not be used by the OS.
	///
	/// # Returns
	///
	/// A reference to an object implementing [`DeviceTreeMemoryReservation`]
	fn memory_reservation_parser(&self,) -> &impl DeviceTreeMemoryReservation
	{
		self
	}

	/// Returns a reference to the structure parser
	///
	/// The structure parser handles the main structure block of the device
	/// tree, which contains the actual device nodes and their properties.
	///
	/// # Returns
	///
	/// A reference to an object implementing [`DeviceTreeStructure`]
	fn structure_parser(&self,) -> &impl DeviceTreeStructure
	{
		self
	}

	/// Returns a reference to the strings parser
	///
	/// The strings parser handles the strings block of the device tree,
	/// which contains null-terminated strings referenced by the structure
	/// block.
	///
	/// # Returns
	///
	/// A reference to an object implementing [`DeviceTreeStrings`]
	fn strings_parser(&self,) -> &impl DeviceTreeStrings
	{
		self
	}
}

/// Device Tree header parsing and validation interface
///
/// This trait provides methods for parsing and validating the Device Tree
/// header, which contains metadata about the device tree structure and layout.
///
/// The header is always located at the beginning of the device tree blob and
/// contains offsets and sizes for the other blocks.
///
/// # Device Tree Header Format
///
/// The header contains the following fields (all in big-endian format):
/// - Magic number (0xd00dfeed)
/// - Total size of the device tree
/// - Offset to structure block
/// - Offset to strings block
/// - Offset to memory reservation block
/// - Version information
/// - Boot CPU ID
/// - Block sizes
pub trait DeviceTreeHeader
{
	/// Validates the device tree magic number
	///
	/// The magic number should be 0xd00dfeed (big-endian) for a valid device
	/// tree. This is the first validation step when parsing a device tree.
	///
	/// # Returns
	///
	/// `true` if the magic number is valid, `false` otherwise
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// if device_tree.check_magic() {
	///     println!("Valid device tree found");
	/// } else {
	///     println!("Invalid device tree magic number");
	/// }
	/// ```
	fn check_magic(&self,) -> bool;

	/// Returns the total size of the device tree in bytes
	///
	/// This includes all blocks: header, memory reservation, structure, and
	/// strings.
	///
	/// # Returns
	///
	/// Total size of the device tree blob in bytes
	fn total_size(&self,) -> usize;

	/// Returns the byte offset to the structure block from the beginning of the
	/// device tree
	///
	/// The structure block contains the actual device tree nodes and
	/// properties.
	///
	/// # Returns
	///
	/// Byte offset to the structure block
	fn structure_block_offset(&self,) -> usize;

	/// Returns the byte offset to the strings block from the beginning of the
	/// device tree
	///
	/// The strings block contains null-terminated strings referenced by
	/// properties.
	///
	/// # Returns
	///
	/// Byte offset to the strings block
	fn strings_block_offset(&self,) -> usize;

	/// Returns the byte offset to the memory reservation block from the
	/// beginning of the device tree
	///
	/// The memory reservation block contains entries describing reserved memory
	/// regions.
	///
	/// # Returns
	///
	/// Byte offset to the memory reservation block
	fn memory_reservation_block_offset(&self,) -> usize;

	/// Returns the device tree version number
	///
	/// Different versions may have different features or formats.
	///
	/// # Returns
	///
	/// Device tree version number
	fn version(&self,) -> usize;

	/// Returns the last compatible version number
	///
	/// This indicates the oldest version that can parse this device tree.
	///
	/// # Returns
	///
	/// Last compatible version number
	fn last_compatible_version(&self,) -> usize;

	/// Returns the physical ID of the system boot CPU
	///
	/// This identifies which CPU should be used for booting the system.
	///
	/// # Returns
	///
	/// Physical ID of the boot CPU
	fn system_boot_cpu_physical_id(&self,) -> usize;

	/// Returns the size of the strings block in bytes
	///
	/// # Returns
	///
	/// Size of the strings block in bytes
	fn strings_block_size(&self,) -> usize;

	/// Returns the size of the structure block in bytes
	///
	/// # Returns
	///
	/// Size of the structure block in bytes
	fn structure_block_size(&self,) -> usize;
}

/// Memory reservation block parsing interface
///
/// This trait provides methods for parsing the memory reservation block of the
/// device tree. The memory reservation block contains a list of memory regions
/// that are reserved and should not be used by the operating system.
///
/// Each entry in the memory reservation block consists of an address and size
/// pair, both stored as 64-bit big-endian values.
pub trait DeviceTreeMemoryReservation: MemoryReserveEntry
{
	/// Returns the number of memory reservation entries
	///
	/// The memory reservation block contains zero or more entries, terminated
	/// by an entry with both address and size set to zero.
	///
	/// # Returns
	///
	/// Number of memory reservation entries (excluding the terminating entry)
	fn mem_entries_count(&self,) -> usize;

	/// Returns the nth memory reservation entry
	///
	/// # Arguments
	///
	/// * `n` - Index of the entry to retrieve (0-based)
	///
	/// # Returns
	///
	/// Memory reservation entry data for the specified index
	///
	/// # Panics
	///
	/// May panic if `n` is greater than or equal to the number of entries
	fn nth(&self, n: usize,) -> MemoryReserveEntryData;
}

/// Individual memory reservation entry interface
///
/// This trait provides methods for accessing the data within a single memory
/// reservation entry. Each entry describes a contiguous region of memory
/// that is reserved and should not be used by the OS.
pub trait MemoryReserveEntry: BinaryParser<false, usize,>
{
	/// Returns the physical address of the reserved memory region
	///
	/// # Returns
	///
	/// Physical address of the start of the reserved region
	fn address(&self,) -> usize;

	/// Returns the size of the reserved memory region in bytes
	///
	/// # Returns
	///
	/// Size of the reserved memory region in bytes
	fn mem_size(&self,) -> usize;
}

/// Device Tree structure block parsing interface
///
/// This trait provides methods for parsing the structure block of the device
/// tree, which contains the actual device nodes and their properties. The
/// structure block uses a token-based format to represent the tree structure.
///
/// The structure block contains tokens that represent:
/// - Begin/end node markers
/// - Property definitions
/// - No-operation tokens
/// - End-of-structure marker
pub trait DeviceTreeStructure: DeviceTreeStrings
{
	/// Returns the next structure token from the current position
	///
	/// This method advances the parser position and returns the next token
	/// in the structure block.
	///
	/// # Returns
	///
	/// The next structure token
	fn next_node(&self,) -> StructureToken;

	/// Returns the next structure token in tree traversal order
	///
	/// This method provides tree-aware traversal of the device tree structure,
	/// handling the hierarchical nature of device nodes.
	///
	/// # Returns
	///
	/// The next structure token in tree order
	fn next_node_tree(&self,) -> StructureToken;

	/// Finds and positions the parser at the specified node
	///
	/// This method searches for a device node with the given name and
	/// positions the parser at that node for further processing.
	///
	/// # Arguments
	///
	/// * `name` - Name of the device node to find
	fn get_node(&self, name: &str,);
}

/// Device Tree strings block parsing interface
///
/// This trait provides methods for accessing strings stored in the strings
/// block of the device tree. The strings block contains null-terminated strings
/// that are referenced by properties in the structure block.
pub trait DeviceTreeStrings
{
	/// Retrieves a string from the strings block at the specified offset
	///
	/// # Arguments
	///
	/// * `offset` - Byte offset from the start of the strings block
	///
	/// # Returns
	///
	/// A string slice containing the null-terminated string at the offset
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let device_name = strings_parser.get_name(property_offset);
	/// println!("Device name: {}", device_name);
	/// ```
	fn get_name(&self, offset: usize,) -> &str;

	/// Checks if the string at the given offset matches the specified name
	///
	/// This is a convenience method that combines string retrieval and
	/// comparison.
	///
	/// # Arguments
	///
	/// * `offset` - Byte offset from the start of the strings block
	/// * `name` - Name to compare against
	///
	/// # Returns
	///
	/// `true` if the string at the offset matches the name, `false` otherwise
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// if strings_parser.is_node_of(offset, "pci") {
	///     println!("Found PCI node");
	/// }
	/// ```
	fn is_node_of(&self, offset: usize, name: &str,) -> bool
	{
		self.get_name(offset,) == name
	}
}
