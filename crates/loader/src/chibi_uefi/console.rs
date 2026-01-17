use super::table::system_table;
use crate::raw::protocol::text::TextOutputProtocol;

#[macro_export]
macro_rules! print {
	($($args:tt)*) => {
		$crate::chibi_uefi::console::print(core::format_args!($($args)*),);
	};
}

#[macro_export]
macro_rules! println {
	() => {
		$crate::print!("\n");
	};
	($($args:tt)*)=>{
		print!("{}{}", core::format_args!($($args)*), "\n");
	}
}

pub fn print(args: core::fmt::Arguments,) {
	use core::fmt::Write;
	let st = unsafe { system_table().as_ref() };
	unsafe { st.stdout.as_mut() }.unwrap().write_fmt(args,).unwrap();
}

impl core::fmt::Write for TextOutputProtocol {
	fn write_str(&mut self, s: &str,) -> core::fmt::Result {
		self.output(s,)?;
		Ok((),)
	}
}
