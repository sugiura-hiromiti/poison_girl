//! # Tree Data Structure Module
//!
//! This module provides a generic tree data structure and associated traits for
//! traversal, manipulation, and navigation. The tree implementation is designed
//! for use in system-level programming where memory efficiency and performance
//! are critical.
//!
//! ## Key Components
//!
//! - `Tree<'a, N>`: The main tree structure with lifetime-managed references
//! - `TreeWalk<N>`: Trait for tree traversal and navigation operations
//! - `TreeWindow<N>`: Trait for windowed views of tree sections
//! - `NodeValue`: Trait for values that can be stored in tree nodes
//! - `Coordinate`: Trait for representing positions within the tree
//!
//! ## Design Principles
//!
//! The tree implementation uses lifetime parameters to ensure memory safety
//! without requiring heap allocation, making it suitable for `no_std`
//! environments. The trait-based design allows for flexible tree operations
//! while maintaining type safety.

use crate::data::node::NodeValue;

pub mod coord;
pub mod walk_rslt;
pub mod walker;

//  TODO: -implement TreePart for Tree
// - refactor codebase to use `TreePart` object as an atomic unit in `TreeWalk`
//   and `TreeWindow`
pub trait TreePart {
	type N: NodeValue;

	fn node(&self,) -> Self::N;
}

/// A generic tree data structure with lifetime-managed references.
///
/// This tree structure uses references to manage parent-child relationships
/// without requiring heap allocation. The generic parameter `N` allows for
/// different node value types, which can be made heterogeneous using enums.
///
/// # Type Parameters
///
/// - `'a`: Lifetime parameter for the tree references
/// - `N`: Type implementing `NodeValue` trait for the node data
///
/// # Examples
///
/// ```rust,ignore
/// use oso_no_std_shared::data::tree::{Tree, Node};
///
/// let root_node = Node("root".to_string());
/// let children = [];
/// let tree = Tree {
///     value: root_node,
///     children: &children,
///     parent: None,
/// };
/// ```
pub struct Tree<'a, N: NodeValue,> {
	/// The value stored in this node
	value:    N,
	/// Array of child trees (slice to avoid heap allocation)
	children: &'a [Tree<'a, N,>],
	/// Optional reference to the parent tree node
	parent:   Option<&'a Tree<'a, N,>,>,
}
