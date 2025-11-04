//! # UEFI Status Code Specification Parser
//!
//! This module provides functionality for parsing UEFI status codes from the
//! official UEFI specification HTML documents. It can extract success, error,
//! and warning codes along with their mnemonics, values, and descriptions.
//!
//! The parser works by:
//! 1. Fetching the UEFI specification page via HTTP
//! 2. Parsing the HTML content to extract status code tables
//! 3. Converting the table data into structured Rust types
//!
//! This is particularly useful for generating constants and enums for UEFI
//! status codes in operating system development.

use crate::RsltP;
use crate::oso_proc_macro_helper::Diag;
use anyhow::Result as Rslt;
use anyhow::anyhow;
use anyhow::bail;
use html5ever::local_name;
use html5ever::tendril::TendrilSink;
use markup5ever::LocalNameStaticSet;
use markup5ever_rcdom::Node;
use markup5ever_rcdom::NodeData;
use markup5ever_rcdom::RcDom;
use proc_macro2::Span;
use std::rc::Rc;

/// HTML element ID of the main status codes section in the UEFI specification
const MAIN_SECTION_ID: &str = "status-codes";

/// HTML element ID of the success codes table in the UEFI specification
const SUCCESS_CODE_TABLE_ID: &str =
	"efi-status-success-codes-high-bit-clear-apx-d-status-codes";

/// HTML element ID of the error codes table in the UEFI specification
const ERROR_CODE_TABLE_ID: &str =
	"efi-status-error-codes-high-bit-set-apx-d-status-codes";

/// HTML element ID of the warning codes table in the UEFI specification
const WARN_CODE_TABLE_ID: &str =
	"efi-status-warning-codes-high-bit-clear-apx-d-status-codes";

/// Trait for converting status code information into token stream parts.
///
/// This trait provides a method to convert status code information into
/// the token stream components needed for generating match arms and
/// associated constants in the Status implementation.
trait TokenParts {
	/// Converts status code information into token stream parts.
	///
	/// # Parameters
	///
	/// * `is_err` - Whether these status codes represent error conditions
	///
	/// # Returns
	///
	/// Returns a vector of tuples where each tuple contains:
	/// - Match arm token stream for the ok_or() method
	/// - Associated constant token stream for the Status impl
	fn token_parts(
		&self,
		is_err: bool,
	) -> Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream,),>;
}

/// Implementation of TokenParts for vectors of StatusCodeInfo.
///
/// This implementation processes each status code in the vector and generates
/// the appropriate token streams for both match arms and associated constants.
impl TokenParts for Vec<StatusCodeInfo,> {
	fn token_parts(
		&self,
		is_err: bool,
	) -> Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream,),> {
		self.iter()
			.map(|sci| {
				// Create identifier from the status code mnemonic
				let mnemonic =
					syn::Ident::new(&sci.mnemonic, Span::call_site(),);

				// Create literal from the status code value
				let value = syn::Lit::Int(syn::LitInt::new(
					&format!("{}", sci.value),
					Span::call_site(),
				),);

				// Generate appropriate match arm based on error status
				let match_arms = if is_err {
					err_match(&mnemonic, &sci.desc,)
				} else {
					ok_match(&mnemonic,)
				};

				// Generate associated constant with documentation
				let assoc = assoc_const(&mnemonic, &value, &sci.desc,);

				(match_arms, assoc,)
			},)
			.collect()
	}
}

/// Container for all UEFI status codes organized by category
///
/// This struct holds the complete set of UEFI status codes parsed from the
/// specification, organized into success, error, and warning categories.
#[derive(Debug,)]
pub struct StatusCode {
	/// Success status codes (high bit clear)
	pub success: Vec<StatusCodeInfo,>,
	/// Error status codes (high bit set)
	pub error:   Vec<StatusCodeInfo,>,
	/// Warning status codes (high bit clear, but indicate warnings)
	pub warn:    Vec<StatusCodeInfo,>,
}

/// Information about a single UEFI status code
///
/// Each status code consists of a mnemonic name (like "EFI_SUCCESS"),
/// a numeric value, and a human-readable description.
#[derive(Debug,)]
pub struct StatusCodeInfo {
	/// The mnemonic name of the status code (e.g., "EFI_SUCCESS")
	pub mnemonic: String,
	/// The numeric value of the status code
	pub value:    usize,
	/// Human-readable description of what the status code means
	pub desc:     String,
}

impl StatusCodeInfo {
	/// Bit mask for error status codes (high bit set)
	///
	/// UEFI error codes have the most significant bit set to 1,
	/// distinguishing them from success and warning codes.
	pub const ERROR_BIT: usize = 1 << (usize::BITS - 1);
}

pub fn status(version: syn::Lit,) -> RsltP {
	let syn::Lit::Float(version,) = version else {
		bail!("version is floating point literal. found {version:?}")
	};

	// Construct the URL for the UEFI specification page
	let status_spec_url = format!(
		"https://uefi.org/specs/UEFI/{version}/Apx_D_Status_Codes.html"
	);

	// Fetch and parse the specification page
	let spec_page = status_spec_page(&status_spec_url,)?;
	// Generate the Status struct implementation using the helper
	let c_enum_impl = impl_status(&spec_page,);

	// Generate the complete Status struct with all implementations
	let enum_def = quote::quote! {
			#[repr(transparent)]
			#[derive(Eq, PartialEq, Clone, Debug,)]
			pub struct Status(pub usize);

			#c_enum_impl
	};

	Ok((enum_def, vec![],),)
}

/// Fetches and parses UEFI status codes from the official specification
///
/// This function downloads the UEFI specification page, parses the HTML
/// content, and extracts all status codes (success, error, and warning) into a
/// structured format. Error codes are automatically marked with the high bit
/// set as per UEFI specification.
///
/// # Arguments
///
/// * `status_spec_url` - URL to the UEFI specification status codes page
///
/// # Returns
///
/// A `Result<StatusCode>` containing all parsed status codes organized by
/// category, or an error if the page cannot be fetched or parsed.
///
/// # Errors
///
/// This function will return an error if:
/// - The HTTP request to fetch the specification fails
/// - The HTML parsing fails
/// - Required HTML elements (tables) are not found
/// - Status code values cannot be parsed as integers
///
/// # Examples
///
/// ```ignore
/// let url = "https://uefi.org/specs/UEFI/2.10/Appendix_D_Status_Codes.html";
/// let status_codes = status_spec_page(url)?;
/// println!("Found {} success codes", status_codes.success.len());
/// ```
pub fn status_spec_page(
	status_spec_url: impl Into<String,>,
) -> Rslt<StatusCode,> {
	// Fetch the specification page
	let mut rsp = ureq::get(status_spec_url.into(),).call()?;
	let rsp_body = rsp.body_mut().read_to_string()?;

	// Parse the HTML document
	let dom = html5ever::parse_document(RcDom::default(), Default::default(),)
		.one(rsp_body.as_str(),);

	let node = dom.document;

	// Find the main status codes section
	let main_section = get_element_by_id(node.clone(), MAIN_SECTION_ID,)
		.expect("failed to get main section node",);

	// Extract the three status code tables
	let success_code_table =
		get_element_by_id(main_section.clone(), SUCCESS_CODE_TABLE_ID,).ok_or(
			anyhow!("ELEMENT WITH ID NOT FOUND: {SUCCESS_CODE_TABLE_ID}"),
		)?;
	let error_code_table =
		get_element_by_id(main_section.clone(), ERROR_CODE_TABLE_ID,).ok_or(
			anyhow!("ELEMENT WITH ID NOT FOUND: {ERROR_CODE_TABLE_ID}"),
		)?;
	let warn_code_table =
		get_element_by_id(main_section.clone(), WARN_CODE_TABLE_ID,).ok_or(
			anyhow!("ELEMENT WITH ID NOT FOUND: {WARN_CODE_TABLE_ID}"),
		)?;

	// Extract table rows from each table (skip header row)
	let success_code_table_rows = table_rows(success_code_table.clone(),);
	let error_code_table_rows = table_rows(error_code_table.clone(),);
	let warn_code_table_rows = table_rows(warn_code_table.clone(),);

	// Parse table data from each row
	let success_codes_info: Vec<Vec<String,>,> = success_code_table_rows
		.iter()
		.map(|n| table_data(n.clone(),),)
		.collect();
	let error_codes_info: Vec<Vec<String,>,> =
		error_code_table_rows.iter().map(|n| table_data(n.clone(),),).collect();
	let warn_codes_info: Vec<Vec<String,>,> =
		warn_code_table_rows.iter().map(|n| table_data(n.clone(),),).collect();

	// Convert raw table data to structured status code info
	let success_codes = status_codes_info(success_codes_info,);
	let mut error_codes = status_codes_info(error_codes_info,);
	let warn_codes = status_codes_info(warn_codes_info,);

	// Set the error bit for all error codes as per UEFI specification
	error_codes.iter_mut().for_each(|sci| {
		sci.value |= StatusCodeInfo::ERROR_BIT;
	},);

	Ok(StatusCode {
		success: success_codes,
		error:   error_codes,
		warn:    warn_codes,
	},)
}

/// Generates the implementation block for the UEFI Status struct.
///
/// This function takes parsed status code information from the UEFI
/// specification and generates a complete implementation block including
/// associated constants for all status codes and error handling methods.
///
/// # Parameters
///
/// * `spec_page` - Parsed status code information from the UEFI specification
///
/// # Returns
///
/// Returns a `proc_macro2::TokenStream` containing the complete implementation
/// block for the Status struct, including:
/// - Associated constants for success, warning, and error status codes
/// - `ok_or()` method for converting status to Result
/// - `ok_or_with()` method for custom error handling
///
/// # Generated Methods
///
/// - `ok_or()`: Converts the status to a Result, returning Ok for
///   success/warning status codes and Err for error status codes
/// - `ok_or_with()`: Similar to ok_or but allows custom transformation of
///   success values
pub fn impl_status(spec_page: &StatusCode,) -> proc_macro2::TokenStream {
	// Generate token parts for success status codes (non-error)
	let (success_match, success_assoc,): (Vec<_,>, Vec<_,>,) =
		spec_page.success.token_parts(false,).into_iter().unzip();

	// Generate token parts for warning status codes (non-error)
	let (warn_match, warn_assoc,): (Vec<_,>, Vec<_,>,) =
		spec_page.warn.token_parts(false,).into_iter().unzip();

	// Generate token parts for error status codes (error)
	let (error_match, error_assoc,): (Vec<_,>, Vec<_,>,) =
		spec_page.error.token_parts(true,).into_iter().unzip();

	quote::quote! {
		impl Status {
			// Associated constants for all status codes
			#(#success_assoc)*
			#(#warn_assoc)*
			#(#error_assoc)*

			/// Converts the status to a Result type.
			///
			/// Returns Ok(Self) for success and warning status codes,
			/// and Err(UefiError) for error status codes.
			pub fn ok_or(self) -> Rslt<Self, oso_error::loader::UefiError> {
				use alloc::string::ToString;
				match self {
					// Success status codes return Ok
					#(#success_match)*
					// Warning status codes return Ok
					#(#warn_match)*
					// Error status codes return Err
					#(#error_match)*
					// Unknown status codes return custom error
					Self(code) => Err(oso_error::oso_err!(oso_error::loader::UefiError::CustomStatus)),
				}
			}

			/// Converts the status to a Result with custom transformation.
			///
			/// Similar to ok_or(), but allows applying a transformation function
			/// to the success value before returning.
			pub fn ok_or_with<T>(self, with: impl FnOnce(Self) -> T) -> Rslt<T, oso_error::loader::UefiError> {
				let status = self.ok_or()?;
				Ok(with(status))
			}
		}
	}
}

/// Generates a match arm for successful (non-error) status codes.
///
/// Creates a match arm that returns `Ok(Self::MNEMONIC)` for the given status
/// code. This is used for success and warning status codes in the `ok_or()`
/// method.
///
/// # Parameters
///
/// * `mnemonic` - The identifier for the status code constant
///
/// # Returns
///
/// Returns a token stream representing a match arm that returns Ok
fn ok_match(mnemonic: &syn::Ident,) -> proc_macro2::TokenStream {
	quote::quote! {
		Self::#mnemonic => Ok(Self::#mnemonic,),
	}
}

/// Generates a match arm for error status codes.
///
/// Creates a match arm that returns an error with the status code description.
/// This is used for error status codes in the `ok_or()` method.
///
/// # Parameters
///
/// * `mnemonic` - The identifier for the status code constant
/// * `msg` - The description message for the error
///
/// # Returns
///
/// Returns a token stream representing a match arm that returns an error
fn err_match(mnemonic: &syn::Ident, msg: &String,) -> proc_macro2::TokenStream {
	let mnemonic_str = mnemonic.to_string();
	quote::quote! {
	Self::#mnemonic => {
		let mut mnemonic = concat!(#mnemonic_str, ": ", #msg);
		Err(oso_error::oso_err!(UefiError::ErrorStatus(mnemonic)))
	},
	}
}

/// Generates an associated constant for a status code.
///
/// Creates an associated constant with documentation derived from the status
/// code description. The constant has the same name as the mnemonic and
/// contains the numeric value of the status code.
///
/// # Parameters
///
/// * `mnemonic` - The identifier for the status code constant
/// * `value` - The numeric value of the status code
/// * `msg` - The description to use as documentation
///
/// # Returns
///
/// Returns a token stream representing an associated constant with
/// documentation
fn assoc_const(
	mnemonic: &syn::Ident,
	value: &syn::Lit,
	msg: &String,
) -> proc_macro2::TokenStream {
	quote::quote! {
		#[doc = #msg]
		pub const #mnemonic: Self = Self(#value);
	}
}

/// Searches for an HTML element with a specific ID in the DOM tree
///
/// This function recursively traverses the HTML DOM tree to find an element
/// with the specified ID attribute. It performs a depth-first search through
/// all child nodes.
///
/// # Arguments
///
/// * `node` - The root node to start searching from
/// * `id` - The ID attribute value to search for
///
/// # Returns
///
/// An `Option<Rc<Node>>` containing the found element, or `None` if not found
pub fn get_element_by_id(node: Rc<Node,>, id: &str,) -> Option<Rc<Node,>,> {
	// Check if current node has the target ID
	let found = if let NodeData::Element { attrs, .. } = &node.data {
		let attrs_borrow = attrs.borrow();
		attrs_borrow.iter().any(|a| {
			// Create a tendril for the target ID
			let value = unsafe {
				tendril::StrTendril::from_byte_slice_without_validating(
					id.as_bytes(),
				)
			};
			let local_name = local_name!("id");

			// Check if this attribute is an ID with the target value
			*a.name.local == *local_name && a.value == value
		},)
	} else {
		false
	};

	if found {
		Some(node,)
	} else {
		// Recursively search child nodes
		node.children
			.borrow()
			.iter()
			.find_map(|n| get_element_by_id(n.clone(), id,),)
	}
}

/// Searches for HTML elements with a specific attribute value
///
/// This function recursively searches the DOM tree for elements that have
/// an attribute with the specified name containing the specified value.
///
/// # Arguments
///
/// * `node` - The root node to start searching from
/// * `attr` - The attribute name to search for
/// * `value` - The value that the attribute should contain
///
/// # Returns
///
/// A vector of all matching nodes. Returns an empty vector if no matches are
/// found.
///
/// # Note
///
/// This function is currently unused but kept for potential future use.
#[allow(dead_code)]
fn get_elements_by_attribute(
	node: Rc<Node,>,
	attr: &str,
	value: &str,
) -> Vec<Rc<Node,>,> {
	let mut rslt = vec![];

	// Check if current node matches the attribute criteria
	let matches = match &node.data {
		NodeData::Element { attrs, .. } => attrs.borrow().iter().any(|a| {
			let local_name =
				string_cache::Atom::<LocalNameStaticSet,>::from(attr,);
			*a.name.local == *local_name && a.value.contains(value,)
		},),
		_ => false,
	};

	if matches {
		rslt.push(node.clone(),);
	}

	// Recursively search child nodes
	node.children.borrow().iter().for_each(|n| {
		let mut child_matches =
			get_elements_by_attribute(n.clone(), attr, value,);
		rslt.append(&mut child_matches,);
	},);

	rslt
}

/// Searches for HTML elements with a specific tag name
///
/// This function recursively searches the DOM tree for elements with
/// the specified tag name (e.g., "div", "table", "tr").
///
/// # Arguments
///
/// * `node` - The root node to start searching from
/// * `tag_name` - The HTML tag name to search for
///
/// # Returns
///
/// A vector of all matching elements
///
/// # Caution
///
/// clone argument passed to `node` every time
fn get_elements_by_name(node: Rc<Node,>, tag_name: &str,) -> Vec<Rc<Node,>,> {
	let mut rslt = vec![];

	// Check if current node matches the tag name
	let matches = match &node.data {
		NodeData::Element { name, .. } => {
			let element_name =
				string_cache::Atom::<LocalNameStaticSet,>::from(tag_name,);
			*name.local == *element_name
		},
		_ => false,
	};

	if matches {
		rslt.push(node.clone(),);
	}

	// Recursively search child nodes
	node.children.borrow().clone().into_iter().for_each(|n| {
		let mut child_matches = get_elements_by_name(n.clone(), tag_name,);
		rslt.append(&mut child_matches,);
	},);

	rslt
}

/// Extracts table rows from an HTML table, excluding the header row
///
/// This function finds all `<tr>` elements within a table and returns all
/// rows except the first one (which is assumed to be the header).
///
/// # Arguments
///
/// * `node` - The table node to extract rows from
///
/// # Returns
///
/// A vector of table row nodes, excluding the header row
fn table_rows(node: Rc<Node,>,) -> Vec<Rc<Node,>,> {
	// Get all <tr> elements and skip the first one (header)
	get_elements_by_name(node.clone(), "tr",)[1..].to_vec()
}

/// Extracts text data from table cells in a table row
///
/// This function expects a table row with exactly 3 cells containing
/// paragraph (`<p>`) elements with text content. It extracts the text
/// from each cell to build the status code information.
///
/// # Arguments
///
/// * `node` - The table row node to extract data from
///
/// # Returns
///
/// A vector of 3 strings representing:
/// 1. Status code mnemonic (e.g., "EFI_SUCCESS")
/// 2. Status code value (e.g., "0x00000000")
/// 3. Status code description
///
/// # Panics
///
/// Panics if the expected paragraph elements or text nodes are not found
fn table_data(node: Rc<Node,>,) -> Vec<String,> {
	let mut rslt = vec![];

	// Find all paragraph elements in the row (should be 3)
	let row = get_elements_by_name(node.clone(), "p",);

	// Extract text from the first cell (mnemonic)
	let NodeData::Text { ref contents, } =
		row[0].clone().children.borrow()[0].clone().data
	else {
		panic!("text node expected: {:#?}", row[0].clone())
	};
	rslt.push(contents.borrow().as_str().to_string(),);

	// Extract text from the second cell (value)
	let NodeData::Text { ref contents, } =
		row[1].clone().children.borrow()[0].clone().data
	else {
		panic!("text node expected: {:#?}", row[1].clone())
	};
	rslt.push(contents.borrow().as_str().to_string(),);

	// Extract text from the third cell (description)
	let NodeData::Text { ref contents, } =
		row[2].clone().children.borrow()[0].clone().data
	else {
		panic!("text node expected: {:#?}", row[2].clone())
	};
	rslt.push(contents.borrow().as_str().to_string(),);

	rslt
}

/// Converts raw table data into structured status code information
///
/// This function takes the raw string data extracted from HTML tables
/// and converts it into `StatusCodeInfo` structs with proper type conversion.
///
/// # Arguments
///
/// * `rows` - A vector of string vectors, where each inner vector contains
///   [mnemonic, value, description] for one status code
///
/// # Returns
///
/// A vector of `StatusCodeInfo` structs with parsed data
///
/// # Panics
///
/// Panics if any status code value cannot be parsed as an integer
fn status_codes_info(rows: Vec<Vec<String,>,>,) -> Vec<StatusCodeInfo,> {
	rows.into_iter()
		.map(|row| StatusCodeInfo {
			mnemonic: row[0].clone(),
			// Parse the hex value string to integer
			value:    row[1]
				.parse()
				.expect("value expected being parsable to integer",),
			desc:     row[2].clone(),
		},)
		.collect()
}

/// Debug utility function to inspect the children of an HTML node
///
/// This function emits diagnostic messages showing information about all
/// child nodes of a given HTML element. Useful for debugging HTML parsing
/// issues.
///
/// # Arguments
///
/// * `node` - The HTML node whose children should be inspected
///
/// # Note
///
/// This function is only used for debugging and emits procedural macro
/// diagnostics.
#[allow(dead_code)]
fn inspect_children(node: Rc<Node,>,) -> Vec<Diag,> {
	// Iterate through all child nodes and emit diagnostic info
	node.children
		.borrow()
		.iter()
		.enumerate()
		.map(|(i, n,)| {
			let name = match &n.data {
				markup5ever_rcdom::NodeData::Document => {
					todo!("inspect_children/Document")
				},
				markup5ever_rcdom::NodeData::Doctype { .. } => {
					todo!("inspect_children/Doctype")
				},
				markup5ever_rcdom::NodeData::Text { contents, } => {
					format!("text: {contents:?}")
				},
				markup5ever_rcdom::NodeData::Comment { .. } => {
					todo!("inspect_children/Comment")
				},
				markup5ever_rcdom::NodeData::Element { name, .. } => {
					format!("element: {name:?}")
				},
				markup5ever_rcdom::NodeData::ProcessingInstruction {
					..
				} => {
					todo!("inspect_children/ProcessingInstruction")
				},
			};
			Diag::Note(format!("{i}, {name}"),)
		},)
		.collect()
}

/// Debug utility function to inspect a single HTML node
///
/// This function emits a diagnostic message with the full debug representation
/// of an HTML node. Useful for debugging HTML parsing and structure issues.
///
/// # Arguments
///
/// * `node` - The HTML node to inspect
///
/// # Note
///
/// This function is only used for debugging and emits procedural macro
/// diagnostics.
#[allow(dead_code)]
fn inspect_node(node: Rc<Node,>,) -> Diag {
	Diag::Note(format!("{node:#?}"),)
}

#[cfg(test)]
mod tests {
	use super::*;
	use html5ever::QualName;
	use html5ever::ns;
	use markup5ever::namespace_url;

	const BASIC_HTML: &str = r#"
<div class="wow" id="identical">
	<p style="color: blue">text</p>
</div>
<section>hohoho</section>
<section class="main_sec wow">
	<h1 id="first_header">welcome</h1>
	<p class="wow">0w0</p>
</section>"#;

	/// this fn converts text input into node representation
	fn parse_text(txt: impl Into<String,>,) -> Rc<Node,> {
		let dom = html5ever::parse_fragment(
			RcDom::default(),
			Default::default(),
			QualName::new(None, ns!(), local_name!(""),),
			vec![],
		)
		.one(txt.into(),);
		dom.document
	}

	#[test]
	fn test_parse_text() {
		let node = parse_text(BASIC_HTML,);
		eprintln!("{node:#?}")
	}

	#[test]
	fn test_get_element_by_id() -> Rslt<(),> {
		let node = parse_text(BASIC_HTML,);
		get_element_by_id(node.clone(), "identical",)
			.ok_or(anyhow!("failed to get element with id identical"),)?;
		get_element_by_id(node.clone(), "first_header",)
			.ok_or(anyhow!("failed to get element with id first_header"),)?;
		get_element_by_id(node.clone(), "non_exist_id",)
			.ok_or(anyhow!("success"),)
			.unwrap_err();
		Ok((),)
	}

	#[test]
	fn test_get_elements_by_attribute() {
		let node = parse_text(BASIC_HTML,);
		let class_wow =
			get_elements_by_attribute(node.clone(), "class", "wow",);
		assert_eq!(class_wow.len(), 3);

		let class_main_sec =
			get_elements_by_attribute(node.clone(), "class", "main_sec",);
		assert_eq!(class_main_sec.len(), 1);

		let style_color_bule =
			get_elements_by_attribute(node.clone(), "style", "color: blue",);
		assert_eq!(style_color_bule.len(), 1);
	}

	#[test]
	fn test_get_elements_by_name() {
		let node = parse_text(BASIC_HTML,);
		let div = get_elements_by_name(node.clone(), "div",);
		assert_eq!(div.len(), 1);

		let p = get_elements_by_name(node.clone(), "p",);
		assert_eq!(p.len(), 2);

		let section = get_elements_by_name(node.clone(), "section",);
		assert_eq!(section.len(), 2);

		let h1 = get_elements_by_name(node.clone(), "h1",);
		assert_eq!(h1.len(), 1);
	}

	#[test]
	fn test_status_code_info_error_bit() {
		// Test the ERROR_BIT constant
		assert_eq!(StatusCodeInfo::ERROR_BIT, 1 << (usize::BITS - 1));

		// Test that it's the most significant bit
		assert_eq!(StatusCodeInfo::ERROR_BIT, 0x8000000000000000_usize);
	}

	#[test]
	fn test_status_code_info_creation() {
		let info = StatusCodeInfo {
			mnemonic: "EFI_SUCCESS".to_string(),
			value:    0,
			desc:     "The operation completed successfully".to_string(),
		};

		assert_eq!(info.mnemonic, "EFI_SUCCESS");
		assert_eq!(info.value, 0);
		assert_eq!(info.desc, "The operation completed successfully");
	}

	#[test]
	fn test_status_code_creation() {
		let status_code = StatusCode {
			success: vec![StatusCodeInfo {
				mnemonic: "EFI_SUCCESS".to_string(),
				value:    0,
				desc:     "Success".to_string(),
			}],
			error:   vec![StatusCodeInfo {
				mnemonic: "EFI_LOAD_ERROR".to_string(),
				value:    StatusCodeInfo::ERROR_BIT | 1,
				desc:     "Load error".to_string(),
			}],
			warn:    vec![StatusCodeInfo {
				mnemonic: "EFI_WARN_UNKNOWN_GLYPH".to_string(),
				value:    1,
				desc:     "Warning".to_string(),
			}],
		};

		assert_eq!(status_code.success.len(), 1);
		assert_eq!(status_code.error.len(), 1);
		assert_eq!(status_code.warn.len(), 1);

		// Check that error code has the error bit set
		assert!(status_code.error[0].value & StatusCodeInfo::ERROR_BIT != 0);
	}

	#[test]
	fn test_table_rows_filtering() {
		// Create a simple table structure
		let table_html = r#"
<table>
	<tr><th>Header 1</th><th>Header 2</th></tr>
	<tr><td>Row 1 Col 1</td><td>Row 1 Col 2</td></tr>
	<tr><td>Row 2 Col 1</td><td>Row 2 Col 2</td></tr>
</table>"#;

		let node = parse_text(table_html,);
		let table_node =
			get_elements_by_name(node.clone(), "table",)[0].clone();
		let rows = table_rows(table_node,);

		// Should return 2 rows (excluding header)
		assert_eq!(rows.len(), 2);
	}

	#[test]
	fn test_table_data_extraction() {
		// Create a table row with paragraph elements
		let row_html = r#"
<table>
	<tr>
		<td><p>EFI_SUCCESS</p></td>
		<td><p>0x00000000</p></td>
		<td><p>The operation completed successfully.</p></td>
	</tr>
<table/>"#;

		let node = parse_text(row_html,);
		let row_node = get_elements_by_name(node.clone(), "tr",);
		assert_eq!(row_node.len(), 1, "{row_node:#?}");
		let data = table_data(row_node[0].clone(),);

		assert_eq!(data.len(), 3);
		assert_eq!(data[0], "EFI_SUCCESS");
		assert_eq!(data[1], "0x00000000");
		assert_eq!(data[2], "The operation completed successfully.");
	}

	#[test]
	fn test_status_codes_info_conversion() {
		let raw_data = vec![
			vec![
				"EFI_SUCCESS".to_string(),
				"0".to_string(),
				"Success".to_string(),
			],
			vec![
				"EFI_LOAD_ERROR".to_string(),
				"1".to_string(),
				"Load error".to_string(),
			],
		];

		let status_codes = status_codes_info(raw_data,);

		assert_eq!(status_codes.len(), 2);
		assert_eq!(status_codes[0].mnemonic, "EFI_SUCCESS");
		assert_eq!(status_codes[0].value, 0);
		assert_eq!(status_codes[1].mnemonic, "EFI_LOAD_ERROR");
		assert_eq!(status_codes[1].value, 1);
	}

	#[test]
	#[should_panic(expected = "value expected being parsable to integer")]
	fn test_status_codes_info_invalid_value() {
		let raw_data = vec![vec![
			"EFI_SUCCESS".to_string(),
			"invalid_number".to_string(),
			"Success".to_string(),
		]];

		status_codes_info(raw_data,);
	}

	#[test]
	fn test_get_elements_by_name_nested() {
		let nested_html = r#"
<div>
	<p>Outer paragraph</p>
	<section>
		<p>Inner paragraph 1</p>
		<div>
			<p>Deeply nested paragraph</p>
		</div>
		<p>Inner paragraph 2</p>
	</section>
</div>"#;

		let node = parse_text(nested_html,);
		let paragraphs = get_elements_by_name(node, "p",);

		// Should find all 4 paragraph elements regardless of nesting
		assert_eq!(paragraphs.len(), 4);
	}

	#[test]
	fn test_get_elements_by_attribute_partial_match() {
		let html_with_classes = r#"
<div class="status-code-table">Table 1</div>
<div class="status-code-list">List 1</div>
<div class="other-table">Table 2</div>
<div class="status-warning-table">Warning Table</div>"#;

		let node = parse_text(html_with_classes,);
		let status_elements =
			get_elements_by_attribute(node, "class", "status",);

		// Should find elements where class contains "status"
		assert_eq!(status_elements.len(), 3);
	}

	#[test]
	fn test_get_element_by_id_not_found() {
		let simple_html = r#"
<div id="existing">Content</div>
<div>No ID</div>"#;

		let node = parse_text(simple_html,);
		let result = get_element_by_id(node, "nonexistent",);

		assert!(result.is_none());
	}

	#[test]
	fn test_get_element_by_id_nested() {
		let nested_html = r#"
<div>
	<section>
		<div id="deeply-nested">Found me!</div>
	</section>
</div>"#;

		let node = parse_text(nested_html,);
		let result = get_element_by_id(node, "deeply-nested",);

		assert!(result.is_some());
	}

	#[test]
	fn test_constants_values() {
		// Test that the HTML element ID constants are correct
		assert_eq!(MAIN_SECTION_ID, "status-codes");
		assert_eq!(
			SUCCESS_CODE_TABLE_ID,
			"efi-status-success-codes-high-bit-clear-apx-d-status-codes"
		);
		assert_eq!(
			ERROR_CODE_TABLE_ID,
			"efi-status-error-codes-high-bit-set-apx-d-status-codes"
		);
		assert_eq!(
			WARN_CODE_TABLE_ID,
			"efi-status-warning-codes-high-bit-clear-apx-d-status-codes"
		);
	}

	#[test]
	fn test_debug_implementations() {
		let status_info = StatusCodeInfo {
			mnemonic: "TEST".to_string(),
			value:    42,
			desc:     "Test description".to_string(),
		};

		let status_code = StatusCode {
			success: vec![status_info],
			error:   vec![],
			warn:    vec![],
		};

		// Should be able to debug print both structs
		let info_debug = format!("{:?}", status_code.success[0]);
		let code_debug = format!("{:?}", status_code);

		assert!(info_debug.contains("StatusCodeInfo"));
		assert!(info_debug.contains("TEST"));
		assert!(code_debug.contains("StatusCode"));
	}

	#[test]
	fn test_empty_html_parsing() {
		let empty_html = "";
		let node = parse_text(empty_html,);

		// Should not panic and should return a valid node
		assert_eq!(node.children.borrow().len(), 1);
	}

	#[test]
	fn test_malformed_html_parsing() {
		let malformed_html =
			r#"<div><p>Unclosed paragraph<div>Nested without closing</div>"#;
		let node = parse_text(malformed_html,);

		// HTML5 parser should handle malformed HTML gracefully
		let divs = get_elements_by_name(node, "div",);
		assert!(divs.len() > 0);
	}

	#[test]
	fn test_html_with_attributes() {
		let html_with_attrs = r#"
<div id="test-id" class="test-class" data-value="123">
	<p style="color: red;" title="Test paragraph">Content</p>
</div>"#;

		let node = parse_text(html_with_attrs,);

		// Test ID search
		let by_id = get_element_by_id(node.clone(), "test-id",);
		assert!(by_id.is_some());

		// Test class search
		let by_class =
			get_elements_by_attribute(node.clone(), "class", "test-class",);
		assert_eq!(by_class.len(), 1);

		// Test data attribute search
		let by_data =
			get_elements_by_attribute(node.clone(), "data-value", "123",);
		assert_eq!(by_data.len(), 1);

		// Test style attribute search
		let by_style = get_elements_by_attribute(node, "style", "color: red",);
		assert_eq!(by_style.len(), 1);
	}
}
