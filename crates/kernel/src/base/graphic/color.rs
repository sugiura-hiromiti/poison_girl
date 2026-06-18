use {
	core::{
		convert::From,
		option::Option::{None, Some},
	},
	poison_girl_no_std_error::{
		GraphicError, PoisonGirlB, X, Y, poison_girl_err,
	},
};

pub trait PixelFormat: PixFmtNew
{
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3];

	fn try_color_repr(&self, color: &impl ColorRpr,) -> PoisonGirlB<[u8; 3],>
	{
		let color = color.try_to_color()?;
		X(self.color_repr(&color,),)
	}
}

pub const trait PixFmtNew
{
	fn new_pix() -> Self;
}

pub struct Rgb;
impl PixelFormat for Rgb
{
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3]
	{
		[color.red(), color.green(), color.blue(),]
	}
}

const impl PixFmtNew for Rgb
{
	fn new_pix() -> Self
	{
		Rgb
	}
}

pub struct Bgr;
impl PixelFormat for Bgr
{
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3]
	{
		[color.blue(), color.green(), color.red(),]
	}
}

const impl PixFmtNew for Bgr
{
	fn new_pix() -> Self
	{
		Bgr
	}
}

pub struct Bitmask;
impl PixelFormat for Bitmask
{
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3]
	{
		let _ = color;
		todo!()
	}
}

const impl PixFmtNew for Bitmask
{
	fn new_pix() -> Self
	{
		Bitmask
	}
}

pub struct BltOnly;
impl PixelFormat for BltOnly
{
	fn color_repr(&self, color: &impl ColorRpr,) -> [u8; 3]
	{
		let _ = color;
		todo!()
	}
}

const impl PixFmtNew for BltOnly
{
	fn new_pix() -> Self
	{
		BltOnly
	}
}

/// trait for types which can represent color format
/// implement this trait ensures to be able to get value of red, green, blue
pub trait ColorRpr
{
	fn red(&self,) -> u8;
	fn green(&self,) -> u8;
	fn blue(&self,) -> u8;
	fn red_mut(&mut self, val: u8,);
	fn green_mut(&mut self, val: u8,);
	fn blue_mut(&mut self, val: u8,);
	fn to_color(&self,) -> Color
	{
		Color { red: self.red(), green: self.green(), blue: self.blue(), }
	}
	fn try_to_color(&self,) -> PoisonGirlB<Color,>
	{
		X(self.to_color(),)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq,)]
pub struct Color
{
	red:   u8,
	green: u8,
	blue:  u8,
}

impl Color
{
	pub fn try_from_hex(value: &str,) -> PoisonGirlB<Self,>
	{
		if value.len() != 7 || !value.starts_with('#',) {
			return Y(poison_girl_err!(GraphicError::InvalidColor),);
		}

		X(Self {
			red:   try_hex_component(value, 1, 3,)?,
			green: try_hex_component(value, 3, 5,)?,
			blue:  try_hex_component(value, 5, 7,)?,
		},)
	}
}

impl ColorRpr for Color
{
	fn red(&self,) -> u8
	{
		self.red
	}

	fn green(&self,) -> u8
	{
		self.green
	}

	fn blue(&self,) -> u8
	{
		self.blue
	}

	fn red_mut(&mut self, val: u8,)
	{
		self.red = val;
	}

	fn green_mut(&mut self, val: u8,)
	{
		self.green = val;
	}

	fn blue_mut(&mut self, val: u8,)
	{
		self.blue = val;
	}
}

impl ColorRpr for (u8, u8, u8,)
{
	fn red(&self,) -> u8
	{
		self.0
	}

	fn green(&self,) -> u8
	{
		self.1
	}

	fn blue(&self,) -> u8
	{
		self.2
	}

	fn red_mut(&mut self, val: u8,)
	{
		self.0 = val;
	}

	fn green_mut(&mut self, val: u8,)
	{
		self.1 = val;
	}

	fn blue_mut(&mut self, val: u8,)
	{
		self.2 = val;
	}
}

/// this impl assumes format such as `#012345`
impl ColorRpr for &str
{
	fn red(&self,) -> u8
	{
		hex_component(self, 1, 3,)
	}

	fn green(&self,) -> u8
	{
		hex_component(self, 3, 5,)
	}

	fn blue(&self,) -> u8
	{
		hex_component(self, 5, 7,)
	}

	fn red_mut(&mut self, _val: u8,)
	{
		todo!()
	}

	fn green_mut(&mut self, _val: u8,)
	{
		todo!()
	}

	fn blue_mut(&mut self, _val: u8,)
	{
		todo!()
	}

	fn try_to_color(&self,) -> PoisonGirlB<Color,>
	{
		Color::try_from_hex(self,)
	}
}

fn try_hex_component(value: &str, start: usize, end: usize,)
-> PoisonGirlB<u8,>
{
	match value
		.get(start..end,)
		.and_then(|hex| u8::from_str_radix(hex, 16,).ok(),)
	{
		Some(component,) => X(component,),
		None => Y(poison_girl_err!(GraphicError::InvalidColor),),
	}
}

fn hex_component(value: &str, start: usize, end: usize,) -> u8
{
	match value
		.get(start..end,)
		.and_then(|hex| u8::from_str_radix(hex, 16,).ok(),)
	{
		Some(component,) => component,
		None => 0,
	}
}

impl From<(u8, u8, u8,),> for Color
{
	fn from(value: (u8, u8, u8,),) -> Self
	{
		Color { red: value.0, green: value.1, blue: value.2, }
	}
}

#[cfg(test)]
mod tests
{
	use {super::*, core::prelude::rust_2024::test};

	#[test]
	fn invalid_short_color_hex_returns_error()
	{
		assert!(matches!(Color::try_from_hex("#12345",), Y(_)));
	}

	#[test]
	fn invalid_color_hex_without_prefix_returns_error()
	{
		assert!(matches!(Color::try_from_hex("123456",), Y(_)));
	}

	#[test]
	fn invalid_long_color_hex_returns_error()
	{
		assert!(matches!(Color::try_from_hex("#1234567",), Y(_)));
	}

	#[test]
	fn invalid_color_hex_character_returns_error()
	{
		assert!(matches!(Color::try_from_hex("#12345z",), Y(_)));
	}

	#[test]
	fn invalid_color_hex_component_slice_returns_error()
	{
		assert!(matches!(try_hex_component("#zz3456", 1, 3,), Y(_)));
		assert!(matches!(try_hex_component("#123456", 5, 8,), Y(_)));
	}

	#[test]
	fn valid_color_hex_parses_components()
	{
		assert!(matches!(
			Color::try_from_hex("#0a1Bff",),
			X(Color { red: 0x0a, green: 0x1b, blue: 0xff, })
		));
	}

	#[test]
	fn valid_color_hex_accepts_uppercase_and_lowercase()
	{
		assert!(matches!(
			Color::try_from_hex("#AaBbCc",),
			X(Color { red: 0xaa, green: 0xbb, blue: 0xcc, })
		));
		assert!(matches!(
			Color::try_from_hex("#aabbcc",),
			X(Color { red: 0xaa, green: 0xbb, blue: 0xcc, })
		));
	}

	#[test]
	fn color_can_be_built_from_tuple()
	{
		assert_eq!(
			Color::from((0x12, 0x34, 0x56,),),
			Color { red: 0x12, green: 0x34, blue: 0x56, }
		);
	}

	#[test]
	fn color_repr_mutation_updates_color_and_tuple()
	{
		let mut color = Color::from((0, 0, 0,),);
		color.red_mut(0x12,);
		color.green_mut(0x34,);
		color.blue_mut(0x56,);
		assert_eq!(color, Color { red: 0x12, green: 0x34, blue: 0x56, });

		let mut tuple = (0, 0, 0,);
		tuple.red_mut(0x9a,);
		tuple.green_mut(0xbc,);
		tuple.blue_mut(0xde,);
		assert_eq!(tuple, (0x9a, 0xbc, 0xde,));
	}

	#[test]
	fn rgb_and_bgr_represent_color_in_channel_order()
	{
		let color = Color::from((0x12, 0x34, 0x56,),);

		assert_eq!(Rgb.color_repr(&color,), [0x12, 0x34, 0x56,]);
		assert_eq!(Bgr.color_repr(&color,), [0x56, 0x34, 0x12,]);
	}

	#[test]
	fn str_try_to_color_parses_hex_color()
	{
		assert!(matches!(
			"#010203".try_to_color(),
			X(Color { red: 0x01, green: 0x02, blue: 0x03, })
		));
		assert!(matches!("010203".try_to_color(), Y(_)));
	}
}
