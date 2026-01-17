pub trait PixelFormat {
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3];
}

pub struct Rgb;
impl PixelFormat for Rgb {
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3] {
		[color.red(), color.green(), color.blue(),]
	}
}

pub struct Bgr;
impl PixelFormat for Bgr {
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3] {
		[color.blue(), color.green(), color.red(),]
	}
}

pub struct Bitmask;
impl PixelFormat for Bitmask {
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3] {
		let _ = color;
		todo!()
	}
}

pub struct BltOnly;
impl PixelFormat for BltOnly {
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3] {
		let _ = color;
		todo!()
	}
}

/// trait for types which can represent color format
/// implement this trait ensures to be able to get value of red, green, blue
pub trait ColorRpr {
	fn red(&self,) -> u8;
	fn green(&self,) -> u8;
	fn blue(&self,) -> u8;
	fn red_mut(&mut self, val: u8,);
	fn green_mut(&mut self, val: u8,);
	fn blue_mut(&mut self, val: u8,);
	fn to_color(&self,) -> Color {
		Color { red: self.red(), green: self.green(), blue: self.blue(), }
	}
}

pub struct Color {
	red:   u8,
	green: u8,
	blue:  u8,
}

impl ColorRpr for Color {
	fn red(&self,) -> u8 {
		self.red
	}

	fn green(&self,) -> u8 {
		self.green
	}

	fn blue(&self,) -> u8 {
		self.blue
	}

	fn red_mut(&mut self, val: u8,) {
		self.red = val;
	}

	fn green_mut(&mut self, val: u8,) {
		self.green = val;
	}

	fn blue_mut(&mut self, val: u8,) {
		self.blue = val;
	}
}

impl ColorRpr for (u8, u8, u8,) {
	fn red(&self,) -> u8 {
		self.0
	}

	fn green(&self,) -> u8 {
		self.1
	}

	fn blue(&self,) -> u8 {
		self.2
	}

	fn red_mut(&mut self, val: u8,) {
		self.0 = val;
	}

	fn green_mut(&mut self, val: u8,) {
		self.1 = val;
	}

	fn blue_mut(&mut self, val: u8,) {
		self.2 = val;
	}
}

/// this impl assumes format such as `#012345`
impl ColorRpr for &str {
	fn red(&self,) -> u8 {
		u8::from_str_radix(&self[1..3], 16,)
			.expect("incorrect representation of color format",)
	}

	fn green(&self,) -> u8 {
		u8::from_str_radix(&self[3..5], 16,)
			.expect("incorrect representation of color format",)
	}

	fn blue(&self,) -> u8 {
		u8::from_str_radix(&self[5..7], 16,)
			.expect("incorrect representation of color format",)
	}

	fn red_mut(&mut self, _val: u8,) {
		todo!()
	}

	fn green_mut(&mut self, _val: u8,) {
		todo!()
	}

	fn blue_mut(&mut self, _val: u8,) {
		todo!()
	}
}

impl From<(u8, u8, u8,),> for Color {
	fn from(value: (u8, u8, u8,),) -> Self {
		Color { red: value.0, green: value.1, blue: value.2, }
	}
}
