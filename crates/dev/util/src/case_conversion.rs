use {
	poison_girl_dev_error::{
		PathIsNotValidUtf8, PathNotFound, PoisonGirlB, ReShape, X, Y,
		poison_girl_err,
	},
	std::path::PathBuf,
};

pub trait StrEnhanced: CaseConvert + StringKind
{
}

pub trait CaseConvert
{
	type _Marker;
	fn is_camel(&self,) -> bool;
	fn is_snake(&self,) -> bool;
	fn is_screaming_snake(&self,) -> bool;
	fn is_kebab(&self,) -> bool;

	fn to_camel<S1: StringKind,>(&self,) -> S1
	{
		self.case_transit(
			|s| format!("{}{}", s[..1].to_ascii_uppercase(), &s[1..]),
			None,
		)
	}

	fn to_snake<S1: StringKind,>(&self,) -> S1
	{
		self.case_transit(|s| s.to_ascii_lowercase(), Some('_',),)
	}

	fn to_screaming_snake<S1: StringKind,>(&self,) -> S1
	{
		self.case_transit(|s| s.to_ascii_uppercase(), Some('_',),)
	}

	fn to_kebab<S1: StringKind,>(&self,) -> S1
	{
		self.case_transit(|s| s.to_ascii_lowercase(), Some('-',),)
	}

	fn case_transit<S: StringKind,>(
		&self,
		converter: impl FnMut(String,) -> String,
		spacer: Option<char,>,
	) -> S
	{
		let converted: Vec<_,> =
			self.words().into_iter().map(converter,).collect();
		let spacer = spacer.map_or("".to_string(), |c| c.to_string(),);
		let converted = converted.join(&spacer,);
		S::from(converted,)
	}

	fn find_spacer<S: StringKind,>(&self,) -> Option<S,>;
	fn words(&self,) -> Vec<String,>;
	fn as_string_kind(&self,) -> Option<&impl StringKind,>;
}

pub trait StringKind
{
	type DumpReturn;
	fn dump_string(&self,) -> Self::DumpReturn;
	fn from(s: impl Into<String,>,) -> Self;
	fn as_case_convert(&self,) -> Option<&impl CaseConvert,>;
}

impl StrEnhanced for String
{
}

impl CaseConvert for String
{
	type _Marker = String;

	fn is_camel(&self,) -> bool
	{
		is_xxx_format_with_case(self.clone(), None, Form::StartWithUpper,)
	}

	fn is_snake(&self,) -> bool
	{
		is_xxx_format_with_case(self.clone(), Some('_',), Form::Lower,)
	}

	fn is_screaming_snake(&self,) -> bool
	{
		is_xxx_format_with_case(self.clone(), Some('_',), Form::Upper,)
	}

	fn is_kebab(&self,) -> bool
	{
		is_xxx_format_with_case(self.clone(), Some('-',), Form::Lower,)
	}

	fn find_spacer<S1: StringKind,>(&self,) -> Option<S1,>
	{
		let s: String = self.clone();
		if s.contains("_",) {
			Some(S1::from("_".to_string(),),)
		} else if s.contains("-",) {
			Some(S1::from("-".to_string(),),)
		} else {
			None
		}
	}

	fn words(&self,) -> Vec<String,>
	{
		let s: String = self.clone();
		if self.is_camel() {
			let mut rslt = vec![];
			let mut idx = 0;
			while let Some(sub,) = s.get(idx + 1..,)
				&& let Some(tail,) = sub.find(|c: char| c.is_ascii_uppercase(),)
			{
				// tail is relative to sub, so we need to add idx + 1 to get the
				// absolute position
				let absolute_pos = idx + 1 + tail;
				rslt.push(s[idx..absolute_pos].to_string(),);
				idx = absolute_pos; // Move to the position of the uppercase letter
			}
			// Add the remaining part if any
			if let Some(remaining,) = s.get(idx..,)
				&& !remaining.is_empty()
			{
				rslt.push(remaining.to_string(),);
			}
			rslt
		} else {
			// Cache the spacer to avoid repeated calls
			let spacer = s.find_spacer().unwrap_or(" ".to_string(),);
			s.split(|c: char| spacer == c.to_string(),)
				.map(|s| s.to_string(),)
				.collect()
		}
	}

	#[allow(refining_impl_trait)]
	fn as_string_kind(&self,) -> Option<&Self,>
	{
		Some(self,)
	}
}

impl StringKind for String
{
	type DumpReturn = Self;

	fn dump_string(&self,) -> Self::DumpReturn
	{
		self.clone()
	}

	fn from(s: impl Into<String,>,) -> Self
	{
		s.into()
	}

	#[allow(refining_impl_trait)]
	fn as_case_convert(&self,) -> Option<&Self,>
	{
		Some(self,)
	}
}

enum Form
{
	StartWithUpper,
	Upper,
	Lower,
}

fn is_xxx_format_with_case(
	s: impl Into<String,> + Clone,
	spacer: Option<char,>,
	form: Form,
) -> bool
{
	let s: String = s.into();

	let spacer_checker = || -> Box<dyn Fn(char,) -> bool,> {
		match spacer {
			Some(spacer,) => Box::new(move |c| c == spacer,),
			None => Box::new(|c| c.is_ascii_alphanumeric(),),
		}
	};
	let checker = || -> Box<dyn Fn(&String,) -> bool,> {
		match form {
			Form::StartWithUpper => Box::new(|s| {
				s.starts_with(|c: char| c.is_ascii_uppercase(),)
					&& s.chars().all(|c| {
						c.is_ascii_alphanumeric() && spacer_checker()(c,)
					},)
			},),
			Form::Upper => Box::new(|s| {
				s.chars().all(|c| {
					c.is_ascii_uppercase()
						|| c.is_numeric() || spacer_checker()(c,)
				},)
			},),
			Form::Lower => Box::new(|s| {
				s.chars().all(|c| {
					c.is_ascii_lowercase()
						|| c.is_numeric() || spacer_checker()(c,)
				},)
			},),
		}
	};

	checker()(&s,)
}

impl StrEnhanced for PathBuf
{
}

impl CaseConvert for PathBuf
{
	type _Marker = PathBuf;

	fn is_camel(&self,) -> bool
	{
		self.dump_string().is_x_and(|s| s.is_camel(),)
	}

	fn is_snake(&self,) -> bool
	{
		self.dump_string().is_x_and(|s| s.is_snake(),)
	}

	fn is_screaming_snake(&self,) -> bool
	{
		self.dump_string().is_x_and(|s| s.is_screaming_snake(),)
	}

	fn is_kebab(&self,) -> bool
	{
		self.dump_string().is_x_and(|s| s.is_kebab(),)
	}

	fn find_spacer<S: StringKind,>(&self,) -> Option<S,>
	{
		// let spacer = self.dump_string().map(|s| s.find_spacer(),)?;
		// Some(spacer,)
		match self.dump_string() {
			X(s,) => s.find_spacer(),
			Y(e,) => {
				// to avoid panic! macro
				eprintln!("{e}");
				None
			},
		}
	}

	fn words(&self,) -> Vec<String,>
	{
		match self.dump_string() {
			X(s,) => s.words(),
			Y(e,) => {
				eprintln!("{e}");
				vec![]
			},
		}
	}

	#[allow(refining_impl_trait)]
	fn as_string_kind(&self,) -> Option<&Self,>
	{
		Some(self,)
	}
}

impl StringKind for PathBuf
{
	type DumpReturn = PoisonGirlB<String,>;

	fn dump_string(&self,) -> Self::DumpReturn
	{
		self.file_prefix()
			.reshape(poison_girl_err!(PathNotFound(
				"failed to get file/dir name".to_string()
			)),)?
			.to_str()
			.reshape(poison_girl_err!(PathIsNotValidUtf8),)
			.map(|s| s.to_string(),)
	}

	#[cold]
	#[track_caller]
	#[expect(
		clippy::unreachable,
		reason = "indicates method that should not use but required to \
		          implement trait"
	)]
	fn from(_: impl Into<String,>,) -> Self
	{
		unreachable!("you should not use `PathBuf::from`")
	}

	#[allow(refining_impl_trait)]
	fn as_case_convert(&self,) -> Option<&Self,>
	{
		Some(self,)
	}
}

#[cfg(test)] mod tests;
