//! # Parser Module
//!
//! This module provides parsing utilities and code generation capabilities for
//! the OSO operating system. It includes parsers for different data formats and
//! a framework for building custom parsers.
//!
//! ## Submodules
//!
//! - `binary`: Binary data parsing utilities
//! - `generator`: Parser generation framework and core traits
//! - `html`: HTML parsing capabilities (currently empty)
//!
//! ## Design Philosophy
//!
//! The parser framework is built around traits that allow for composable and
//! extensible parsing capabilities. The design emphasizes zero-cost
//! abstractions and compile-time optimizations suitable for system-level
//! programming.

pub mod binary;
pub mod generator;
pub mod html;
