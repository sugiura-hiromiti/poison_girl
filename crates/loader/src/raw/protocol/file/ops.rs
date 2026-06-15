mod v1;
mod v2;

pub(super) use self::{
	v1::{
		FileClose, FileDelete, FileFlush, FileGetInfo, FileGetPosition,
		FileOpen, FileRead, FileSetInfo, FileSetPosition, FileWrite,
	},
	v2::{FileFlushEx, FileOpenEx, FileReadEx, FileWriteEx},
};
