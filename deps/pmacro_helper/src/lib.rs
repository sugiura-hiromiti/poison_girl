#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

/// `MRslt` stand for *Macro Result*
pub struct MRslt<T>(Result<(T,Vec<Diag>));

impl<T> MRslt<T> {
	pub fn dewrap(self)-> Result<(T,Vec<Diag)> {
		self.0
	}
}

pub trait ErrorDiagnose {
	type T;
	fn unwrap_or_emit(self,) -> Self::T;
}

// impl<T,> ErrorDiagnose for anyhow::Result<(T, Vec<Diag,>,),> {
// 	type T = T;
//
// 	fn unwrap_or_emit(self,) -> Self::T {
// 		match self {
// 			Self::Ok((o, diag,),) => {
// 				diag.iter().for_each(|d| match d {
// 					Diag::Err(msg,) => {
// 						Diagnostic::new(Level::Error, msg,).emit()
// 					},
// 					Diag::Warn(msg,) => {
// 						Diagnostic::new(Level::Warning, msg,).emit()
// 					},
// 					Diag::Note(msg,) => {
// 						Diagnostic::new(Level::Note, msg,).emit()
// 					},
// 					Diag::Help(msg,) => {
// 						Diagnostic::new(Level::Help, msg,).emit()
// 					},
// 				},);
//
// 				o
// 			},
// 			Self::Err(e,) => {
// 				Diagnostic::new(Level::Error, format!("{e}"),).emit();
// 				panic!("{e}");
// 			},
// 		}
// 	}
// }

impl<T,> ErrorDiagnose for MRslt<T>{
	type T = T;

	fn unwrap_or_emit(self,) -> Self::T {
		match self.dewrap() {
			Ok((v, diag)) => {
				match diag {
					Diag::Err(msg)=>{
						compile_error!(msg);
					},
					Diag::Warn(msg,)=>{
						Diagnostic::new(Level::Warning,msg).emit(),
					}
					Diag::Note(msg)=>{
						Diagnostic::new(Level::Note, msg).emit()
					}
					Diag::Help(msg)=>{
						Diagnostic::new(Level::Help, msg).emit()
					}
				};

				v
			},
			Err(e)=>{
				compile_error!("{e}")
			}
		}
	}
}
