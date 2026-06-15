use {
	super::*,
	html5ever::{QualName, local_name, ns, tendril::TendrilSink},
	markup5ever_rcdom::{Node, RcDom},
	poison_girl_macro_error::{rslt::test_helper::TestRslt, success},
	std::rc::Rc,
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
fn test_get_elements_by_attribute()
{
	let node = parse_text(BASIC_HTML,);
	let class_wow = get_elements_by_attribute(node.clone(), "class", "wow",);
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
	let table_node = get_elements_by_name(node.clone(), "table",)[0].clone();
	let rows = table_rows(table_node,);

	// Should return 2 rows (excluding header)
	assert_eq!(rows.len(), 2);
}

#[test]
fn test_table_data_extraction() -> TestRslt
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
	let data = table_data(row_node[0].clone(),)?;

	assert_eq!(data.len(), 3);
	assert_eq!(data[0], "EFI_SUCCESS");
	assert_eq!(data[1], "0x00000000");
	assert_eq!(data[2], "The operation completed successfully.");
	success!()
}

#[test]
fn test_status_codes_info_conversion() -> TestRslt
{
	let raw_data = vec![
		vec!["EFI_SUCCESS".to_string(), "0".to_string(), "Success".to_string()],
		vec![
			"EFI_LOAD_ERROR".to_string(),
			"1".to_string(),
			"Load error".to_string(),
		],
	];

	let status_codes = status_codes_info(raw_data,)?;

	assert_eq!(status_codes.len(), 2);
	assert_eq!(status_codes[0].mnemonic, "EFI_SUCCESS");
	assert_eq!(status_codes[0].value, 0);
	assert_eq!(status_codes[1].mnemonic, "EFI_LOAD_ERROR");
	assert_eq!(status_codes[1].value, 1);
	success!()
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
	let status_elements = get_elements_by_attribute(node, "class", "status",);

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
	let by_data = get_elements_by_attribute(node.clone(), "data-value", "123",);
	assert_eq!(by_data.len(), 1);

	// Test style attribute search
	let by_style = get_elements_by_attribute(node, "style", "color: red",);
	assert_eq!(by_style.len(), 1);
}
