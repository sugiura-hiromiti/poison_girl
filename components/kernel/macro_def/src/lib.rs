#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use crate::pmh::atr;
use crate::pmh::drv;
use crate::pmh::fnl;
use oso_proc_macro_logic::oso_proc_macro_helper::Diag;
use proc_macro::Diagnostic;
use proc_macro::Level;
use shion_pmacro_helper as pmh;

fnl!(font => syn::LitStr,
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
);
