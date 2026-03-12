#![feature(proc_macro_diagnostic)]

use poison_girl_proc_macro_helper::atr;

atr!{
	wrapper,
	[as syn::Ident,],
	[as syn::ItemTrait,],
	r#"Generates wrapper functions for trait methods.

This attribute macro takes a trait definition and generates corresponding
wrapper functions that delegate to a static instance. This is useful for
creating global function interfaces that wrap trait implementations.

# Parameters

* `attr` - The identifier of the static frame buffer or instance to delegate to
* `item` - The trait definition to generate wrappers for

# Returns

Returns the original trait definition along with generated wrapper functions.
Each trait method becomes a standalone function that calls the corresponding
method on the specified static instance.

# Generated Code

For each trait method, generates a function with:
- Same signature as the trait method (excluding `self` parameter)
- Same visibility, safety, async, const, and ABI attributes
- Delegation to the static instance method

# Examples

```rust,ignore
#[gen_wrapper_fn(GLOBAL_FRAMEBUFFER)]
trait Display {
    fn write_pixel(&mut self, x: u32, y: u32, color: u32);
    fn clear(&mut self);
}

// Generates:
// pub fn write_pixel(x: u32, y: u32, color: u32) {
//     GLOBAL_FRAMEBUFFER.write_pixel(x, y, color)
// }
// pub fn clear() {
//     GLOBAL_FRAMEBUFFER.clear()
// }
```

# Panics

This macro will cause a compile-time error if:
- The attribute is not a valid identifier
- The item is not a valid trait definition
- Any trait method has an unsupported signature"#,
}
