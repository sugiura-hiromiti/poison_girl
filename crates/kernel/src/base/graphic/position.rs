/// trait for types which can represent 2 dimentional area
/// implement this trait ensures to be able to get value of x axis & y axis
pub trait Coordinal {
	fn x(&self,) -> usize;
	fn y(&self,) -> usize;
	fn x_mut(&mut self,) -> &mut usize;
	fn y_mut(&mut self,) -> &mut usize;
	fn to_coord(&self,) -> Coord {
		Coord { x: self.x(), y: self.y(), }
	}
}

#[derive(Clone,)]
pub struct Coord {
	pub x: usize,
	pub y: usize,
}

impl Coordinal for Coord {
	fn x(&self,) -> usize {
		self.x
	}

	fn y(&self,) -> usize {
		self.y
	}

	fn x_mut(&mut self,) -> &mut usize {
		&mut self.x
	}

	fn y_mut(&mut self,) -> &mut usize {
		&mut self.y
	}
}

impl Coordinal for (usize, usize,) {
	fn x(&self,) -> usize {
		self.0
	}

	fn y(&self,) -> usize {
		self.1
	}

	fn x_mut(&mut self,) -> &mut usize {
		&mut self.0
	}

	fn y_mut(&mut self,) -> &mut usize {
		&mut self.1
	}
}
