#![no_std]

pub use poison_girl_proc_macro_def::*;

#[macro_export]
macro_rules! cfg_if {
	(
		if #[cfg($($ic_token:tt)+)] {
			$( $i_token:tt )*
		} $(else if #[cfg($($eic_token:tt)+)] {
			$( $ei_token:tt )*
		})* $(else { $( $e_token:tt )* })?
	) => {
		$crate::cfg_if! {
			__items__();
			( ($($ic_token)*) ($($i_token)*) ),
			$(
				( ($($eic_token)+) ($($ei_token)*) ),
			)*
			$(
				( () ($($e_token)*) ),
			)?
		}
	};
	(__items__($(($($_:tt)*),)*);) => {};
	(
		__items__($(($($no:tt)+),)*);
		(($($($yes:tt)+)?)($($tokens:tt)*)),
		$($rest:tt,)*
	) => {
		#[cfg(all(
			$( $($yes)+ , )?
			not(any( $( $($no)+ ),* ))
		))]
		$crate::cfg_if! { __tmp__ $($tokens)* }

		$crate::cfg_if! {
			__items__( $( ($($no)+) , )* $( ($($yes)+) , )? );
			$( $rest , )*
		}
	};
	(__tmp__ $($tokens:tt)*) => { $($tokens)* };
}
