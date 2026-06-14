//! # PCI Device Driver and Device Tree Parser
//!
//! This module provides PCI (Peripheral Component Interconnect) device
//! discovery and management through Device Tree (FDT - Flattened Device Tree)
//! parsing. The Device Tree is provided by firmware (such as UEFI or
//! bootloader) and contains hardware configuration information.
//!
//! ## Overview
//!
//! The PCI driver implements a complete Device Tree parser that can extract PCI
//! device information from the Flattened Device Tree format. This is essential
//! for hardware discovery on ARM and RISC-V systems where PCI devices are
//! described in the device tree rather than being enumerable through
//! configuration space scanning.
//!
//! ## Features
//!
//! - **Device Tree Parsing**: Complete FDT (Flattened Device Tree) parser
//!   implementation
//! - **PCI Device Discovery**: Extract PCI device information from device tree
//!   nodes
//! - **Memory Reservation**: Handle memory reservation entries from device tree
//! - **Binary Data Parsing**: Generic binary parser framework for device tree
//!   structures
//! - **Big-Endian Support**: Device trees are typically stored in big-endian
//!   format
//!
//! ## Device Tree Structure
//!
//! A Flattened Device Tree consists of four main blocks:
//!
//! 1. **Header Block**: Contains metadata about the device tree
//! 2. **Memory Reservation Block**: Lists reserved memory regions
//! 3. **Structure Block**: Contains the actual device tree nodes and properties
//! 4. **Strings Block**: Contains null-terminated strings referenced by the
//!    structure block
//!
//! ## Architecture
//!
//! The module is organized around several key traits:
//!
//! - [`DeviceTree`]: Main interface for device tree operations
//! - [`DeviceTreeHeader`]: Header parsing and validation
//! - [`DeviceTreeMemoryReservation`]: Memory reservation handling
//! - [`DeviceTreeStructure`]: Node and property parsing
//! - [`BinaryParser`]: Generic binary data parsing framework
//!
//! ## Usage
//!
//! ```rust,ignore
//! use poison_girl_kernel::driver::pci::{DeviceTree, DeviceTreeData};
//!
//! // Parse device tree from firmware-provided pointer
//! let device_tree = DeviceTreeData::new(device_tree_ptr);
//!
//! // Validate device tree header
//! if device_tree.check_magic() {
//!     // Extract PCI device information
//!     let pci_node = device_tree.get_node("pci");
//!     // Process PCI devices...
//! }
//! ```
//!
//! ## Implementation Status
//!
//! This module is currently under development. Many functions are marked as
//! `todo!()` and will be implemented as part of the PCI subsystem development.
//!
//! ## TODO
//!
//! - Implement derive macros for automatic parser generation from type
//!   definitions
//! - Provide foundation for macro-generated parsers in
//!   `poison_girl_binary_parser`
//! - Complete implementation of all parser methods
//! - Add PCI-specific device tree node parsing
//! - Implement PCI device enumeration and initialization
//!
//! ## Safety Considerations
//!
//! Device tree parsing involves:
//! - Raw pointer manipulation for binary data access
//! - Endianness conversion for multi-byte values
//! - Bounds checking for memory safety
//! - Validation of device tree structure integrity

#![allow(dead_code)]

mod binary;
mod data;
mod traits;

pub use {
	binary::{BinaryParser, BinaryParserTarget},
	data::{DeviceTreeData, MemoryReserveEntryData, StructureToken},
	traits::{
		DeviceTree, DeviceTreeHeader, DeviceTreeMemoryReservation,
		DeviceTreeStrings, DeviceTreeStructure, MemoryReserveEntry,
	},
};
