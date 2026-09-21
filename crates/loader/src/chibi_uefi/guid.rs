use {
	crate::raw::types::Guid,
	alloc::vec::Vec,
	poison_girl_no_std_error::{GuidError, PoisonGirlB, X, Y, poison_girl_err},
};

#[macro_export]
macro_rules! guid {
	($s:literal) => {{
		use poison_girl_no_std_error::ConstContainer;
		const GUID: $crate::raw::types::Guid = Guid::fix_by($s,).const_unwrap();
		GUID
	}};
}

impl Guid
{
	#[track_caller]
	pub fn gen_from_str(s: impl AsRef<str,>,) -> PoisonGirlB<Self,>
	{
		let mut s = guid_hexes(s.as_ref(),)?.into_iter();
		let mut time_low = next_byte_chunk::<4, _,>(&mut s,)?;
		time_low.reverse();
		let mut time_mid = next_byte_chunk::<2, _,>(&mut s,)?;
		time_mid.reverse();
		let mut time_high_and_version = next_byte_chunk::<2, _,>(&mut s,)?;
		time_high_and_version.reverse();

		let clock_seq_high_and_reserved = next_byte_chunk::<1, _,>(&mut s,)?[0];
		let clock_seq_low = next_byte_chunk::<1, _,>(&mut s,)?[0];
		let node = next_byte_chunk::<6, _,>(&mut s,)?;

		// NOTE: input is correctly 32 nibbles = 16 bytes. if it contains extra
		// nibbles/bytes, return as failure
		if s.next().is_some() {
			return Y(poison_girl_err!(GuidError::InvalidLength),);
		}

		X(Self::new(
			time_low,
			time_mid,
			time_high_and_version,
			clock_seq_high_and_reserved,
			clock_seq_low,
			node,
		),)
	}

	pub const fn fix_by(s: &str,) -> PoisonGirlB<Self,>
	{
		let guid_hex = match GuidHex::new(s,) {
			X(guid_hex,) => guid_hex,
			Y(e,) => return Y(e,),
		};

		let hex = guid_hex.into_bytes();

		let time_low: [u8; 4] = [hex[3], hex[2], hex[1], hex[0],];
		let time_mid: [u8; 2] = [hex[5], hex[4],];
		let time_high_and_version: [u8; 2] = [hex[7], hex[6],];
		let clock_seq_high_and_reserved = hex[8];
		let clock_seq_low = hex[9];
		let node: [u8; 6] =
			[hex[10], hex[11], hex[12], hex[13], hex[14], hex[15],];
		X(Self::new(
			time_low,
			time_mid,
			time_high_and_version,
			clock_seq_high_and_reserved,
			clock_seq_low,
			node,
		),)
	}
}

struct GuidHex([HexByte; 16],);

impl GuidHex
{
	const fn new(s: &str,) -> PoisonGirlB<Self,>
	{
		let hex_digits = match parse_hex_digits::<32,>(s,) {
			X(s,) => s,
			Y(e,) => return Y(e,),
		};
		let hex_bytes = bytes_from_nibble_pairs(hex_digits,);
		X(Self(hex_bytes,),)
	}

	const fn into_bytes(self,) -> [u8; 16]
	{
		self.0.map(const |hex_byte| hex_byte.into_u8(),)
	}
}

struct HexByte
{
	high: HexDigit,
	low:  HexDigit,
}

impl HexByte
{
	const fn into_u8(self,) -> u8
	{
		self.high.into_u8() * 16 + self.low.into_u8()
	}
}

const fn parse_hex_digits<const N: usize,>(
	s: &str,
) -> PoisonGirlB<[HexDigit; N],>
{
	let mut buf = [None; N];

	if !s.is_ascii() {
		return Y(poison_girl_err!(GuidError::NonAsciiChar),);
	}

	let rslt =
		s.as_bytes().iter().filter(|c| **c != b'-',).enumerate().try_for_each(
			|(i, c,)| {
				let hex_digit = HexDigit::from_u8(*c,)?;
				if i >= N {
					return Y(poison_girl_err!(GuidError::InvalidLength),);
				}
				buf[i] = Some(hex_digit,);
				X((),)
			},
		);

	match rslt {
		X(_,) => (),
		Y(e,) => return Y(e,),
	}

	if buf[N - 1].is_none() {
		return Y(poison_girl_err!(GuidError::InvalidLength),);
	}

	let buf = buf.map(|hex| hex.unwrap_or(HexDigit::Zero,),);
	X(buf,)
}

const DOUBLE<const N: usize,>: usize = N * 2;

const fn bytes_from_nibble_pairs<const N: usize,>(
	bytes: [HexDigit; DOUBLE::<N,>],
) -> [HexByte; N]
{
	bytes
		.chunks(2,)
		.map(|chunk| HexByte { high: chunk[0], low: chunk[1], },)
		.collect()
}

fn guid_hexes(s: &str,) -> PoisonGirlB<Vec<u8,>,>
{
	let mut hexes = Vec::with_capacity(32,);
	for c in s.chars() {
		if c == '-' {
			continue;
		}
		match HexDigit::try_from(c,) {
			Ok(hex,) => hexes.push(hex as u8,),
			Err(err,) => return Y(poison_girl_err!(err),),
		}
	}
	X(hexes,)
}

fn next_nibble(hexes: &mut impl Iterator<Item = u8,>,) -> PoisonGirlB<u8,>
{
	match hexes.next() {
		Some(hex,) => X(hex,),
		None => Y(poison_girl_err!(GuidError::InvalidLength),),
	}
}

fn next_nibble_chunk<const N: usize, I: Iterator<Item = u8,>,>(
	hexes: &mut I,
) -> PoisonGirlB<[u8; N],>
{
	let mut chunk = [0; N];
	for hex in &mut chunk {
		*hex = next_nibble(hexes,)?;
	}
	X(chunk,)
}

fn convert_nibble_chunk_to_byte_chunk<const N: usize,>(
	nibbles: [u8; DOUBLE::<N,>],
) -> [u8; N]
{
	let mut byte_chunk = [0; N];
	(0..N).for_each(|i| {
		byte_chunk[i] = (nibbles[i * 2] << 4) | nibbles[i * 2 + 1];
	},);
	byte_chunk
}

fn next_byte_chunk<const N: usize, I: Iterator<Item = u8,>,>(
	hexes: &mut I,
) -> PoisonGirlB<[u8; N],>
where [(); DOUBLE::<N,>]:
{
	let nibble_chunk = next_nibble_chunk::<{ DOUBLE::<N,> }, _,>(hexes,)?;
	let byte_chunk = convert_nibble_chunk_to_byte_chunk::<N,>(nibble_chunk,);
	X(byte_chunk,)
}

#[repr(u8)]
#[derive(Clone, Copy, Debug,)]
pub enum HexDigit
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

impl HexDigit
{
	pub const fn from_u8(byte: u8,) -> PoisonGirlB<Self,>
	{
		let rslt = match byte {
			b'0' => HexDigit::Zero,
			b'1' => HexDigit::One,
			b'2' => HexDigit::Two,
			b'3' => HexDigit::Three,
			b'4' => HexDigit::Four,
			b'5' => HexDigit::Five,
			b'6' => HexDigit::Six,
			b'7' => HexDigit::Seven,
			b'8' => HexDigit::Eight,
			b'9' => HexDigit::Nine,
			b'a' | b'A' => HexDigit::Ten,
			b'b' | b'B' => HexDigit::Eleven,
			b'c' | b'C' => HexDigit::Twelve,
			b'd' | b'D' => HexDigit::Thirteen,
			b'e' | b'E' => HexDigit::Fourteen,
			b'f' | b'F' => HexDigit::Fifteen,
			_ => return Y(poison_girl_err!(GuidError::InvalidHexChar),),
		};
		X(rslt,)
	}

	pub const fn into_u8(self,) -> u8
	{
		match self {
			Self::Zero => 0,
			Self::One => 1,
			Self::Two => 2,
			Self::Three => 3,
			Self::Four => 4,
			Self::Five => 5,
			Self::Six => 6,
			Self::Seven => 7,
			Self::Eight => 8,
			Self::Nine => 9,
			Self::Ten => 10,
			Self::Eleven => 11,
			Self::Twelve => 12,
			Self::Thirteen => 13,
			Self::Fourteen => 14,
			Self::Fifteen => 15,
		}
	}

	pub const fn is_valid_hex(byte: u8,) -> bool
	{
		(byte >= b'0' && byte <= b'9')
			|| (byte >= b'a' && byte <= b'f')
			|| (byte >= b'A' && byte <= b'F')
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

pub trait BytesIsEven<const B: bool, const N: usize,>
{
}

impl<const BYTES: usize,> BytesIsEven<{ bytes_is_even::<BYTES,>() }, BYTES,>
	for [HexDigit; BYTES]
{
}

const fn bytes_is_even<const BYTES: usize,>() -> bool
{
	BYTES.is_multiple_of(2,)
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_test::{PoisonGirlTestB, success},
		poison_girl_no_std_error::Y,
	};

	#[test]
	fn valid_guid_variants_parse_equally() -> PoisonGirlTestB
	{
		let canonical =
			Guid::gen_from_str("09576e91-6d3f-11d2-8e39-00a0c969723b",)?;
		for guid in [
			"09576E91-6D3F-11D2-8E39-00A0C969723B",
			"09576e916d3f11d28e3900a0c969723b",
			"09576e9-16d3f11d2-8e3900a0-c969723b",
		] {
			assert_eq!(canonical, Guid::gen_from_str(guid,)?);
		}
		success!()
	}

	#[test]
	fn invalid_guid_variants_return_error()
	{
		for guid in [
			"09576e91-6d3f-11d2-8e39-00a0c969723_",
			"09576e91-6d3f-11d2-8e39-00a0c969723",
			"09576e91-6d3f-11d2-8e39-00a0c969723b0",
		] {
			assert!(matches!(Guid::gen_from_str(guid,), Y(_)));
		}
	}

	#[test]
	fn fix_by_matches_runtime_parsing_for_guid_variants() -> PoisonGirlTestB
	{
		for guid in [
			"09576e91-6d3f-11d2-8e39-00a0c969723b",
			"09576E91-6D3F-11D2-8E39-00A0C969723B",
		] {
			assert_eq!(Guid::gen_from_str(guid,)?, Guid::fix_by(guid,)?);
		}
		success!()
	}
}
