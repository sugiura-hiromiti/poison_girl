use crate::rslt::Rslt;

pub type TestRslt = Rslt<(),>;

#[macro_export]
macro_rules! fail {
	($msg:expr) => {{
		return $crate::rslt::test_helper::TestRslt::new_err($msg,);
	}};
}

#[macro_export]
macro_rules! success {
	() => {{
		return $crate::rslt::test_helper::TestRslt::new((),);
	}};
}
