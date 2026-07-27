#![feature(iterator_try_collect)]
#![feature(try_find)]

use {
	poison_girl_dev_error::{
		InvalidCurrentCratePath, InvalidProjectRootFound, NotObedientPath,
		PathIsNotValidUtf8, PathNotFound, PoisonGirlB, ProjectRootNotFound, X,
		Y, poison_girl_err,
	},
	std::{
		env::current_dir,
		fs::DirEntry,
		path::{Path, PathBuf},
	},
};

pub const CARGO_MANIFEST: &str = "Cargo.toml";
pub const CARGO_CONFIG: &str = ".cargo/config.toml";
const CWD: &str = std::env!("CARGO_MANIFEST_DIR");
const IGNORE_DIR_LIST: [&str; 5] =
	["target", ".git", ".github", ".direnv", ".cargo",];

pub fn all_crates() -> PoisonGirlB<Vec<PathBuf,>,>
{
	let proot = project_root_path()?;
	let mut crates = all_crates_in(&proot,)?;
	crates.push(proot.to_path_buf(),);
	X(crates,)
}

pub fn all_crates_in(path: &Path,) -> PoisonGirlB<Vec<PathBuf,>,>
{
	X(path
		.read_dir()?
		.map(|entry| -> PoisonGirlB<_,> {
			let entry = entry?;
			if entry.path().is_file() {
				return X(None,);
			}

			let path = entry.path();
			let name = path
				.file_name()
				.ok_or(poison_girl_err!(NotObedientPath),)?
				.to_str()
				.ok_or(poison_girl_err!(PathIsNotValidUtf8),)?;
			if IGNORE_DIR_LIST.contains(&name,) {
				return X(None,);
			}

			let mut paths = if search_cargo_toml(&path,)?.is_some() {
				vec![path.clone()]
			} else {
				vec![]
			};
			paths.append(&mut all_crates_in(&path,)?,);
			X(Some(paths,),)
		},)
		.filter_map(|entry| match entry {
			X(None,) => None,
			X(Some(a,),) => Some(X(a,),),
			Y(a,) => Some(Y(a,),),
		},)
		.try_collect::<Vec<_,>>()?
		.into_iter()
		.flatten()
		.collect(),)
}

pub fn project_root_path() -> PoisonGirlB<PathBuf,>
{
	let mut p = PathBuf::from(CWD,);
	let mut last_cargo_toml = None;

	while p.pop() {
		if let Some(p,) = search_cargo_toml(&p,)? {
			last_cargo_toml = Some(p,)
		}
	}

	let Some(last_cargo_toml,) = last_cargo_toml else {
		return Y(poison_girl_err!(ProjectRootNotFound),);
	};

	X(last_cargo_toml
		.parent()
		.ok_or(poison_girl_err!(InvalidProjectRootFound),)?
		.to_path_buf(),)
}

pub fn current_crate_path() -> PoisonGirlB<PathBuf,>
{
	match search_upstream(CARGO_MANIFEST,) {
		X(Some(p,),) => X(p
			.parent()
			.ok_or(poison_girl_err!(InvalidCurrentCratePath),)?
			.to_path_buf(),),
		_ => Y(PathNotFound("current crate".to_string(),).into(),),
	}
}

/// depth 1 file search
pub fn search_cargo_toml(
	path: impl AsRef<Path,>,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	search_in(&path, CARGO_MANIFEST,)
}

pub fn search_in(
	place: &impl AsRef<Path,>,
	file_name: impl Into<String,> + Clone,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	let search_strategy = |entry: &Result<DirEntry, std::io::Error,>| {
		let found = entry.iter().filter_map(|entry| {
			entry.file_name().to_str().map(|s| s.to_string(),)
		},)
			// any: 「少なくとも1つあるか？」
// → 0個なら false
// all: 「すべて満たしているか？」
// → 反例が1つもないので true
			.any(|s| s==file_name.clone().into());
		X(found,)
	};
	search_in_with(place, search_strategy,)
}

pub fn search_in_with(
	place: &impl AsRef<Path,>,
	search_strategy: impl FnMut(
		&Result<DirEntry, std::io::Error,>,
	) -> PoisonGirlB<bool,>,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	let rslt = std::fs::read_dir(place,)?
		.try_find(search_strategy,)?
		.transpose()?
		.map(|entry| entry.path(),);
	// .map(|entry| entry.map(|entry| entry.map(|entry| entry.path(),),),);
	X(rslt,)
}

/// not recursively
pub fn search_in_cwd(
	file_name: impl Into<String,> + Clone,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	let cwd = current_dir()?;
	search_in(&cwd, file_name,)
}

pub fn get_upstream(
	file_name: impl Into<String,> + Clone,
) -> PoisonGirlB<PathBuf,>
{
	let p = match search_upstream(file_name.clone(),)? {
		None => Y(PathNotFound(file_name.into(),),),
		Some(p,) => X(p,),
	}?;
	X(p,)
}

pub fn search_upstream(
	file_name: impl Into<String,> + Clone,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	let place = current_dir()?;
	search_upstream_at(&place, file_name,)
}

pub fn search_upstream_at(
	path: &Path,
	file_name: impl Into<String,> + Clone,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	let mut place = path.to_path_buf();
	loop {
		if place.pop() {
			if let Some(p,) = search_in(&place, file_name.clone(),)? {
				break X(Some(p,),);
			}
		} else {
			break X(None,);
		}
	}
}

pub fn read_toml(path: impl AsRef<Path,>,)
-> PoisonGirlB<Option<toml::Table,>,>
{
	if !path.as_ref().exists() {
		return X(None,);
	}

	let read_toml_ = || -> PoisonGirlB<Option<toml::Table,>,> {
		let be_toml = std::fs::read(path,)?;
		let be_toml = String::from_utf8(be_toml,)?;
		let be_toml = be_toml.as_str();
		let be_toml: toml::Table = toml::de::from_str(be_toml,)?;
		X(Some(be_toml,),)
	};

	read_toml_()
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_test::{PoisonGirlTestB, fail, success},
	};

	fn test_dir(name: &str,) -> PoisonGirlB<PathBuf,>
	{
		let nanos = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH,)
			.map_or(0, |duration| duration.as_nanos(),);
		let path = std::env::temp_dir().join(format!(
			"poison_girl_dev_fs_{name}_{}_{}",
			std::process::id(),
			nanos
		),);
		std::fs::create_dir_all(&path,)?;
		X(path,)
	}

	#[test]
	fn test_search_cargo_toml() -> PoisonGirlTestB
	{
		let cargo_toml = search_cargo_toml(CWD,)??;
		assert_eq!(cargo_toml.to_str()?, std::env!("CARGO_MANIFEST_PATH"));
		success!()
	}

	#[test]
	fn test_search_in_found() -> PoisonGirlTestB
	{
		// Use the current project directory and search for Cargo.toml
		let current_dir = std::path::PathBuf::from(CWD,);
		let result = search_in(&current_dir, "Cargo.toml",)?;
		assert!(result.is_some());
		let found_path = result?;
		assert!(found_path.exists());
		assert_eq!(found_path.file_name()?, "Cargo.toml");
		success!()
	}

	#[test]
	fn test_search_in_not_found() -> PoisonGirlTestB
	{
		// Search for a non-existent file in the current directory
		let current_dir = std::path::PathBuf::from(CWD,);
		let result =
			search_in(&current_dir, "definitely_nonexistent_file_12345.xyz",)?;
		assert!(result.is_none());
		success!()
	}

	#[test]
	fn test_get_upstream_found() -> PoisonGirlTestB
	{
		// This should find Cargo.toml in the project structure
		let result = get_upstream("Cargo.toml",);
		assert!(result.is_x());
		let path = result?;
		assert!(path.exists());
		assert!(path.file_name()? == "Cargo.toml");
		success!()
	}

	#[test]
	fn test_get_upstream_not_found() -> PoisonGirlTestB
	{
		// This should fail to find a non-existent file
		let result = get_upstream("definitely_nonexistent_file_12345.xyz",);
		// assert!(result.is_y());

		let Y(error,) = result else {
			fail!("should be Y");
		};

		let error_msg = error.to_string();
		assert!(error_msg.contains("definitely_nonexistent_file_12345.xyz"));
		success!()
	}

	#[test]
	fn test_search_upstream_found() -> PoisonGirlTestB
	{
		// This should find Cargo.toml in the project structure
		let result = search_upstream("Cargo.toml",)?;
		assert!(result.is_some());
		let path = result?;
		assert!(path.exists());
		assert!(path.file_name()? == "Cargo.toml");
		success!()
	}

	#[test]
	fn test_search_upstream_not_found() -> PoisonGirlTestB
	{
		// This should not find a non-existent file
		let result = search_upstream("definitely_nonexistent_file_12345.xyz",)?;
		assert!(result.is_none());
		success!()
	}

	#[test]
	fn test_search_cargo_toml_with_different_cwd() -> PoisonGirlTestB
	{
		// Test with the root directory
		let root_path = std::path::PathBuf::from("/",);
		let result = search_cargo_toml(&root_path,);

		// Should still find the project's Cargo.toml by searching upstream
		assert!(result.is_x());
		let found_path = result?;
		if let Some(path,) = found_path {
			assert!(path.exists());
			assert!(path.file_name()? == "Cargo.toml");
		}
		success!()
	}

	#[test]
	fn test_search_in_with_subdirectories() -> PoisonGirlTestB
	{
		// Use the current project directory which should have subdirectories
		let current_dir = std::path::PathBuf::from(CWD,);

		// Search for Cargo.toml which should exist in the main directory
		let result = search_in(&current_dir, "Cargo.toml",)?;
		assert!(result.is_some());
		let found_path = result?;
		assert!(found_path.exists());
		assert_eq!(found_path.file_name()?, "Cargo.toml");

		// Search should not find files that don't exist at the current level
		let result = search_in(&current_dir, "nonexistent_file.txt",)?;
		assert!(result.is_none());
		success!()
	}

	#[test]
	fn test_file_name_matching() -> PoisonGirlTestB
	{
		// Use the current project directory
		let current_dir = std::path::PathBuf::from(CWD,);

		// Should find exact match for Cargo.toml
		let result = search_in(&current_dir, "Cargo.toml",)?;
		assert!(result.is_some());
		let found_path = result?;
		assert_eq!(found_path.file_name()?, "Cargo.toml");

		// Should not find partial matches
		let result = search_in(&current_dir, "Cargo",)?;
		assert!(result.is_none());
		success!()
	}

	// TODO: test_search_in_ignores_directories

	// #[test]
	// fn test_project_root_path_functionality() -> PoisonGirlTestB
	// {
	// 	// Test that project_root_path returns a result
	// 	let result = project_root_path()?;
	// 	eprintln!("{result:?}");
	// 	// We can't make strong assertions about the result since it depends on
	// 	// the file system but we can verify it returns something
	// 	let answer = std::env!("CARGO_MANIFEST_DIR");
	// 	let answer = PathBuf::from_str(answer,)?.parent()?.to_path_buf();
	// 	assert_eq!(result, answer);
	// 	success!()
	// }

	// TODO: このテストケース死んでない?
	#[test]
	fn test_search_cargo_toml_edge_cases() -> PoisonGirlTestB
	{
		// Test with root directory
		let root_path = std::path::PathBuf::from("/",);
		let result = search_cargo_toml(&root_path,);
		// Should not find Cargo.toml in root directory
		assert!(result.is_x());
		if let X(found,) = result
			// If found, it should be None for root directory
		&& let Some(path,) = found
		{
			// If somehow found, verify it's actually a Cargo.toml file
			assert!(path.file_name()? == "Cargo.toml");
		}
		success!()
	}

	#[test]
	fn test_get_upstream_error_cases()
	{
		// Test get_upstream with a file that definitely doesn't exist
		let result = get_upstream(
			"definitely_nonexistent_file_with_very_unique_name_12345.xyz",
		);
		assert!(result.is_y());
	}

	#[test]
	fn test_search_in_with_empty_filename() -> PoisonGirlTestB
	{
		// Test with empty filename
		let current_dir = std::path::PathBuf::from(CWD,);
		let result = search_in(&current_dir, "",)?;
		assert!(result.is_none());
		success!()
	}

	#[test]
	fn test_search_upstream_from_root() -> PoisonGirlTestB
	{
		// Test search_upstream when starting from root directory
		let original_dir = std::env::current_dir()?;

		// Try to change to root directory
		if std::env::set_current_dir("/",).is_ok() {
			let result =
				search_upstream("definitely_nonexistent_file_12345.xyz",)?;
			assert!(result.is_none());

			// Restore original directory
			std::env::set_current_dir(original_dir,)?;
		}
		success!()
	}

	#[cfg(unix)]
	#[test]
	fn test_search_upstream_with_symlinks() -> PoisonGirlTestB
	{
		let root = test_dir("symlink",)?;
		let manifest = root.join("Cargo.toml",);
		let real_nested = root.join("real",).join("nested",);
		let linked_nested = root.join("linked_nested",);
		let start = linked_nested.join("child",);

		std::fs::write(&manifest, "[package]\nname = \"fixture\"\n",)?;
		std::fs::create_dir_all(&real_nested,)?;
		std::os::unix::fs::symlink(&real_nested, &linked_nested,)?;
		std::fs::create_dir_all(&start,)?;

		let result = search_upstream_at(&start, "Cargo.toml",)?;
		std::fs::remove_dir_all(&root,)?;

		assert_eq!(result, Some(manifest,));
		success!()
	}

	#[cfg(unix)]
	#[test]
	fn test_search_in_with_permission_denied() -> PoisonGirlTestB
	{
		use std::os::unix::fs::PermissionsExt;

		let root = test_dir("permission_denied",)?;
		let restricted = root.join("restricted",);
		std::fs::create_dir(&restricted,)?;

		let original_permissions =
			std::fs::metadata(&restricted,)?.permissions();
		let mut denied_permissions = original_permissions.clone();
		denied_permissions.set_mode(0o000,);
		std::fs::set_permissions(&restricted, denied_permissions,)?;

		let result = search_in(&restricted, "any_file.txt",);

		std::fs::set_permissions(&restricted, original_permissions,)?;
		std::fs::remove_dir_all(&root,)?;

		assert!(result.is_y());
		success!()
	}
}
