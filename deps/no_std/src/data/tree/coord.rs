/// Trait for representing coordinates/positions within a tree structure.
///
/// This trait provides methods for working with multi-dimensional coordinates
/// that can represent positions in tree hierarchies.
pub trait Coordinate {
	/// Get the value at the nth dimension.
	///
	/// # Parameters
	///
	/// - `n`: The dimension index (0-based)
	///
	/// # Returns
	///
	/// The coordinate value at the specified dimension
	fn nth_dim(&self, n: usize,) -> usize;

	/// Get the value of the first dimension.
	///
	/// This is a convenience method equivalent to `nth_dimension(0)`.
	///
	/// # Returns
	///
	/// The coordinate value at dimension 0
	fn first_dim(&self,) -> usize {
		self.nth_dim(0,)
	}

	/// Get the value of the last dimension.
	///
	/// # Returns
	///
	/// The coordinate value at the highest dimension index
	fn last_dim(&self,) -> usize {
		let last_dimension_is = self.dim_count();
		self.nth_dim(last_dimension_is - 1,)
	}

	/// Get the total number of dimensions in this coordinate.
	///
	/// # Returns
	///
	/// The number of dimensions
	fn dim_count(&self,) -> usize;

	/// Set the value at a specific dimension.
	///
	/// # Parameters
	///
	/// - `dim`: The dimension index to modify
	/// - `value`: The new value for that dimension
	fn set_at(&mut self, dim: usize, value: usize,);

	fn set_first(&mut self, value: usize,) {
		self.set_at(0, value,);
	}

	fn set_last(&mut self, value: usize,) {
		let dim_count = self.dim_count();
		self.set_at(dim_count - 1, value,);
	}

	fn set_dim_count(&mut self, dim_count: usize,);
	fn add_dim(&mut self, init: usize,);
}
