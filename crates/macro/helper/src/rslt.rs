use {
	crate::diagnostic::{Diag, ErrDiag, NotationDiag},
	poison_girl_dev_error::PoisonGirlB,
	std::{
		convert::Infallible,
		fmt::Debug,
		ops::{FromResidual, Try},
		process::Termination,
	},
};

pub struct Rslt<V,>
{
	val:      Option<V,>,
	notation: Vec<NotationDiag,>,
	err:      Option<ErrDiag,>,
}

impl<V,> Default for Rslt<V,>
{
	fn default() -> Self
	{
		Self {
			val:      Default::default(),
			notation: Default::default(),
			err:      Default::default(),
		}
	}
}

impl<V,> Rslt<V,>
{
	pub fn new(val: V,) -> Self
	{
		Self { val: Some(val,), notation: vec![], err: None, }
	}

	pub fn new_err(e: impl Debug,) -> Self
	{
		Self {
			val:      None,
			notation: vec![],
			err:      Some(ErrDiag::new(format!("{e:?}"),),),
		}
	}

	pub fn with_err(mut self, err: ErrDiag,) -> Self
	{
		self.err = Some(err,);
		self
	}

	pub fn inject_err(mut self, err: Option<ErrDiag,>,) -> Self
	{
		if !self.has_err() && err.is_some() {
			self.err = err;
		}
		self
	}

	pub fn add_notation(mut self, nt: NotationDiag,) -> Self
	{
		self.notation.push(nt,);
		self
	}

	pub fn add_notations(mut self, mut nts: Vec<NotationDiag,>,) -> Self
	{
		self.notation.append(&mut nts,);
		self
	}

	pub fn with_diag(self, diag: impl Into<Diag,>,) -> Self
	{
		match diag.into() {
			Diag::Err(err_diag,) => self.with_err(err_diag,),
			Diag::Notation(notation_diag,) => self.add_notation(notation_diag,),
		}
	}

	pub fn with_diags(self, diags: Vec<impl Into<Diag,>,>,) -> Self
	{
		diags.into_iter().fold(self, |acc, diag| acc.with_diag(diag,),)
	}

	pub fn has_err(&self,) -> bool
	{
		self.err.is_some()
	}

	pub fn err(&self,) -> Option<&ErrDiag,>
	{
		self.err.as_ref()
	}

	pub fn into_err(self,) -> Option<ErrDiag,>
	{
		self.err
	}

	pub fn value(&self,) -> Option<&V,>
	{
		self.val.as_ref()
	}

	pub fn value_mut(&mut self,) -> Option<&mut V,>
	{
		self.val.as_mut()
	}

	pub fn notation(&self,) -> &[NotationDiag]
	{
		&self.notation
	}

	pub fn into_value(self,) -> Option<V,>
	{
		self.val
	}

	pub fn into_notation(self,) -> Vec<NotationDiag,>
	{
		self.notation
	}

	// pub fn unwrap(self,) -> Option<V,>
	// {
	// 	match self.err {
	// 		Some(e,) => panic!("Error Diagnostic: {e:?}"),
	// 		None => self.into_value(),
	// 	}
	// }

	pub fn replace<V2,>(self, val: V2,) -> Rslt<V2,>
	{
		let Self { notation, err, .. } = self;
		Rslt { val: Some(val,), notation, err, }
	}

	pub fn replace_by<V2,>(self, f: impl FnOnce(V,) -> Rslt<V2,>,)
	-> Rslt<V2,>
	{
		let Self { val, notation, err, } = self;
		match val {
			Some(v,) => {
				let new = f(v,).add_notations(notation,);
				match (new.has_err(), err,) {
					(false, Some(e,),) => new.with_err(e,),
					_ => new,
				}
			},
			None => Rslt { val: None, notation, err, },
		}
	}
}

impl<V,> Rslt<Vec<V,>,>
{
	pub fn push_elem(self, one: Rslt<V,>,) -> Self
	{
		let Rslt { val, notation, err, } = one;
		let Rslt { val: val2, notation, err, } =
			self.inject_err(err,).add_notations(notation,);

		let val = match (val, val2,) {
			(None, v,) => v,
			(Some(v,), None,) => Some(vec![v],),
			(Some(val,), Some(mut vval,),) => {
				vval.push(val,);
				Some(vval,)
			},
		};

		Self { val, notation, err, }
	}
}

impl<V,> std::ops::Residual<V,> for Rslt<Infallible,>
{
	type TryType = Rslt<V,>;
}

impl<V,> Try for Rslt<V,>
{
	type Output = V;
	type Residual = Rslt<Infallible,>;

	fn from_output(output: Self::Output,) -> Self
	{
		Self { val: Some(output,), notation: vec![], err: None, }
	}

	fn branch(self,) -> std::ops::ControlFlow<Self::Residual, Self::Output,>
	{
		let Self { notation, err, .. } = self;
		std::ops::ControlFlow::Break(Rslt { val: None, notation, err, },)
	}
}

impl<V,> FromResidual for Rslt<V,>
{
	fn from_residual(residual: <Self as Try>::Residual,) -> Self
	{
		let Rslt { notation, err, .. } = residual;
		Self { val: None, notation, err, }
	}
}

impl<V,> FromResidual<PoisonGirlB<Infallible,>,> for Rslt<V,>
{
	#[expect(clippy::unreachable, reason = "necessary")]
	fn from_residual(residual: PoisonGirlB<Infallible,>,) -> Self
	{
		match residual {
			poison_girl_dev_error::X(_,) => unreachable!(),
			poison_girl_dev_error::Y(e,) => Rslt::new_err(e,),
		}
	}
}

impl<V, E: Debug,> FromResidual<Result<Infallible, E,>,> for Rslt<V,>
{
	#[expect(clippy::unreachable, reason = "necessary")]
	fn from_residual(residual: Result<Infallible, E,>,) -> Self
	{
		match residual {
			Ok(_,) => unreachable!(),
			Err(e,) => Rslt::new_err(e,),
		}
	}
}

impl<V,> FromResidual<Option<Infallible,>,> for Rslt<V,>
{
	#[expect(clippy::unreachable, reason = "necessary")]
	fn from_residual(residual: Option<Infallible,>,) -> Self
	{
		match residual {
			Some(_,) => unreachable!(),
			None => Rslt::new_err("option is none",),
		}
	}
}

impl<V,> Termination for Rslt<V,>
{
	fn report(self,) -> std::process::ExitCode
	{
		if self.has_err() {
			std::process::ExitCode::FAILURE
		} else {
			std::process::ExitCode::SUCCESS
		}
	}
}

#[cfg(test)]
mod tests
{

	use super::*;

	#[test]
	fn test_multiple_module_interaction() -> anyhow::Result<(),>
	{
		// Test that modules can work together without conflicts

		// Create some diagnostics
		let diags = vec![
			Diag::err("Error from module interaction",),
			Diag::warn("Warning from module interaction",),
		];

		// Test that we can create a result with diagnostics
		let result =
			Rslt::new(quote::quote! { fn test() {} },).with_diags(diags,);
		assert!(!result.has_err());
		assert_eq!(result.notation().len(), 2);

		let tokens = result.unwrap();
		assert!(!tokens.ok_or(anyhow!("no token parsed"))?.is_empty());
		Ok((),)
	}

	#[test]
	fn test_rslt_p_complex_scenarios() -> anyhow::Result<(),>
	{
		// Test RsltP with complex token streams and multiple diagnostics

		fn complex_function() -> Rslt<proc_macro2::TokenStream,>
		{
			let complex_tokens = quote::quote! {
				pub struct ComplexStruct<T> where T: Clone + Send + Sync {
					field1: T,
					field2: Option<T>,
					field3: Vec<T>,
				}

				impl<T> ComplexStruct<T> where T: Clone + Send + Sync {
					pub fn new(value: T) -> Self {
						Self {
							field1: value.clone(),
							field2: Some(value.clone()),
							field3: vec![value],
						}
					}
				}
			};

			let complex_diags = vec![
				Diag::note("Complex structure created",),
				Diag::warn("This is a test warning",),
				Diag::help("Consider using simpler types",),
			];

			Rslt::new(complex_tokens,).with_diags(complex_diags,)
		}

		let result = complex_function();
		assert!(!result.has_err());

		assert_eq!(result.notation().len(), 3);
		let tokens = result.unwrap();
		assert!(!tokens.as_ref().ok_or(anyhow!("no token parsed"))?.is_empty());

		// Verify token stream contains expected content
		let token_str = tokens.ok_or(anyhow!("no token parsed"),)?.to_string();
		assert!(token_str.contains("ComplexStruct"));
		assert!(token_str.contains("Clone"));
		assert!(token_str.contains("Send"));
		assert!(token_str.contains("Sync"));
		Ok((),)
	}
}
