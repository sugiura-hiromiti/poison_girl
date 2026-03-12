#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_font as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::fnl,
};

fnl! {
	font,
	[as syn::LitStr,],
	r#"Generates embedded font data from font files at compile time.

This procedural macro takes a relative path to the project root and processes
font files to generate embedded data structures that can be used at runtime.
The macro converts font data into bitfield representations for efficient storage.

# Parameters

* `path` - A string literal containing the relative path from the project root to the directory
  containing font data files

# Returns

Returns a token stream representing an array slice of processed font data.
The generated code will be in the form `&[font_data_1, font_data_2, ...]`.

# Examples

```rust,ignore
// Generate font data from files in the "assets/fonts" directory
let fonts = fonts_data!("assets/fonts");
```

# Panics

This macro will cause a compile-time error if:
- The specified path does not exist
- Font files in the path cannot be processed
- The path parameter is not a valid string literal"#
}
