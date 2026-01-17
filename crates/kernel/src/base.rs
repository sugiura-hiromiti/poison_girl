//! # Core Kernel Functionality and Basic Data Structures
//!
//! This module provides the fundamental building blocks and core functionality
//! for the OSO kernel. It includes hardware abstraction layers, basic data
//! structures, and essential system utilities.
//!
//! ## Overview
//!
//! The base module serves as the foundation for all kernel operations,
//! providing wrapped functionality for physical world interactions including
//! display management, USB device handling, I/O operations, and system
//! utilities.
//!
//! ## Features
//!
//! - **Hardware Abstraction**: Low-level hardware interface wrappers
//! - **Graphics Management**: Display and framebuffer operations
//! - **I/O Operations**: Input/output handling and device communication
//! - **System Utilities**: Core system functions and data structures
//!
//! ## Modules
//!
//! - [`graphic`]: Graphics and display management functionality
//! - [`io`]: Input/output operations and device communication
//! - [`util`]: System utilities and helper functions
//!
//! ## Usage
//!
//! This module is used throughout the kernel to provide consistent interfaces
//! to hardware and system resources.
//!
//! ```rust,ignore
//! use oso_kernel::base::graphic::FrameBuffer;
//! use oso_kernel::base::io;
//! use oso_kernel::base::util;
//!
//! // Graphics operations
//! // let framebuffer = FrameBuffer::new(...);
//! // framebuffer.draw_pixel(x, y, color);
//!
//! // I/O operations
//! // io::read_input();
//! // io::write_output(data);
//!
//! // Utility functions
//! // util::system_time();
//! ```

/// Graphics and display management functionality
///
/// Provides framebuffer operations, pixel manipulation, and display control.
pub mod graphic;

/// Input/output operations and device communication
///
/// Handles keyboard input, mouse events, and other I/O device interactions.
pub mod io;

/// System utilities and helper functions
///
/// Contains various utility functions and data structures used throughout the
/// kernel.
pub mod util;
