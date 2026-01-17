#[macro_export]
/// Usage example:
/// ```ignore
/// # use oso_loader::raw::types::util::c_style_enum;
/// c_style_enum! {
/// #[derive(Default)]
/// pub enum UnixBool: i32 => #[allow(missing_docs)] {
/// 	FALSE          = 0,
/// 	TRUE           = 1,
/// 	FILE_NOT_FOUND = -1,
/// }}
/// ```
macro_rules! c_style_enum {
	(
		$(#[$type_attrs:meta])*
		$visibility:vis enum $type:ident : $base_integer:ty => $(#[$impl_attrs:meta])* {
			$(
				$(#[$variant_attrs:meta])*
				$variant:ident = $value:expr,
			)*
		}
	) => {
		$(#[$type_attrs])*
		#[repr(transparent)]
		#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
		$visibility struct $type(pub $base_integer);

		$(#[$impl_attrs])*
		#[allow(unused)]
		impl $type {
			$(
				$(#[$variant_attrs])*
				pub const $variant: $type = $type($value);
			)*
		}

		impl core::fmt::Debug for $type {
			fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
				match *self {
					$(
						$type::$variant => write!(f, stringify!($variant)),
					)*
					$type(unknown) => {
						write!(f, "{}({})", stringify!($type), unknown)
					}
				}
			}
		}
	};
}
