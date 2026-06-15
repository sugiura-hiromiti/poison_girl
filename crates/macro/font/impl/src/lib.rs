#![feature(iterator_try_collect)]
use {
	poison_girl_macro_error::rslt::Rslt, proc_macro2::TokenStream, syn::LitStr,
};

/// Number of ASCII characters supported (0-255)
const CHARACTER_COUNT: usize = 256;

pub fn font(path: syn::LitStr,) -> Rslt<TokenStream,>
{
	let fonts = convert_bitfield(&font_data(path,)?,)?;
	Rslt::new(quote::quote! {
		&[#(#fonts),*]
	},)
}

fn font_data(specified_path: LitStr,) -> Rslt<Vec<String,>,>
{
	// Get the project root directory, falling back to compile-time directory if
	// needed
	let project_root = std::env::var("CARGO_MANIFEST_DIR",)?;

	// Construct the full path to the font file
	let path = format!("{project_root}/{}", specified_path.value());

	// Read the font data file
	let font_data = std::fs::read_to_string(&path,)?;

	// Split the file into lines and filter out empty lines and hex values
	let fonts_data_lines: Vec<&str,> = font_data
		.split("\n",)
		.collect::<Vec<&str,>>()
		.into_iter()
		.filter(|s| !(s.is_empty() || s.contains("0x",)),) // Remove empty lines and hex values
		.collect();
	let expected_line_count = CHARACTER_COUNT * 16;
	if fonts_data_lines.len() != expected_line_count {
		return Rslt::new_err(format!(
			"font file must contain {expected_line_count} bitmap rows, found \
			 {}",
			fonts_data_lines.len()
		),);
	}

	// Process each character (16 lines per character)
	let mut fonts = vec!["".to_string(); CHARACTER_COUNT];
	for idx in 0..CHARACTER_COUNT {
		// Each character consists of 16 consecutive lines
		fonts[idx] = fonts_data_lines[idx * 16..(idx + 1) * 16].join("",);
	}

	// Verify that each character has exactly 128 characters (16 lines × 8
	// chars)
	for (idx, font,) in fonts.iter().enumerate() {
		if font.len() != 128 {
			return Rslt::new_err(format!(
				"font character {idx} must contain 128 pixels, found {}",
				font.len()
			),);
		}
	}
	Rslt::new(fonts,)
}

fn convert_bitfield(fonts: &[String],) -> Rslt<Vec<u128,>,>
{
	let fonts: Vec<u128,> = fonts
		.iter()
		.map(|s| {
			// Split each character's bitmap into 16 lines
			let lines = s.split("\n",).collect::<Vec<&str,>>();

			// Process each line and combine into a single u128
			let a: u128 = lines
				.into_iter()
				.enumerate()
				.map(|(i, s,)| {
					// Convert '.' to '0' and '@' to '1'
					let s = s.replace(".", "0",).replace("@", "1",);

					// Reverse the bit order for proper display orientation
					let s: String = s.chars().rev().collect();

					// Parse the binary string to get the line value
					let line = u128::from_str_radix(&s, 2,)?;

					// Shift the line to its proper position (line i goes to bit
					// position i*8)
					Rslt::new(line << i,)
				},)
				.try_collect::<Vec<u128,>>()?
				.into_iter()
				.sum(); // Combine all lines using bitwise OR (via sum)
			Rslt::new(a,)
		},)
		.try_collect()?;
	Rslt::new(fonts,)
}

#[cfg(test)] mod tests;
