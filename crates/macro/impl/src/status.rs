use {
	html5ever::{
		LocalNameStaticSet, local_name,
		tendril::{self, TendrilSink},
	},
	markup5ever_rcdom::{Node, NodeData, RcDom},
	poison_girl_proc_macro_helper::{diagnostic::Diag, rslt::Rslt},
	proc_macro2::{Span, TokenStream},
	std::{path::PathBuf, rc::Rc},
};

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
const CRATE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

trait TokenParts
{
	fn token_parts(
		&self,
		is_err: bool,
	) -> Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream,),>;
}

impl TokenParts for Vec<StatusCodeInfo,>
{
	fn token_parts(
		&self,
		is_err: bool,
	) -> Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream,),>
	{
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

#[derive(Debug,)]
pub struct StatusCode
{
	/// Success status codes (high bit clear)
	pub success: Vec<StatusCodeInfo,>,
	/// Error status codes (high bit set)
	pub error:   Vec<StatusCodeInfo,>,
	/// Warning status codes (high bit clear, but indicate warnings)
	pub warn:    Vec<StatusCodeInfo,>,
}

#[derive(Debug,)]
pub struct StatusCodeInfo
{
	/// The mnemonic name of the status code (e.g., "EFI_SUCCESS")
	pub mnemonic: String,
	/// The numeric value of the status code
	pub value:    usize,
	/// Human-readable description of what the status code means
	pub desc:     String,
}

impl StatusCodeInfo
{
	/// Bit mask for error status codes (high bit set)
	///
	/// UEFI error codes have the most significant bit set to 1,
	/// distinguishing them from success and warning codes.
	pub const ERROR_BIT: usize = 1 << (usize::BITS - 1);
}

struct HtmlSource
{
	version: syn::LitFloat,
}

impl HtmlSource
{
	fn new(version: syn::LitFloat,) -> Self
	{
		Self { version, }
	}

	fn fetch(&self,) -> Rslt<String,>
	{
		let local_path = PathBuf::from(CRATE_ROOT,)
			.join(format!("status_{}.html", self.version),);

		if !std::fs::exists(&local_path,)? {
			panic!("file: {} not found", local_path.display());
		}

		Rslt::new(std::fs::read_to_string(local_path,)?,)
	}
}

pub fn status(version: syn::Lit,) -> Rslt<TokenStream,>
{
	let syn::Lit::Float(version,) = version else {
		return Rslt::new_err(format!(
			"version is floating point literal. found {version:?}"
		),);
	};

	// Fetch and parse the specification page
	status_spec_page(version,).replace_by(|spec_page| {
		let c_enum_impl = impl_status(&spec_page,);
		// Generate the complete Status struct with all implementations
		let enum_def = quote::quote! {
				#[repr(transparent)]
				#[derive(Eq, PartialEq, Clone, Debug,)]
				pub struct Status(pub usize);

				#c_enum_impl
		};
		Rslt::new(enum_def,)
	},)
}

pub fn status_spec_page(version: syn::LitFloat,) -> Rslt<StatusCode,>
{
	let rsp_body = HtmlSource::new(version,).fetch()??;

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
			format!("ELEMENT WITH ID NOT FOUND: {SUCCESS_CODE_TABLE_ID}"),
		)?;
	let error_code_table =
		get_element_by_id(main_section.clone(), ERROR_CODE_TABLE_ID,).ok_or(
			format!("ELEMENT WITH ID NOT FOUND: {ERROR_CODE_TABLE_ID}"),
		)?;
	let warn_code_table =
		get_element_by_id(main_section.clone(), WARN_CODE_TABLE_ID,).ok_or(
			format!("ELEMENT WITH ID NOT FOUND: {WARN_CODE_TABLE_ID}"),
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

	Rslt::new(StatusCode {
		success: success_codes,
		error:   error_codes,
		warn:    warn_codes,
	},)
}

pub fn impl_status(spec_page: &StatusCode,) -> proc_macro2::TokenStream
{
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
			pub fn x_or(self) -> poison_girl_no_std_error::PoisonGirlB<Self> {
				use alloc::string::ToString;
				match self {
					// Success status codes return Ok
					#(#success_match)*
					// Warning status codes return Ok
					#(#warn_match)*
					// Error status codes return Err
					#(#error_match)*
					// Unknown status codes return custom error
					Self(code) => poison_girl_no_std_error::Y(poison_girl_no_std_error::poison_girl_err!(poison_girl_no_std_error::UefiError::CustomStatus(code))),
				}
			}

			/// Converts the status to a Result with custom transformation.
			///
			/// Similar to ok_or(), but allows applying a transformation function
			/// to the success value before returning.
			pub fn x_or_with<T>(self, with: impl FnOnce(Self) -> T) -> poison_girl_no_std_error::PoisonGirlB<T,> {
				let status = self.x_or()?;
				poison_girl_no_std_error::X(with(status))
			}
		}
	}
}

fn ok_match(mnemonic: &syn::Ident,) -> proc_macro2::TokenStream
{
	quote::quote! {
		Self::#mnemonic => poison_girl_no_std_error::X(Self::#mnemonic,),
	}
}

fn err_match(mnemonic: &syn::Ident, msg: &String,) -> proc_macro2::TokenStream
{
	let mnemonic_str = mnemonic.to_string();
	quote::quote! {
	Self::#mnemonic => {
		let mut mnemonic = concat!(#mnemonic_str, ": ", #msg);
		poison_girl_no_std_error::Y(poison_girl_no_std_error::poison_girl_err!(poison_girl_no_std_error::UefiError::Status(mnemonic)))
	},
	}
}

fn assoc_const(
	mnemonic: &syn::Ident,
	value: &syn::Lit,
	msg: &String,
) -> proc_macro2::TokenStream
{
	quote::quote! {
		#[doc = #msg]
		pub const #mnemonic: Self = Self(#value);
	}
}

pub fn get_element_by_id(node: Rc<Node,>, id: &str,) -> Option<Rc<Node,>,>
{
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

#[allow(dead_code)]
fn get_elements_by_attribute(
	node: Rc<Node,>,
	attr: &str,
	value: &str,
) -> Vec<Rc<Node,>,>
{
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

fn get_elements_by_name(node: Rc<Node,>, tag_name: &str,) -> Vec<Rc<Node,>,>
{
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

fn table_rows(node: Rc<Node,>,) -> Vec<Rc<Node,>,>
{
	// Get all <tr> elements and skip the first one (header)
	get_elements_by_name(node.clone(), "tr",)[1..].to_vec()
}

fn table_data(node: Rc<Node,>,) -> Vec<String,>
{
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

fn status_codes_info(rows: Vec<Vec<String,>,>,) -> Vec<StatusCodeInfo,>
{
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

#[allow(dead_code)]
fn inspect_children(node: Rc<Node,>,) -> Vec<Diag,>
{
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
			Diag::note(format!("{i}, {name}"),)
		},)
		.collect()
}

#[allow(dead_code)]
fn inspect_node(node: Rc<Node,>,) -> Diag
{
	Diag::note(format!("{node:#?}"),)
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		html5ever::{QualName, ns},
	};

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
	fn parse_text(txt: impl Into<String,>,) -> Rc<Node,>
	{
		let dom = html5ever::parse_fragment(
			RcDom::default(),
			Default::default(),
			QualName::new(None, ns!(), local_name!(""),),
			vec![],
			true,
		)
		.one(txt.into(),);
		dom.document
	}

	#[test]
	fn test_parse_text()
	{
		let node = parse_text(BASIC_HTML,);
		eprintln!("{node:#?}")
	}

	#[test]
	fn test_get_element_by_id() -> Rslt<(),>
	{
		let node = parse_text(BASIC_HTML,);
		get_element_by_id(node.clone(), "identical",)
			.ok_or("failed to get element with id identical",)?;
		get_element_by_id(node.clone(), "first_header",)
			.ok_or("failed to get element with id first_header",)?;
		get_element_by_id(node.clone(), "non_exist_id",)
			.ok_or("success",)
			.unwrap_err();
		Rslt::new((),)
	}

	#[test]
	fn test_get_elements_by_attribute()
	{
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
	fn test_get_elements_by_name()
	{
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
	fn test_status_code_info_error_bit()
	{
		// Test the ERROR_BIT constant
		assert_eq!(StatusCodeInfo::ERROR_BIT, 1 << (usize::BITS - 1));

		// Test that it's the most significant bit
		assert_eq!(StatusCodeInfo::ERROR_BIT, 0x8000000000000000_usize);
	}

	#[test]
	fn test_status_code_info_creation()
	{
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
	fn test_status_code_creation()
	{
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
	fn test_table_rows_filtering()
	{
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
	fn test_table_data_extraction()
	{
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
	fn test_status_codes_info_conversion()
	{
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
	fn test_status_codes_info_invalid_value()
	{
		let raw_data = vec![vec![
			"EFI_SUCCESS".to_string(),
			"invalid_number".to_string(),
			"Success".to_string(),
		]];

		status_codes_info(raw_data,);
	}

	#[test]
	fn test_get_elements_by_name_nested()
	{
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
	fn test_get_elements_by_attribute_partial_match()
	{
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
	fn test_get_element_by_id_not_found()
	{
		let simple_html = r#"
<div id="existing">Content</div>
<div>No ID</div>"#;

		let node = parse_text(simple_html,);
		let result = get_element_by_id(node, "nonexistent",);

		assert!(result.is_none());
	}

	#[test]
	fn test_get_element_by_id_nested()
	{
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
	fn test_constants_values()
	{
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
	fn test_debug_implementations()
	{
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
	fn test_empty_html_parsing()
	{
		let empty_html = "";
		let node = parse_text(empty_html,);

		// Should not panic and should return a valid node
		assert_eq!(node.children.borrow().len(), 1);
	}

	#[test]
	fn test_malformed_html_parsing()
	{
		let malformed_html =
			r#"<div><p>Unclosed paragraph<div>Nested without closing</div>"#;
		let node = parse_text(malformed_html,);

		// HTML5 parser should handle malformed HTML gracefully
		let divs = get_elements_by_name(node, "div",);
		assert!(!divs.is_empty());
	}

	#[test]
	fn test_html_with_attributes()
	{
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
