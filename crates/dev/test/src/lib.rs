#![feature(try_trait_v2)]

use std::{
	convert::Infallible,
	fmt::Display,
	ops::{FromResidual, Try},
	process::Termination,
};
pub use this_is_b::{B::*, *};

pub struct PoisonGirlTestB(B<(), String,>,);

impl PoisonGirlTestB
{
	pub fn x() -> Self
	{
		Self(X((),),)
	}

	pub fn y(m: impl Display,) -> Self
	{
		Self(Y(m.to_string(),),)
	}
}

impl AsRef<B<(), String,>,> for PoisonGirlTestB
{
	fn as_ref(&self,) -> &B<(), String,>
	{
		&self.0
	}
}

impl Termination for PoisonGirlTestB
{
	fn report(self,) -> std::process::ExitCode
	{
		match self.as_ref() {
			X(_,) => std::process::ExitCode::SUCCESS,
			Y(m,) => {
				eprintln!("{m}");
				std::process::ExitCode::FAILURE
			},
		}
	}
}

impl Try for PoisonGirlTestB
{
	type Output = ();
	type Residual = B<Infallible, String,>;

	fn from_output(output: Self::Output,) -> Self
	{
		Self(X(output,),)
	}

	fn branch(self,) -> std::ops::ControlFlow<Self::Residual, Self::Output,>
	{
		let Self(b,) = self;
		match b {
			X(o,) => std::ops::ControlFlow::Continue(o,),
			Y(m,) => std::ops::ControlFlow::Break(Y(m,),),
		}
	}
}

impl<M: Display,> FromResidual<B<Infallible, M,>,> for PoisonGirlTestB
{
	#[track_caller]
	#[expect(clippy::unreachable, reason = "required by try trait v2 system")]
	fn from_residual(residual: B<Infallible, M,>,) -> Self
	{
		match residual {
			X(_,) => unreachable!(),
			Y(m,) => Self(Y(m.to_string(),),),
		}
	}
}

// `PoisonGirlTestB`が返り値の時でも`Result`に?演算子を使えるようにする
impl<M: Display,> FromResidual<Result<Infallible, M,>,> for PoisonGirlTestB
{
	#[track_caller]
	#[expect(clippy::unreachable, reason = "required by try trait v2 system")]
	fn from_residual(residual: Result<Infallible, M,>,) -> Self
	{
		match residual {
			Ok(_,) => unreachable!(),
			Err(m,) => PoisonGirlTestB(Y(m.to_string(),),),
		}
	}
}

// `PoisonGirlTestB`が返り値の時でも`Option`に?演算子を使えるようにする
impl FromResidual<Option<Infallible,>,> for PoisonGirlTestB
{
	#[track_caller]
	#[expect(clippy::unreachable, reason = "required by try trait v2 system")]
	fn from_residual(residual: Option<Infallible,>,) -> Self
	{
		match residual {
			Some(_,) => unreachable!(),
			None => PoisonGirlTestB(Y("std::option::Option::None".into(),),),
		}
	}
}

#[macro_export]
macro_rules! fail {
	($msg:expr) => {
		return $crate::PoisonGirlTestB::y($msg,)
	};
}

#[macro_export]
macro_rules! success {
	() => {{
		return $crate::PoisonGirlTestB::x();
	}};
}

#[cfg(test)]
mod tests
{
	use super::*;

	fn b_residual() -> PoisonGirlTestB
	{
		let value: B<(), &str,> = Y("b failed",);
		value?;
		success!()
	}

	fn result_residual() -> PoisonGirlTestB
	{
		let value: Result<(), &str,> = Err("result failed",);
		value?;
		success!()
	}

	fn option_residual() -> PoisonGirlTestB
	{
		let value: Option<(),> = None;
		value?;
		success!()
	}

	#[test]
	fn converts_b_residual_to_test_failure()
	{
		assert!(
			matches!(b_residual().as_ref(), Y(message) if message == "b failed")
		);
	}

	#[test]
	fn converts_result_residual_to_test_failure()
	{
		assert!(
			matches!(result_residual().as_ref(), Y(message) if message == "result failed")
		);
	}

	#[test]
	fn converts_option_residual_to_test_failure()
	{
		assert!(matches!(
			option_residual().as_ref(),
			Y(message) if message == "std::option::Option::None"
		));
	}
}
