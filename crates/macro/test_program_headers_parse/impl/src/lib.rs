#![feature(iter_array_chunks)]

use {
	poison_girl_dev_cargo::{Arch, BuildMode, Opts},
	poison_girl_dev_orchestrate::decl_manage::{
		OrchestrationResolver, PoisonGirlCargoInterface,
		crate_::PoisonGirlCrateChart,
	},
	poison_girl_macro_error::rslt::Rslt,
	proc_macro2::{Span, TokenStream},
	std::{process::Command, str::FromStr},
};

pub trait IntField: Sized
{
	fn parse(hex: &str,) -> Rslt<Self,>;
}

impl IntField for u32
{
	fn parse(hex: &str,) -> Rslt<Self,>
	{
		let rslt = Self::from_str_radix(hex, 16,)?;
		Rslt::new(rslt,)
	}
}

impl IntField for u64
{
	fn parse(hex: &str,) -> Rslt<Self,>
	{
		let rslt = Self::from_str_radix(hex, 16,)?;
		Rslt::new(rslt,)
	}
}

#[derive(Default, Debug,)]
pub struct ReadElfL
{
	/// Segment type (e.g., "LOAD", "INTERP", "DYNAMIC")
	pub ty:               String,
	/// File offset where the segment begins
	pub offset:           u64,
	/// Virtual address where the segment should be loaded
	pub virtual_address:  u64,
	/// Physical address where the segment should be loaded (usually same as
	/// virtual)
	pub physical_address: u64,
	/// Size of the segment in the file
	pub file_size:        u64,
	/// Size of the segment in memory (may be larger than file_size for BSS)
	pub memory_size:      u64,
	/// Segment flags (read/write/execute permissions)
	pub flags:            u32,
	/// Required alignment for the segment
	pub align:            u64,
}

pub fn test_program_headers_parse(
	rslt: syn::punctuated::Punctuated<syn::Ident, syn::Token![,],>,
) -> Rslt<TokenStream,>
{
	let var_name = rslt.get(0,)?;
	let arch = rslt.get(1,)?.to_string();
	let build_mode = rslt.get(2,)?.to_string();
	program_headers_info(arch, build_mode,).replace_by(|v| {
		Rslt::new(quote::quote! {
			if cfg!(debug_assertions) {
				assert_eq!(#v, #var_name);
			}
		},)
	},)
}

pub fn program_headers_info(
	arch: String,
	build_mode: String,
) -> Rslt<TokenStream,>
{
	readelf_l(arch, build_mode,).replace_by(|program_headers| {
		let program_headers = program_headers.iter().map(|rel| {
			let ty = parse_program_header_type(rel,);
			let flags = rel.flags;
			let offset = rel.offset;
			let virtual_address = rel.virtual_address;
			let physical_address = rel.physical_address;
			let file_size = rel.file_size;
			let memory_size = rel.memory_size;
			let align = rel.align;

			quote::quote! {
				ProgramHeader {
					ty: #ty,
					flags: #flags,
					offset: #offset,
					virtual_address: #virtual_address,
					physical_address: #physical_address,
					file_size: #file_size,
					memory_size: #memory_size,
					align: #align,
				}
			}
		},);
		Rslt::new(quote::quote! {
			alloc::vec![
				#(#program_headers, )*
			]
		},)
	},)
}

fn parse_program_header_type(
	program_header: &ReadElfL,
) -> proc_macro2::TokenStream
{
	// Convert underscore_separated to CamelCase
	let camel_cased: String = program_header
		.ty
		.split("_",)
		.flat_map(|word| {
			word.char_indices().map(|(i, c,)| {
				if i == 0 { c } else { (c as u8 - b'A' + b'a') as char }
			},)
		},)
		.collect();

	let ident = syn::Ident::new(&camel_cased, Span::call_site(),);

	quote::quote! {
		ProgramHeaderType::#ident
	}
}

pub fn readelf_l(arch: String, build_mode: String,) -> Rslt<Vec<ReadElfL,>,>
{
	readelf_l_out(arch, build_mode,)
		.replace_by(|program_headers_info| {
			program_headers_count(&program_headers_info[0],)
				.replace_by(|count| Rslt::new((count, program_headers_info,),),)
		},)
		.replace_by(|(count, info,)| {
			program_headers_fields(&info, count,)
				.map(|s| {
					let fields_info: Vec<_,> =
						s.split(" ",).filter(|s| !s.is_empty(),).collect();

					let ty = fields_info[0].to_string();
					let offset = parse_str_hex_repr(fields_info[1],)?;
					let virtual_address = parse_str_hex_repr(fields_info[2],)?;
					let physical_address = parse_str_hex_repr(fields_info[3],)?;
					let file_size = parse_str_hex_repr(fields_info[4],)?;
					let memory_size = parse_str_hex_repr(fields_info[5],)?;
					let (flags, align,) = parse_flags_and_align(&fields_info,)?;

					Rslt::new(ReadElfL {
						ty,
						offset,
						virtual_address,
						physical_address,
						file_size,
						memory_size,
						flags,
						align,
					},)
				},)
				.fold(Rslt::new(vec![],), |acc, field| acc.push_elem(field,),)
		},)
}

fn readelf_l_out(arch: String, build_mode: String,) -> Rslt<Vec<String,>,>
{
	let arch = Arch::from_str(&arch,)?;
	let build_mode = BuildMode::from_str(&build_mode,)?;
	let kernel_crate = PoisonGirlCargoInterface::new(
		PoisonGirlCrateChart::Kernel,
		Opts { arch, build_mode, ..Default::default() },
	);
	let kernel_bin_path = kernel_crate.build_artifact()?.path();

	let program_headers_info = Command::new("readelf",)
		.arg("-l",)
		.arg(kernel_bin_path,)
		.output()?
		.stdout;
	let program_headers_info = String::from_utf8(program_headers_info,)?;
	let program_headers_info: Vec<_,> = program_headers_info
		.split("Program Headers:",)
		.map(|s| s.to_string(),)
		.collect();

	Rslt::new(program_headers_info,)
}

fn program_headers_count(info: &str,) -> Rslt<usize,>
{
	let desc_lines_count = info.lines().count();
	if desc_lines_count < 2 {
		return Rslt::new_err(
			"Insufficient lines to parse program header count",
		);
	}
	let program_header_count: usize = info
		.lines()
		.nth(desc_lines_count - 2,)?
		.split(" ",)
		.nth(2,)?
		.parse()?;
	Rslt::new(program_header_count,)
}

fn program_headers_fields(
	infos: &[String],
	count: usize,
) -> impl Iterator<Item = std::string::String,>
{
	infos[1]
		.lines()
		.skip(3,)
		.array_chunks::<2>()
		.map(|s| s.concat(),)
		.take(count,)
}

fn parse_str_hex_repr<I: IntField,>(hex: &str,) -> Rslt<I,>
{
	let hex_repr = if hex.len() < 2 {
		// we can assume that `hex` is not prefixed by `0x`
		hex
	} else {
		let prefix = &hex[..2];
		if "0x" == prefix || "0X" == prefix { &hex[2..] } else { hex }
	};
	I::parse(hex_repr,)
}

fn parse_flags_and_align(fields_info: &[&str],) -> Rslt<(u32, u64,),>
{
	let rslt = if fields_info.len() == 8 {
		let flags_str = fields_info[6];
		let mut flags = 0;
		if flags_str.contains("R",) {
			flags |= 0b100;
		}
		if flags_str.contains("W",) {
			flags |= 0b10;
		}
		if flags_str.contains("X",) {
			flags |= 0b1;
		};

		let align = parse_str_hex_repr(fields_info[7],)?;
		(flags, align,)
	} else if fields_info.len() == 9 {
		let align = parse_str_hex_repr(fields_info[8],)?;
		(0b101, align,)
	} else {
		return Rslt::new_err(format!(
			"fields_info length should be 8 or 9, get {}",
			fields_info.len()
		),);
	};

	Rslt::new(rslt,)
}

#[cfg(test)] mod tests;
