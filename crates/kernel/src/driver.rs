//! # Hardware Device Drivers and Low-Level Hardware Abstraction
//!
//! This module contains device drivers for various hardware components and
//! provides low-level hardware abstraction layers for the OSO kernel. It
//! enables the kernel to interact with different types of hardware devices in a
//! consistent manner.
//!
//! ## Overview
//!
//! The driver module implements device drivers following a modular architecture
//! where each hardware subsystem has its own dedicated driver implementation.
//! This design allows for easy addition of new hardware support and maintains
//! clean separation between different device types.
//!
//! ## Features
//!
//! - **PCI Device Support**: PCI bus enumeration and device management
//! - **USB Device Support**: USB host controller and device drivers
//! - **Hardware Abstraction**: Consistent interfaces for hardware interaction
//! - **Device Discovery**: Automatic detection and initialization of hardware
//!
//! ## Supported Hardware
//!
//! ### PCI Devices
//! - PCI bus scanning and enumeration
//! - PCI configuration space access
//! - PCI device initialization and management
//!
//! ### USB Devices
//! - USB host controller drivers
//! - USB device enumeration and management
//! - USB transfer handling and protocol support
//!
//! ## Modules
//!
//! - [`pci`]: PCI bus and device driver implementation
//! - [`usb`]: USB host controller and device drivers
//!
//! ## Usage
//!
//! Device drivers are typically initialized during kernel boot and provide
//! standardized interfaces for hardware interaction.
//!
//! ```rust,ignore
//! use oso_kernel::driver::pci;
//! use oso_kernel::driver::usb;
//!
//! // Initialize PCI subsystem
//! // pci::init();
//! // let devices = pci::enumerate_devices();
//!
//! // Initialize USB subsystem
//! // usb::init();
//! // let usb_devices = usb::scan_devices();
//! ```
//!
//! ## Architecture
//!
//! The driver architecture follows these principles:
//!
//! 1. **Modularity**: Each hardware type has its own module
//! 2. **Abstraction**: Common interfaces hide hardware-specific details
//! 3. **Safety**: All hardware access is memory-safe and validated
//! 4. **Performance**: Minimal overhead for critical operations

/// PCI bus and device driver implementation
///
/// This module provides PCI (Peripheral Component Interconnect) bus support,
/// including device enumeration, configuration space access, and device
/// management.
pub mod pci;

/// USB host controller and device drivers
///
/// This module implements USB (Universal Serial Bus) support, including host
/// controller drivers, device enumeration, and USB protocol handling.
pub mod usb;
