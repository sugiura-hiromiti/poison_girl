use {
	crate::raw::types::Guid,
	poison_girl_no_std_error::{GuidError, PoisonGirlB, X, Y, poison_girl_err},
};

#[macro_export]
macro_rules! guid {
	($s:literal) => {{
		const GUID: $crate::raw::types::Guid = Guid::fix_by($s,);
		GUID
	}};
}

impl Guid
{
	#[track_caller]
	pub fn gen_from_str(s: impl AsRef<str,>,) -> PoisonGirlB<Self,>
	{
		let mut s = s
			.as_ref()
			.chars()
			.filter_map(|c| Hex::try_from(c,).ok(),)
			.map(|h| h as u8,);
		let mut time_low = next_hex_chunk::<4, _,>(&mut s,)?;
		time_low.reverse();
		let mut time_mid = next_hex_chunk::<2, _,>(&mut s,)?;
		time_mid.reverse();
		let mut time_high_and_version = next_hex_chunk::<2, _,>(&mut s,)?;
		time_high_and_version.reverse();

		let clock_seq_high_and_reserved = next_hex(&mut s,)?;
		let clock_seq_low = next_hex(&mut s,)?;
		let mut node = next_hex_chunk::<6, _,>(&mut s,)?;
		node.reverse();

		X(Self::new(
			time_low,
			time_mid,
			time_high_and_version,
			clock_seq_high_and_reserved,
			clock_seq_low,
			node,
		),)
	}

	pub const fn fix_by(s: &str,) -> Self
	{
		let mut hex = [const { Hex::Zero }; 32];
		read_to_hex(s, &mut hex,);

		let hex: [u8; 16] = AsBytes::<32, [u8; 16],>::as_bytes(&hex,);

		let time_low: [u8; 4] = [hex[3], hex[2], hex[1], hex[0],];
		let time_mid: [u8; 2] = [hex[5], hex[4],];
		let time_high_and_version: [u8; 2] = [hex[7], hex[6],];
		let clock_seq_high_and_reserved = hex[8];
		let clock_seq_low = hex[9];
		let node: [u8; 6] =
			[hex[10], hex[11], hex[12], hex[13], hex[14], hex[15],];
		Guid::new(
			time_low,
			time_mid,
			time_high_and_version,
			clock_seq_high_and_reserved,
			clock_seq_low,
			node,
		)
	}
}

fn next_hex(hexes: &mut impl Iterator<Item = u8,>,) -> PoisonGirlB<u8,>
{
	match hexes.next() {
		Some(hex,) => X(hex,),
		None => Y(poison_girl_err!(GuidError::InvalidLength),),
	}
}

fn next_hex_chunk<const N: usize, I: Iterator<Item = u8,>,>(
	hexes: &mut I,
) -> PoisonGirlB<[u8; N],>
{
	let mut chunk = [0; N];
	for hex in &mut chunk {
		*hex = next_hex(hexes,)?;
	}
	X(chunk,)
}

pub const fn read_to_hex<const N: usize,>(s: &str, buf: &mut [Hex; N],)
{
	let s_ptr = s.as_ptr();
	let s_len = s.len();
	let mut i = 0;
	let mut hex_i = 0;

	while i < s_len {
		let ith = unsafe { *s_ptr.add(i,) };
		if Hex::is_valid_hex(ith,) {
			buf[hex_i] = Hex::to_hex(ith,);
			hex_i += 1;
		}
		i += 1;
	}
}

#[repr(u8)]
#[derive(Clone, Copy, Debug,)]
pub enum Hex
{
	Zero,
	One,
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
	Eleven,
	Twelve,
	Thirteen,
	Fourteen,
	Fifteen,
}

impl Hex
{
	pub const fn to_hex(byte: u8,) -> Self
	{
		match byte {
			b'0' => Hex::Zero,
			b'1' => Hex::One,
			b'2' => Hex::Two,
			b'3' => Hex::Three,
			b'4' => Hex::Four,
			b'5' => Hex::Five,
			b'6' => Hex::Six,
			b'7' => Hex::Seven,
			b'8' => Hex::Eight,
			b'9' => Hex::Nine,
			b'a' | b'A' => Hex::Ten,
			b'b' | b'B' => Hex::Eleven,
			b'c' | b'C' => Hex::Twelve,
			b'd' | b'D' => Hex::Thirteen,
			b'e' | b'E' => Hex::Fourteen,
			b'f' | b'F' => Hex::Fifteen,
			_ => {
				panic!("out of hex representation")
			},
		}
	}

	pub const fn is_valid_hex(byte: u8,) -> bool
	{
		(byte >= b'0' && byte <= b'9')
			|| (byte >= b'a' && byte <= b'f')
			|| (byte >= b'A' && byte <= b'F')
	}
}

impl TryFrom<char,> for Hex
{
	type Error = GuidError;

	fn try_from(value: char,) -> Result<Self, Self::Error,>
	{
		let value = value as u8;
		let code = match value {
			c if Hex::is_valid_hex(c,) => Hex::to_hex(c,),
			_ => {
				return Err(GuidError::InvalidHexChar,);
			},
		};
		Ok(code,)
	}
}

pub const trait BytesToInt<const N: usize,>
{
	fn le_u128(&self,) -> u128;
	fn le_u64(&self,) -> u64
	{
		self.le_u128() as u64
	}
	fn le_u32(&self,) -> u32
	{
		self.le_u128() as u32
	}
	fn le_u16(&self,) -> u16
	{
		self.le_u128() as u16
	}
	fn le_u8(&self,) -> u8
	{
		self.le_u128() as u8
	}
}

pub trait BytesNotTooLong<const B: bool,>
{
}
impl<const BYTES: usize,> BytesNotTooLong<{ bytes_not_too_long::<BYTES,>() },>
	for [Hex; BYTES]
{
}
const fn bytes_not_too_long<const BYTES: usize,>() -> bool
{
	BYTES <= 32
}

pub trait BytesIsEven<const B: bool, const N: usize,>
{
}
impl<const BYTES: usize,> BytesIsEven<{ bytes_is_even::<BYTES,>() }, BYTES,>
	for [Hex; BYTES]
{
}
const fn bytes_is_even<const BYTES: usize,>() -> bool
{
	BYTES.is_multiple_of(2,)
}

const impl<const N: usize,> BytesToInt<N,> for [Hex; N]
where [Hex; N]: BytesNotTooLong<true,>
{
	fn le_u128(&self,) -> u128
	{
		let mut i = 0;
		let mut rslt = 0;
		while i < N {
			rslt += (self[i] as u128) << (4 * i);
			i += 1;
		}
		rslt
	}
}

pub const trait AsBytes<const BYTES: usize, O = Self,>
{
	type Output = O;
	//const LIMIT: usize = BYTES / 2;
	fn as_bytes(&self,) -> Self::Output;
}

const impl<const BYTES: usize,> AsBytes<BYTES, [u8; BYTES / 2],>
	for [Hex; BYTES]
where [Hex; BYTES]: BytesNotTooLong<true,> + BytesIsEven<true, BYTES,>
{
	fn as_bytes(&self,) -> Self::Output
	{
		let mut rslt = [0; BYTES / 2];
		let mut i = 0;
		while i < BYTES / 2 {
			let left = (self[i * 2] as u8) << 4;
			let right = self[i * 2 + 1] as u8;
			rslt[i] = left + right;

			i += 1;
		}
		rslt
	}
}

const impl<const BYTES: usize,> AsBytes<BYTES, [u8; BYTES],> for [Hex; BYTES]
{
	fn as_bytes(&self,) -> Self::Output
	{
		let mut rslt = [0; BYTES];
		let mut i = 0;
		while i < BYTES {
			rslt[i] = self[i] as u8;
			i += 1
		}
		rslt
	}
}

#[allow(dead_code)]
const trait AsLeBytes<const BYTES: usize, O = Self,>:
	AsBytes<BYTES, O,> + BytesNotTooLong<true,> + BytesIsEven<true, BYTES,>
{
	fn as_le_bytes(&self,) -> Self::Output;
}

const impl<const BYTES: usize,> AsLeBytes<BYTES, [u8; BYTES],> for [Hex; BYTES]
where [Hex; BYTES]: BytesNotTooLong<true,> + BytesIsEven<true, BYTES,>
{
	fn as_le_bytes(&self,) -> Self::Output
	{
		let mut le_ordered_hexes = [Hex::Zero; BYTES];
		let mut i = 0;
		while i < BYTES {
			le_ordered_hexes[i] = self[BYTES - i - 1];
			i += 1;
		}
		le_ordered_hexes.as_bytes()
	}
}
