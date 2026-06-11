#![feature(proc_macro_diagnostic)]

use {
	poison_girl_macro_impl_impl_int as poison_girl_proc_macro_impl,
	poison_girl_proc_macro_helper::fnl,
};

fnl! {
	impl_int,
	[as poison_girl_proc_macro_impl::Types,],
	r#"Generates implementations for integer types.

This procedural macro takes a list of types and generates implementations
for them using the logic defined in the `poison_girl_proc_macro_logic::impl_init` module.
It's typically used to reduce boilerplate when implementing common traits
or methods for multiple integer types.

# Parameters

* `types` - A token stream representing the types to implement. The format should match the
  `Types` parser in the logic module.

# Returns

Returns a token stream containing the generated implementations for all
specified types.

# Examples

```rust,ignore
// Generate implementations for u8, u16, u32, u64
impl_int!(u8, u16, u32, u64);
```

# Panics

This macro will cause a compile-time error if:
- The input cannot be parsed as valid types
- The implementation logic fails for any of the specified types"#
}
