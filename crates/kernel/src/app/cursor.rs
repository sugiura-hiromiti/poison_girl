use {
	crate::base::graphic::{
		FRAME_BUFFER,
		position::{Coord, Coordinal},
	},
	poison_girl_no_std_error::{PoisonGirlB, X},
};

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
//const MOUSE_CURSOR: [[char; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
pub const MOUSE_CURSOR: [[char; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
	['@', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '@', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '.', '.', '.', '@', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '@', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '@', ' ', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '@', ' ',],
	['@', '.', '.', '.', '.', '.', '.', '.', '@', '@', '@', '@', '@', '@', '@',],
	['@', '.', '.', '.', '.', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '.', '@', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '.', '@', '@', '.', '.', '.', '@', ' ', ' ', ' ', ' ', ' ',],
	['@', '.', '.', '@', ' ', ' ', '@', '.', '.', '.', '@', ' ', ' ', ' ', ' ',],
	['@', '.', '@', ' ', ' ', ' ', '@', '.', '.', '.', '@', ' ', ' ', ' ', ' ',],
	['@', '@', ' ', ' ', ' ', ' ', ' ', '@', '.', '.', '.', '@', ' ', ' ', ' ',],
	['@', ' ', ' ', ' ', ' ', ' ', ' ', '@', '.', '.', '.', '@', ' ', ' ', ' ',],
	[' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '@', '.', '@', '@', ' ', ' ', ' ',],
	[' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '@', '@', ' ', ' ', ' ', ' ', ' ',],
];

pub trait MouseCursorDraw
{
	fn draw_mouse_cursor(&mut self,) -> PoisonGirlB<(),>;
}

/// belong to `gui` struct
pub struct CursorBuf
{
	pos:    Coord,
	width:  usize,
	height: usize,
}

impl CursorBuf
{
	pub fn new() -> Self
	{
		let mut pos = FRAME_BUFFER.right_bottom();
		*pos.x_mut() = pos.x() / 2;
		*pos.y_mut() = pos.y() / 2;
		Self { pos, width: MOUSE_CURSOR_WIDTH, height: MOUSE_CURSOR_HEIGHT, }
	}
}

impl Default for CursorBuf
{
	fn default() -> Self
	{
		Self::new()
	}
}

impl MouseCursorDraw for CursorBuf
{
	fn draw_mouse_cursor(&mut self,) -> PoisonGirlB<(),>
	{
		let mut coord = self.pos.clone();
		(0..self.height).for_each(|y| {
			MOUSE_CURSOR[y].iter().take(self.width,).for_each(|c| {
				match c {
					'@' => todo!(), //put_pixel(&coord,
					// &self.outline_color,)?,
					'.' => todo!(), //put_pixel(&coord, &self.body_color,)?,
					_ => (),
				}
				*coord.x_mut() += 1;
			},);
			*coord.x_mut() = self.pos.x();
			*coord.y_mut() += 1;
		},);

		X((),)
	}
}

//
impl Coordinal for CursorBuf
{
	fn x(&self,) -> usize
	{
		self.pos.x()
	}

	fn y(&self,) -> usize
	{
		self.pos.y()
	}

	fn x_mut(&mut self,) -> &mut usize
	{
		self.pos.x_mut()
	}

	fn y_mut(&mut self,) -> &mut usize
	{
		self.pos.y_mut()
	}
}
