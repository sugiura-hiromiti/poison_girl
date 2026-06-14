use {
	super::table::system_table,
	crate::raw::protocol::text::TextOutputProtocol,
	poison_girl_no_std_error::{X, Y},
};

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

pub fn print(args: core::fmt::Arguments,)
{
	use core::fmt::Write;
	if let X(st,) = system_table()
		&& let Some(stdout,) = unsafe { st.as_ref().stdout.as_mut() }
	{
		let _ = stdout.write_fmt(args,);
	}
}

impl core::fmt::Write for TextOutputProtocol
{
	fn write_str(&mut self, s: &str,) -> core::fmt::Result
	{
		match self.output(s,) {
			X(_s,) => Ok((),),
			Y(_e,) => Err(core::fmt::Error,),
		}
	}
}
