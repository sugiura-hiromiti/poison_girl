use crate::data::tree::NodeValue;
use crate::data::tree::coord::Coordinate;
use crate::data::tree::walk_rslt::WalkTried;

/// Trait for providing windowed views of tree sections.
///
/// This trait extends `TreeWalk` to provide specialized views of tree sections,
/// allowing for efficient navigation of children and sibling nodes.
///
/// # Type Parameters
///
/// - `N`: The node value type for the current tree level
///
/// # Associated Types
///
/// - `ChildrenN`: Node value type for child nodes
/// - `Children`: Tree walker type for child nodes
/// - `BrothersN`: Node value type for sibling nodes
/// - `Brothers`: Tree walker type for sibling nodes
pub trait TreeWindow<N: NodeValue,>: TreeWalker<N,> {
	/// Node value type for child nodes
	type ChildrenN: NodeValue;
	/// Tree walker type for child nodes
	type Children: TreeWalker<Self::ChildrenN,>;

	/// Node value type for sibling nodes
	type BrothersN: NodeValue;
	/// Tree walker type for sibling nodes
	type Brothers: TreeWalker<Self::BrothersN,>;

	/// Get a walker for the children of the current node.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walker that can traverse the child nodes
	fn children<WT: WalkTried<T = Self::Children,>,>(&mut self,) -> WT;

	/// Get a walker for the siblings of the current node.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walker that can traverse the sibling nodes
	fn brothers<WT: WalkTried<T = Self::Brothers,>,>(&mut self,) -> WT;

	fn as_tree_walk(&self,) -> &impl TreeWalker<N,> {
		self
	}
}

//  TODO: reconsider where to attatch `Iterator`

/// Core trait for tree traversal and navigation operations.
///
/// This trait provides a comprehensive set of methods for navigating through
/// tree structures, including movement to parents, children, siblings, and
/// arbitrary positions within the tree.
///
/// # Design Notes
///
/// - The `nth_ancestor` method has a default implementation for convenience
/// - Border conditions (like reaching root or leaf nodes) are handled through
///   the `WalkTried` return type
/// - Position tracking is handled through the `Coordinate` trait
///
/// # Type Parameters
///
/// - `N`: Type implementing `NodeValue` for the node data
pub trait TreeWalker<N: NodeValue,>: Sized + Iterator {
	// === Navigation Operations ===

	/// Navigate to the root of the tree.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn root<WT: WalkTried,>(&mut self,) -> WT;

	/// Get the current tree position.
	///
	/// This method returns a tree walker positioned at the current location.
	/// Note: This is different from `node()` which returns just the node value.
	///
	/// # Returns
	///
	/// A tree walker at the current position
	fn current(&self,) -> impl TreeWalker<N,>;

	/// Navigate to the parent of the current node.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn parent<WT: WalkTried,>(&mut self,) -> WT;

	/// Navigate to the nth ancestor of the current node.
	///
	/// This method provides a default implementation that recursively calls
	/// `parent()` n times. A value of 0 returns the current position.
	///
	/// # Parameters
	///
	/// - `n`: Number of generations to go up (0 = current, 1 = parent, etc.)
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn nth_ancestor<WT: WalkTried,>(&mut self, n: usize,) -> WT {
		if n == 0 {
			// Base case: return current position
			self.as_walk_tried()
		} else {
			// Recursive case: go to parent and continue
			let mut parent = self.parent::<WT>();
			if parent.has_success() {
				// Successfully found parent, continue recursion
				TreeWalker::nth_ancestor::<WT,>(
					parent.current_tree_mut().as_mut().unwrap(),
					n - 1,
				)
			} else {
				// Failed to find parent, return the failure
				parent
			}
		}
	}

	/// Navigate to the nth sibling of the current node.
	///
	/// # Parameters
	///
	/// - `n`: Index of the target sibling (0-based)
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn nth_brother<WT: WalkTried,>(&mut self, n: usize,) -> WT {
		let mut coord = self.get_pos();
		coord.set_last(n,);
		self.set_pos(coord,)
	}

	/// Navigate to the next sibling of the current node.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn next_brother<WT: WalkTried,>(&mut self,) -> WT {
		let dim_count = self.get_pos().dim_count();
		self.move_pos(dim_count, 1,)
	}

	/// Navigate to the previous sibling of the current node.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn prev_brother<WT: WalkTried,>(&mut self,) -> WT {
		let dim_count = self.get_pos().dim_count();
		self.move_pos(dim_count, -1,)
	}

	/// Navigate to the first sibling (leftmost child of parent).
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn first_brother<WT: WalkTried,>(&mut self,) -> WT {
		let mut coord = self.get_pos();
		coord.set_last(0,);
		self.set_pos(coord,)
	}

	/// Navigate to the last sibling (rightmost child of parent).
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn last_brother<WT: WalkTried,>(&mut self,) -> WT {
		let brother_count = self.brother_count();
		let mut coord = self.get_pos();
		coord.set_last(brother_count - 1,);
		self.set_pos(coord,)
	}

	/// Navigate to the nth child of the current node.
	///
	/// # Parameters
	///
	/// - `n`: Index of the target child (0-based)
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn nth_child<WT: WalkTried,>(&mut self, n: usize,) -> WT {
		let mut coord = self.get_pos();
		coord.add_dim(n,);
		self.set_pos(coord,)
	}

	/// Navigate to the first child of the current node.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn first_child<WT: WalkTried,>(&mut self,) -> WT {
		self.nth_child(0,)
	}

	/// Navigate to the last child of the current node.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn last_child<WT: WalkTried,>(&mut self,) -> WT {
		let children_count = self.child_count();
		self.nth_child(children_count - 1,)
	}

	/// Set the current position to the specified coordinate.
	///
	/// # Parameters
	///
	/// - `coordinate`: Target position implementing the `Coordinate` trait
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result indicating success or failure of the operation
	fn set_pos<WT: WalkTried,>(&mut self, coordinate: impl Coordinate,) -> WT;

	/// similar to `set_pos` but act as moving by relative distance
	fn move_pos<WT: WalkTried,>(
		&mut self,
		dimension: usize,
		distance: isize,
	) -> WT;

	// === Position Information Methods ===

	/// Check if the current node has any children.
	///
	/// # Returns
	///
	/// `true` if the node has children, `false` otherwise
	fn has_child(&self,) -> bool;

	/// Check if the current node has a parent.
	///
	/// # Returns
	///
	/// `true` if the node has a parent, `false` if it's the root
	fn has_parent(&self,) -> bool;

	/// Get the number of children of the current node.
	///
	/// # Returns
	///
	/// The count of direct children
	fn child_count(&self,) -> usize;

	/// Get the number of siblings of the current node.
	///
	/// # Returns
	///
	/// The count of sibling nodes (including self)
	fn brother_count(&self,) -> usize;

	/// Get the depth of the tree from the current node.
	///
	/// # Returns
	///
	/// The number of generations in the subtree rooted at this node
	fn generation_count(&self,) -> usize;

	/// Get the position of this node among its siblings.
	///
	/// # Returns
	///
	/// The 0-based index of this node in its parent's children array
	fn get_pos_in_brother() -> usize;

	/// Get the current position as a coordinate.
	///
	/// # Returns
	///
	/// A coordinate representing the current position in the tree
	fn get_pos(&self,) -> impl Coordinate + use<Self, N,>;

	/// Convert the current state to a walk result.
	///
	/// # Type Parameters
	///
	/// - `WT`: Walk result type that must implement `WalkTried`
	///
	/// # Returns
	///
	/// A walk result representing the current state
	fn as_walk_tried<WT: WalkTried,>(&self,) -> WT;

	fn as_tree_window(&self,) -> &impl TreeWindow<N,>
	where Self: TreeWindow<N,> {
		self
	}

	// === Value Access Methods ===

	/// Get the node value at the current position.
	///
	/// This method returns the node value itself, while `current()` returns
	/// the tree walker with position information.
	///
	/// # Returns
	///
	/// The node value of type `N`
	fn node(&self,) -> N;
}
