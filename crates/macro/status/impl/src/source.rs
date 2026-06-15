use {poison_girl_macro_error::rslt::Rslt, std::path::PathBuf};

const CRATE_ROOT: &str = env!("CARGO_MANIFEST_DIR");
pub(crate) struct HtmlSource
{
	version: syn::LitFloat,
}

impl HtmlSource
{
	pub(crate) fn new(version: syn::LitFloat,) -> Self
	{
		Self { version, }
	}

	pub(crate) fn fetch(&self,) -> Rslt<String,>
	{
		let local_path = PathBuf::from(CRATE_ROOT,)
			.join(format!("status_{}.html", self.version),);

		if !std::fs::exists(&local_path,)? {
			return Rslt::new_err(format!(
				"file: {} not found",
				local_path.display()
			),);
		}

		Rslt::new(std::fs::read_to_string(local_path,)?,)
	}
}
