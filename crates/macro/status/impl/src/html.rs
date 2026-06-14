use {
	crate::{
		model::{StatusCode, StatusCodeInfo},
		source::HtmlSource,
	},
	html5ever::{
		LocalNameStaticSet, local_name,
		tendril::{self, TendrilSink},
	},
	markup5ever_rcdom::{Node, NodeData, RcDom},
	poison_girl_macro_error::{diagnostic::Diag, rslt::Rslt},
	std::rc::Rc,
};

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

pub fn status_spec_page(version: syn::LitFloat,) -> Rslt<StatusCode,>
{
	let rsp_body = HtmlSource::new(version,).fetch()?;

	// Parse the HTML document
	let dom = html5ever::parse_document(RcDom::default(), Default::default(),)
		.one(rsp_body.as_str(),);

	let node = dom.document;

	// Find the main status codes section
	let main_section = get_element_by_id(node.clone(), MAIN_SECTION_ID,)?;

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
		.try_collect()?;
	let error_codes_info: Vec<Vec<String,>,> = error_code_table_rows
		.iter()
		.map(|n| table_data(n.clone(),),)
		.try_collect()?;
	let warn_codes_info: Vec<Vec<String,>,> = warn_code_table_rows
		.iter()
		.map(|n| table_data(n.clone(),),)
		.try_collect()?;

	// Convert raw table data to structured status code info
	let success_codes = status_codes_info(success_codes_info,)?;
	let mut error_codes = status_codes_info(error_codes_info,)?;
	let warn_codes = status_codes_info(warn_codes_info,)?;

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
			*a.name.local == *local_name && *a.value == *value
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
pub(crate) fn get_elements_by_attribute(
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

pub(crate) fn get_elements_by_name(
	node: Rc<Node,>,
	tag_name: &str,
) -> Vec<Rc<Node,>,>
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

pub(crate) fn table_rows(node: Rc<Node,>,) -> Vec<Rc<Node,>,>
{
	// Get all <tr> elements and skip the first one (header)
	get_elements_by_name(node.clone(), "tr",)[1..].to_vec()
}

pub(crate) fn table_data(node: Rc<Node,>,) -> Rslt<Vec<String,>,>
{
	let mut rslt = vec![];

	// Find all paragraph elements in the row (should be 3)
	let row = get_elements_by_name(node.clone(), "p",);

	// Extract text from the first cell (mnemonic)
	let NodeData::Text { ref contents, } =
		row[0].clone().children.borrow()[0].clone().data
	else {
		return Rslt::new_err(format!(
			"text node expected: {:#?}",
			row[0].clone()
		),);
	};
	rslt.push(contents.borrow().as_str().to_string(),);

	// Extract text from the second cell (value)
	let NodeData::Text { ref contents, } =
		row[1].clone().children.borrow()[0].clone().data
	else {
		return Rslt::new_err(format!(
			"text node expected: {:#?}",
			row[1].clone()
		),);
	};
	rslt.push(contents.borrow().as_str().to_string(),);

	// Extract text from the third cell (description)
	let NodeData::Text { ref contents, } =
		row[2].clone().children.borrow()[0].clone().data
	else {
		return Rslt::new_err(format!(
			"text node expected: {:#?}",
			row[2].clone()
		),);
	};
	rslt.push(contents.borrow().as_str().to_string(),);

	Rslt::new(rslt,)
}

pub(crate) fn status_codes_info(
	rows: Vec<Vec<String,>,>,
) -> Rslt<Vec<StatusCodeInfo,>,>
{
	rows.into_iter()
		.map(|row| {
			Rslt::new(StatusCodeInfo {
				mnemonic: row[0].clone(),
				// Parse the hex value string to integer
				value:    row[1].parse()?,
				desc:     row[2].clone(),
			},)
		},)
		.try_collect()
}

#[allow(dead_code)]
fn inspect_node(node: Rc<Node,>,) -> Diag
{
	Diag::note(format!("{node:#?}"),)
}
