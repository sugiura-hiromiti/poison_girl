//! # Application Execution and Management
//!
//! This module provides the application execution framework for the OSO kernel.
//! It handles user application lifecycle management, process creation, and
//! application-specific utilities.
//!
//! ## Features
//!
//! - **Application Lifecycle**: Management of application startup, execution,
//!   and termination
//! - **Process Management**: Creation and management of application processes
//! - **User Interface**: Application-level UI components and utilities
//! - **Resource Management**: Application-specific resource allocation and
//!   cleanup
//!
//! ## Modules
//!
//! - [`cursor`]: Cursor management and display utilities for applications
//!
//! ## Usage
//!
//! This module is primarily used by the kernel to manage user applications
//! and provide application-level services.
//!
//! ```rust,ignore
//! use oso_kernel::app::cursor;
//!
//! // Application cursor management
//! // cursor::set_position(x, y);
//! // cursor::show();
//! ```

/// Cursor management and display utilities
///
/// This module provides functionality for managing application cursors,
/// including position tracking, visibility control, and cursor rendering.
pub mod cursor;
