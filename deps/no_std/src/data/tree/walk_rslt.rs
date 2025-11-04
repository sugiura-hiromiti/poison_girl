use crate::data::tree::NodeValue;
use crate::data::tree::coord::Coordinate;
use crate::data::tree::walker::TreeWalker;

/// Trait for representing the result of tree walk operations.
///
/// This trait encapsulates the result of tree navigation operations, providing
/// information about success/failure and maintaining the last valid position.
///
/// # Type Parameters
///
/// - `N`: Node value type
/// - `T`: Tree walker type
/// - `C`: Coordinate type for position tracking
pub trait WalkTried {
	/// The node value type
	type N: NodeValue;
	/// The tree walker type
	type T: TreeWalker<Self::N,>;
	/// The coordinate type for position tracking
	type C: Coordinate;
	// type TreeNode: TreeWalk<'a, Self::N,>
	// where Self::N: 'a;

	/// Check if the walk operation was successful.
	///
	/// # Returns
	///
	/// `true` if the operation succeeded, `false` otherwise
	fn has_success(&self,) -> bool;

	/// Check if the walk operation failed.
	///
	/// This is the logical inverse of `has_success()`.
	///
	/// # Returns
	///
	/// `true` if the operation failed, `false` otherwise
	fn has_failed(&self,) -> bool {
		!self.has_success()
	}

	/// Get the last valid coordinate before the operation.
	///
	/// This is useful for error recovery and debugging.
	///
	/// # Returns
	///
	/// Reference to the last valid coordinate
	fn last_valid_coordinate(&self,) -> &Self::C;

	/// Get an immutable reference to the current tree walker.
	///
	/// # Returns
	///
	/// Optional reference to the tree walker (None if operation failed)
	fn current_tree(&self,) -> &Option<Self::T,>;

	/// Get a mutable reference to the current tree walker.
	///
	/// # Returns
	///
	/// Optional mutable reference to the tree walker (None if operation failed)
	fn current_tree_mut(&mut self,) -> &mut Option<Self::T,>;

	/// Create a new walk result from a tree walker and coordinate.
	///
	/// # Parameters
	///
	/// - `tn`: The tree walker
	/// - `coord`: The coordinate position
	///
	/// # Returns
	///
	/// A new walk result instance
	fn from(tn: Self::T, coord: Self::C,) -> Self;
}

//  TODO: replace `WalkRslt` with `oso_error::Rslt`

/// Concrete implementation of the `WalkTried` trait.
///
/// This struct represents the result of a tree walk operation, containing
/// either a successful tree walker or information about the failure.
///
/// # Type Parameters
///
/// - `N`: Node value type implementing `NodeValue`
/// - `T`: Tree walker type implementing `TreeWalk<N>`
/// - `C`: Coordinate type implementing `Coordinate`
///
/// # Fields
///
/// - `tree`: Optional tree walker (Some if successful, None if failed)
/// - `coord`: The coordinate position associated with this result
/// - `__constraint`: PhantomData to maintain type parameter `N`
pub struct WalkRslt<N: NodeValue, T: TreeWalker<N,>, C: Coordinate,> {
	/// PhantomData to maintain the node value type parameter
	__constraint: core::marker::PhantomData<N,>,
	/// The tree walker result (None if operation failed)
	tree:         Option<T,>,
	/// The coordinate position for this result
	coord:        C,
}

impl<N: NodeValue, T: TreeWalker<N,>, C: Coordinate,> WalkTried
	for WalkRslt<N, T, C,>
{
	type C = C;
	type N = N;
	type T = T;

	fn has_success(&self,) -> bool {
		self.tree.is_some()
	}

	fn last_valid_coordinate(&self,) -> &Self::C {
		&self.coord
	}

	fn current_tree(&self,) -> &Option<T,> {
		&self.tree
	}

	fn current_tree_mut(&mut self,) -> &mut Option<T,> {
		&mut self.tree
	}

	fn from(tn: T, coord: Self::C,) -> Self {
		Self {
			__constraint: core::marker::PhantomData::<N,>,
			tree: Some(tn,),
			coord,
		}
	}
}
