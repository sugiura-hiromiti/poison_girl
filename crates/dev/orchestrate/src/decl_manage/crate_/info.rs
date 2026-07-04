use {
	super::{CrateCalled, PoisonGirlCrate},
	crate::decl_manage::PoisonGirlPackageMetadata,
	poison_girl_dev_error::{
		PathIsNotValidUtf8, PathNotFound, PoisonGirlB, ReShape, X,
		poison_girl_err,
	},
	poison_girl_dev_fs::{CARGO_CONFIG, CARGO_MANIFEST, read_toml},
	poison_girl_dev_util::toml_tools::TomlMerge,
	std::path::PathBuf,
};

pub trait CrateInfo: CrateCalled
{
	fn is_package(&self,) -> PoisonGirlB<bool,>
	{
		let pkg_sec = self.toml()?;
		let pkg_sec = pkg_sec.get("package",);
		match pkg_sec {
			Some(_,) => X(true,),
			None => X(false,),
		}
	}

	fn is_workspace(&self,) -> PoisonGirlB<bool,>
	{
		let pkg_sec = self.toml()?;
		let pkg_sec = pkg_sec.get("workspace",);
		match pkg_sec {
			Some(_,) => X(true,),
			None => X(false,),
		}
	}

	fn is_pkg_and_ws(&self,) -> PoisonGirlB<bool,>
	{
		X(self.is_package()? && self.is_workspace()?,)
	}

	/// return path to the crate
	/// return type is not Result because macro ensures that path exists and
	/// self has path in compile time
	fn path(&self,) -> PathBuf;

	fn toml(&self,) -> PoisonGirlB<toml::Table,>
	{
		let cargo_toml = self.path().join(CARGO_MANIFEST,);
		read_toml(cargo_toml,)
			.map(|toml| toml.unwrap_or(toml::map::Map::new(),),)
	}

	fn custom_metadata(&self,) -> PoisonGirlB<PoisonGirlPackageMetadata,>
	{
		let manifest = self.toml()?;
		let metadata_table = PoisonGirlPackageMetadata::METADATA_PATH
			.into_iter()
			.try_fold(manifest, |mut acc, segment| {
				let toml::Value::Table(table,) = acc.remove(segment,)? else {
					return None;
				};

				Some(table,)
			},);

		match metadata_table {
			Some(table,) => PoisonGirlPackageMetadata::from_toml_table(&table,),
			None => X(PoisonGirlPackageMetadata::default(),),
		}
	}

	/// NOTE: we've changed return type from `PoisonGirlB<Option<toml::Table>>`
	/// to `PoisonGirlB<toml::Table>` this is because, `cargo.toml` is
	/// originally optional and it has default fallback. so empty `cargo.toml`
	/// is completely vaild, in this case, distinguishing the file existence is
	/// meaningless. instead, this function provides simple semantics: the
	/// config value exists or not.
	fn cargo_conf(&self,) -> PoisonGirlB<toml::Table,>
	{
		// let config_toml = self.path().join(CARGO_CONFIG,);
		// if !config_toml.exists() {
		// 	return None;
		// }
		// Some(read_toml(config_toml,),)
		let mut path = self.path().join(CARGO_CONFIG,);
		// we can treat this as true depth of the path because `path` is
		// canonicalized.
		let depth = path.components().collect::<Vec<_,>>().len() - 1;
		let global_cargo_config_path = std::env::var("CARGO_HOME",)
			.map_or_else(|_| std::env::home_dir(), |s| Some(PathBuf::from(s,),),)
			.reshape(poison_girl_err!(PathNotFound::new("home directory")),)?;

		path.pop();
		(0..depth)
			.map(|_| {
				path.pop();
				// path.push(CARGO_CONFIG,);
				path.clone()
				// read_toml(&path,)
			},)
			.chain([global_cargo_config_path,],)
			.rev()
			.map(|mut p| {
				p.push(CARGO_CONFIG,);
				p
			},)
			.map(read_toml,)
			.try_fold(toml::Table::new(), |acc, config| {
				let config = config?;
				let Some(config,) = config else { return X(acc,) };
				let acc = acc.into_updated_by(config,);
				X(acc,)
			},)
	}

	fn name(&self,) -> PoisonGirlB<String,>
	{
		self.path()
			.file_name()
			.reshape(poison_girl_err!(PathNotFound::new(
				"path of the crate is terminated with .. or root",
			)),)?
			.to_str()
			.reshape(poison_girl_err!(PathIsNotValidUtf8),)
			.map(|s| s.to_string(),)
	}
}

impl CrateInfo for PoisonGirlCrate
{
	fn path(&self,) -> PathBuf
	{
		self.path.clone()
	}
}
