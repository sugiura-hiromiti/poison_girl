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
	drop_fmt_result(try_print(args,),);
}

fn try_print(args: core::fmt::Arguments,) -> core::fmt::Result
{
	use core::fmt::Write;
	if let X(st,) = system_table()
		&& let Some(stdout,) = unsafe { st.as_ref().stdout.as_mut() }
	{
		return stdout.write_fmt(args,);
	}
	Ok((),)
}

fn drop_fmt_result(result: core::fmt::Result,)
{
	match result {
		Ok((),) | Err(_,) => (),
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
