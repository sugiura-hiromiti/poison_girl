use {
	crate::{ReadElfH, readelf_h},
	poison_girl_macro_error::{diagnostic::Diag, rslt::Rslt},
	proc_macro2::{Span, TokenStream},
};

pub fn test_elf_header_parse(
	rslt: proc_macro2::TokenStream,
) -> Rslt<TokenStream,>
{
	elf_header_info().replace_by(|ts| {
		Rslt::new(quote::quote! {
		if cfg!(debug_assertions) {
			assert_eq!(#ts, #rslt);
		}
			},)
	},)
}

pub fn elf_header_info() -> Rslt<TokenStream,>
{
	readelf_h()
		.replace_by(|header| {
			elf_header_ident_build(&header,)
				.replace_by(|ident| Rslt::new((ident, header,),),)
		},)
		.replace_by(|(ident, header,)| {
			parse_ty(&header,)
				.replace_by(|ty| Rslt::new((ident, ty, header,),),)
		},)
		.replace_by(|(ident, ty, ref header,)| {
			let machine = parse_machine(header,);
			let version = parse_version(header,);
			let entry = parse_entry(header,);
			let program_header_offset = parse_program_header_offset(header,);
			let section_header_offset = parse_section_header_offset(header,);
			let flags = parse_flags(header,);
			let elf_header_size = parse_elf_header_size(header,);
			let program_header_entry_size =
				parse_program_header_entry_size(header,);
			let program_header_count = parse_program_header_count(header,);
			let section_header_entry_size =
				parse_section_header_entry_size(header,);
			let section_header_count = parse_section_header_count(header,);
			let section_header_index_of_section_name_string_table =
				parse_section_header_index_of_section_name_string_table(header,);
			Rslt::new(quote::quote! {
					ElfHeader {
						ident: #ident,
						ty : #ty,
						machine : #machine,
						version : #version,
						entry : #entry,
						program_header_offset : #program_header_offset,
						section_header_offset : #section_header_offset,
						flags : #flags,
						elf_header_size : #elf_header_size,
						program_header_entry_size : #program_header_entry_size,
						program_header_count : #program_header_count,
						section_header_entry_size : #section_header_entry_size,
						section_header_count : #section_header_count,
						section_header_index_of_section_name_string_table :
			#section_header_index_of_section_name_string_table, 		}
				},)
		},)
}

fn elf_header_ident_build(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	parse_file_class(header,).replace_by(|file_class| {
		parse_endianness(header,).replace_by(|endianness| {
			parse_elf_version(header,).replace_by(|elf_version| {
				parse_target_os_abi(header,).replace_by(|target_os_abi| {
					parse_abi_version(header,).replace_by(|abi_version| {
						Rslt::new(quote::quote! {
							ElfHeaderIdent {
								file_class: #file_class,
								endianness: #endianness,
								elf_version: #elf_version,
								target_os_abi: #target_os_abi,
								abi_version: #abi_version,
							}
						},)
					},)
				},)
			},)
		},)
	},)
}

pub(crate) fn parse_file_class(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let file_class = header.file_class.as_str();

	let file_class = match file_class {
		"ELF64" => quote::quote! {
			FileClass::Bit64
		},
		"ELF32" => quote::quote! {
			FileClass::Bit32
		},
		_ => {
			return Rslt::new_err(format!(
				"failed to parse file_class info: {file_class}"
			),);
		},
	};

	Rslt::new(file_class,)
}

pub(crate) fn parse_endianness(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let endianness = header.endianness.as_str();

	let endianness = match endianness {
		"little" => quote::quote! {
			Endian::Little
		},
		"big" => quote::quote! {
			Endian::Big
		},
		_ => {
			return Rslt::new_err(format!(
				"failed to parse endianness info: {endianness}"
			),);
		},
	};

	Rslt::new(endianness,)
}

pub(crate) fn parse_elf_version(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let elf_version = header.elf_version.as_str();

	let elf_version = match elf_version {
		"1" => quote::quote! {
			ElfVersion::ONE
		},
		ver => {
			let ver: u8 = ver.parse()?;
			quote::quote! {
				ElfVersion(#ver)
			}
		},
	};

	Rslt::new(elf_version.clone(),).with_diag(Diag::note(format!(
		"unrecognized elf version: {elf_version}"
	),),)
}

pub(crate) fn parse_target_os_abi(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let target_os_abi = header.target_os_abi.as_str();

	let target_os_abi = if target_os_abi.contains("UNIX - System V",) {
		quote::quote! {
		TargetOsAbi::SysV
			}
	} else if target_os_abi.contains("Arm",) {
		quote::quote! {
			TargetOsAbi::Arm
		}
	} else if target_os_abi.contains("Standalone",) {
		quote::quote! {
			TargetOsAbi::Standalone
		}
	} else {
		return Rslt::new_err(format!("target_os_abi : {target_os_abi}"),);
	};

	Rslt::new(target_os_abi,)
}

pub(crate) fn parse_abi_version(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let abi_version = header.abi_version.as_str();

	let abi_version = match abi_version {
		"1" => quote::quote! {
			AbiVersion::ONE
		},
		ver => {
			let ver: u8 = ver.parse()?;
			quote::quote! {
				AbiVersion(#ver)
			}
		},
	};

	Rslt::new(abi_version.clone(),).with_diag(Diag::note(format!(
		"unrecognized abi version: {abi_version}"
	),),)
}

pub(crate) fn parse_ty(header: &ReadElfH,) -> Rslt<TokenStream,>
{
	let ty = header.ty.as_str();

	if ty != "EXEC" {
		return Rslt::new_err(format!(
			"poison_girl_kernel.elf type must be executable: {ty}"
		),);
	}

	Rslt::new(quote::quote! {
		ElfType::Executable
	},)
}

pub(crate) fn parse_machine(header: &ReadElfH,) -> proc_macro2::TokenStream
{
	// Normalize machine name: uppercase and replace spaces with underscores
	let machine: String = header
		.machine
		.as_str()
		.chars()
		.map(|c| match c {
			cap if cap.is_ascii_lowercase() => {
				(cap as u8 + b'A' - b'a') as char
			},
			' ' => '_',
			_ => c,
		},)
		.collect();

	// Create the machine constant identifier
	let mut machine_const = "EM_".to_string();
	machine_const.push_str(&machine,);
	let machine = syn::Ident::new(&machine_const, Span::call_site(),);

	quote::quote! {
		ElfHeader::#machine
	}
}

pub(crate) fn parse_version(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let version = header.version.as_str();
	let version = &version[2..]; // Remove "0x" prefix
	let version = u32::from_str_radix(version, 16,)?;

	Rslt::new(quote::quote! {
		#version
	},)
}

pub(crate) fn parse_entry(header: &ReadElfH,)
-> Rslt<proc_macro2::TokenStream,>
{
	let entry = header.entry.as_str();
	let entry = &entry[2..]; // Remove "0x" prefix
	let entry = u64::from_str_radix(entry, 16,)?;

	Rslt::new(quote::quote! {
		#entry
	},)
}

pub(crate) fn parse_program_header_offset(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let program_header_offset = header.program_header_offset.as_str();
	let program_header_offset = program_header_offset.parse::<u64>()?;

	Rslt::new(quote::quote! {
		#program_header_offset
	},)
}

pub(crate) fn parse_section_header_offset(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let section_header_offset = header.section_header_offset.as_str();
	let section_header_offset = section_header_offset.parse::<u64>()?;

	Rslt::new(quote::quote! {
		#section_header_offset
	},)
}

pub(crate) fn parse_flags(header: &ReadElfH,)
-> Rslt<proc_macro2::TokenStream,>
{
	let flags = header.flags.as_str();
	let flags = &flags[2..]; // Remove "0x" prefix
	let flags = u32::from_str_radix(flags, 16,)?;

	Rslt::new(quote::quote! {
		#flags
	},)
}

pub(crate) fn parse_elf_header_size(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let elf_header_size = header.elf_header_size.as_str();
	let elf_header_size = elf_header_size.parse::<u16>()?;

	Rslt::new(quote::quote! {
		#elf_header_size
	},)
}

pub(crate) fn parse_program_header_entry_size(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let program_header_entry_size = header.program_header_entry_size.as_str();
	let program_header_entry_size = program_header_entry_size.parse::<u16>()?;

	Rslt::new(quote::quote! {
		#program_header_entry_size
	},)
}

pub(crate) fn parse_program_header_count(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let program_header_count = header.program_header_count.as_str();
	let program_header_count = program_header_count.parse::<u16>()?;

	Rslt::new(quote::quote! {
		#program_header_count
	},)
}

pub(crate) fn parse_section_header_entry_size(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let section_header_entry_size = header.section_header_entry_size.as_str();
	let section_header_entry_size = section_header_entry_size.parse::<u16>()?;

	Rslt::new(quote::quote! {
		#section_header_entry_size
	},)
}

pub(crate) fn parse_section_header_count(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let section_header_count = header.section_header_count.as_str();
	let section_header_count = section_header_count.parse::<u16>()?;

	Rslt::new(quote::quote! {
		#section_header_count
	},)
}

pub(crate) fn parse_section_header_index_of_section_name_string_table(
	header: &ReadElfH,
) -> Rslt<proc_macro2::TokenStream,>
{
	let section_header_index_of_section_name_string_table =
		header.section_header_index_of_section_name_string_table.as_str();
	let section_header_index_of_section_name_string_table =
		section_header_index_of_section_name_string_table.parse::<u16>()?;

	Rslt::new(quote::quote! {
		#section_header_index_of_section_name_string_table
	},)
}
