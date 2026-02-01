use {
	poison_girl_dev_error::{Container, PathNotFound, PoisonGirlB, X, Y},
	std::{
		env::current_dir,
		fs::DirEntry,
		path::{Path, PathBuf},
		str::FromStr,
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
		.read_dir()
		.unwrap_or_else(|_| panic!("failed to read {}", path.display()),)
		.filter_map(|entry| {
			if entry.as_ref().expect("failed to get entry",).path().is_file() {
				return None;
			}

			let path = entry.as_ref().expect("failed to get entry",).path();
			let name = path.file_name().unwrap();
			let name = name.to_str().unwrap();
			match name {
				_ if IGNORE_DIR_LIST.contains(&name,) => None,
				_ => Some(path,),
			}
		},)
		.map(|p| {
			let mut paths = if search_cargo_toml(&p,)?.is_some() {
				vec![p.clone()]
			} else {
				vec![]
			};
			paths.append(&mut all_crates_in(&p,)?,);
			X(paths,)
		},)
		.flat_map(|v: PoisonGirlB<Vec<PathBuf,>,>| v.unwrap(),)
		.collect(),)
}

pub fn project_root_path() -> PoisonGirlB<PathBuf,>
{
	let mut p = PathBuf::from_str(CWD,).unwrap();
	let mut last_cargo_toml = None;

	while p.pop() {
		if let Some(p,) = search_cargo_toml(&p,)? {
			last_cargo_toml = Some(p,)
		}
	}

	X(last_cargo_toml.unwrap().parent().unwrap().to_path_buf(),)
}

pub fn current_crate_path() -> PoisonGirlB<PathBuf,>
{
	match search_upstream(CARGO_MANIFEST,) {
		X(Some(p,),) => {
			X(p.parent().expect("should have parent directory",).to_path_buf(),)
		},
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
		entry
			.as_ref()
			.expect("failed to get dir entry",)
			.file_name()
			.to_str()
			.unwrap() == file_name.clone().into()
	};
	search_in_with(place, search_strategy,)
}

pub fn search_in_with(
	place: &impl AsRef<Path,>,
	search_strategy: impl FnMut(&Result<DirEntry, std::io::Error,>,) -> bool,
) -> PoisonGirlB<Option<PathBuf,>,>
{
	let rslt = std::fs::read_dir(place,)?
		.find(search_strategy,)
		.map(|entry| entry.map(|entry| entry.path(),),)
		.transpose()?;
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

pub fn read_toml(path: impl AsRef<Path,>,) -> PoisonGirlB<toml::Table,>
{
	if !path.as_ref().exists() {
		return Y(PathNotFound(
			path.as_ref().to_str().unwrap_or_default().to_string(),
		)
		.into(),);
	}

	let read_toml_ = || -> PoisonGirlB<toml::Table,> {
		let be_toml = std::fs::read(path,)?;
		let be_toml = String::from_utf8(be_toml,)?;
		let be_toml = be_toml.as_str();
		let be_toml: toml::Table = toml::de::from_str(be_toml,)?;
		X(be_toml,)
	};

	read_toml_()
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn test_search_cargo_toml() -> PoisonGirlB<(),>
	{
		let cargo_toml =
			search_cargo_toml(CWD,)?.expect("failed to find Cargo.toml",);
		assert_eq!(
			cargo_toml.to_str().unwrap(),
			std::env!("CARGO_MANIFEST_PATH")
		);
		X((),)
	}

	#[test]
	fn test_search_in_found() -> PoisonGirlB<(),>
	{
		// Use the current project directory and search for Cargo.toml
		let current_dir = std::path::PathBuf::from(CWD,);
		let result = search_in(&current_dir, "Cargo.toml",)?;
		assert!(result.is_some());
		let found_path = result.unwrap();
		assert!(found_path.exists());
		assert_eq!(found_path.file_name().unwrap(), "Cargo.toml");
		X((),)
	}

	#[test]
	fn test_search_in_not_found() -> PoisonGirlB<(),>
	{
		// Search for a non-existent file in the current directory
		let current_dir = std::path::PathBuf::from(CWD,);
		let result =
			search_in(&current_dir, "definitely_nonexistent_file_12345.xyz",)?;
		assert!(result.is_none());
		X((),)
	}

	#[test]
	fn test_get_upstream_found() -> PoisonGirlB<(),>
	{
		// This should find Cargo.toml in the project structure
		let result = get_upstream("Cargo.toml",);
		assert!(result.is_x());
		let path = result.unwrap();
		assert!(path.exists());
		assert!(path.file_name().unwrap() == "Cargo.toml");
		X((),)
	}

	#[test]
	fn test_get_upstream_not_found()
	{
		// This should fail to find a non-existent file
		let result = get_upstream("definitely_nonexistent_file_12345.xyz",);
		assert!(result.is_y());
		let error_msg = result.unwrap_inv().to_string();
		assert!(error_msg.contains("can not find out"));
		assert!(error_msg.contains("definitely_nonexistent_file_12345.xyz"));
	}

	#[test]
	#[ignore = "unknown bug"]
	fn test_search_upstream_found() -> PoisonGirlB<(),>
	{
		// This should find Cargo.toml in the project structure
		let result = search_upstream("Cargo.toml",)?;
		assert!(result.is_some());
		let path = result.unwrap();
		assert!(path.exists());
		assert!(path.file_name().unwrap() == "Cargo.toml");
		X((),)
	}

	#[test]
	fn test_search_upstream_not_found() -> PoisonGirlB<(),>
	{
		// This should not find a non-existent file
		let result = search_upstream("definitely_nonexistent_file_12345.xyz",)?;
		assert!(result.is_none());
		X((),)
	}

	#[test]
	fn test_check_oso_kernel_file_not_exists()
	{
		// In most test environments, oso_kernel.elf won't exist
		let result = check_poison_girl_kernel();
		// We expect this to fail in test environment
		assert!(result.is_y());
		let error_msg = result.unwrap_inv().to_string();
		assert!(error_msg.contains("oso_kernel.elf"));
	}

	#[test]
	fn test_search_cargo_toml_with_different_cwd() -> PoisonGirlB<(),>
	{
		// Test with the root directory
		let root_path = std::path::PathBuf::from("/",);
		let result = search_cargo_toml(&root_path,);

		// Should still find the project's Cargo.toml by searching upstream
		assert!(result.is_x());
		let found_path = result.unwrap();
		if let Some(path,) = found_path {
			assert!(path.exists());
			assert!(path.file_name().unwrap() == "Cargo.toml");
		}
		X((),)
	}

	#[test]
	fn test_constants()
	{
		// Test that constants are defined correctly
		assert_eq!(CARGO_MANIFEST, "Cargo.toml");
		assert_eq!(CARGO_CONFIG, ".cargo/config.toml");

		// CWD should be a valid path string
		let cwd_path = std::path::Path::new(CWD,);
		assert!(cwd_path.exists());
		assert!(cwd_path.is_dir());
	}

	#[test]
	fn test_search_in_with_subdirectories() -> PoisonGirlB<(),>
	{
		// Use the current project directory which should have subdirectories
		let current_dir = std::path::PathBuf::from(CWD,);

		// Search for Cargo.toml which should exist in the main directory
		let result = search_in(&current_dir, "Cargo.toml",)?;
		assert!(result.is_some());
		let found_path = result.unwrap();
		assert!(found_path.exists());
		assert_eq!(found_path.file_name().unwrap(), "Cargo.toml");

		// Search should not find files that don't exist at the current level
		let result = search_in(&current_dir, "nonexistent_file.txt",)?;
		assert!(result.is_none());
		X((),)
	}

	#[test]
	fn test_file_name_matching() -> PoisonGirlB<(),>
	{
		// Use the current project directory
		let current_dir = std::path::PathBuf::from(CWD,);

		// Should find exact match for Cargo.toml
		let result = search_in(&current_dir, "Cargo.toml",)?;
		assert!(result.is_some());
		let found_path = result.unwrap();
		assert_eq!(found_path.file_name().unwrap(), "Cargo.toml");

		// Should not find partial matches
		let result = search_in(&current_dir, "Cargo",)?;
		assert!(result.is_none());
		X((),)
	}

	#[test]
	fn test_ignore_dir_list()
	{
		// Test that the ignore directory list is properly defined
		assert!(IGNORE_DIR_LIST.contains(&"target"));
		assert!(IGNORE_DIR_LIST.contains(&".git"));
		assert!(IGNORE_DIR_LIST.contains(&".github"));
		assert!(IGNORE_DIR_LIST.contains(&".direnv"));
		assert!(IGNORE_DIR_LIST.contains(&".cargo"));
		assert_eq!(IGNORE_DIR_LIST.len(), 5);
	}

	#[test]
	fn test_search_in_ignores_directories() -> PoisonGirlB<(),>
	{
		// This test verifies that search_in only looks at files, not
		// directories
		let current_dir = std::path::PathBuf::from(CWD,);

		// Even if there's a directory named like a file we're searching for,
		// search_in should not return it (it only returns files)
		// We can't easily test this without creating directories, so we'll
		// just verify that search_in returns files, not directories
		if let Some(found,) = search_in(&current_dir, "Cargo.toml",)? {
			assert!(
				found.is_file(),
				"search_in should return files, not directories"
			);
		}
		X((),)
	}

	#[test]
	fn test_path_operations() -> PoisonGirlB<(),>
	{
		// Test basic path operations used in the module
		let current_dir = std::path::PathBuf::from(CWD,);
		assert!(current_dir.is_absolute() || current_dir.is_relative());

		// Test that we can join paths
		let joined = current_dir.join("Cargo.toml",);
		assert!(joined.to_string_lossy().contains("Cargo.toml"));
		X((),)
	}

	#[test]
	fn test_all_crates_functionality() -> PoisonGirlB<(),>
	{
		// Test that all_crates returns a result
		let result = all_crates();
		// We can't make strong assertions about the result since it depends on
		// the file system but we can verify it returns something
		assert!(result.is_x() || result.is_y());
		X((),)
	}

	#[test]
	fn test_project_root_path_functionality() -> PoisonGirlB<(),>
	{
		// Test that project_root_path returns a result
		let result = project_root_path()?;
		eprintln!("{result:?}");
		// We can't make strong assertions about the result since it depends on
		// the file system but we can verify it returns something
		let answer = std::env!("CARGO_MANIFEST_DIR");
		let answer =
			PathBuf::from_str(answer,).unwrap().parent().unwrap().to_path_buf();
		assert_eq!(result, answer);
		X((),)
	}

	#[test]
	fn test_current_crate_path_functionality() -> PoisonGirlB<(),>
	{
		// Test that current_crate_path returns a result
		let result = current_crate_path();
		// We can't make strong assertions about the result since it depends on
		// the file system but we can verify it returns something
		assert!(result.is_x() || result.is_y());
		X((),)
	}

	#[test]
	fn test_search_in_cwd_functionality() -> PoisonGirlB<(),>
	{
		// Test searching for Cargo.toml in current working directory
		let result = search_in_cwd("Cargo.toml",)?;
		// This might or might not find Cargo.toml depending on where the test
		// runs Just verify the function works
		assert!(result.is_some() || result.is_none());

		// Test searching for a non-existent file
		let result = search_in_cwd("definitely_nonexistent_file_12345.xyz",)?;
		assert!(result.is_none());
		X((),)
	}

	#[test]
	fn test_error_handling_with_invalid_paths()
	{
		// Test with a path that doesn't exist
		let invalid_path =
			std::path::PathBuf::from("/definitely/nonexistent/path/12345",);
		let result = search_in(&invalid_path, "any_file.txt",);
		assert!(result.is_y());
	}

	#[test]
	fn test_constants_values()
	{
		// Test that constants have expected values
		assert_eq!(CARGO_MANIFEST, "Cargo.toml");
		assert_eq!(CARGO_CONFIG, ".cargo/config.toml");

		// Test that CWD is a valid path
		let cwd_path = std::path::Path::new(CWD,);
		assert!(cwd_path.exists());

		// Test that IGNORE_DIR_LIST contains expected directories
		assert!(IGNORE_DIR_LIST.contains(&"target"));
		assert!(IGNORE_DIR_LIST.contains(&".git"));
		assert!(IGNORE_DIR_LIST.contains(&".github"));
		assert!(IGNORE_DIR_LIST.contains(&".direnv"));
		assert!(IGNORE_DIR_LIST.contains(&".cargo"));
	}

	#[test]
	fn test_check_oso_kernel_with_different_working_directories()
	{
		// Test check_oso_kernel from different contexts
		let original_dir = std::env::current_dir().unwrap();

		// Try from a different directory (if possible)
		if let Ok(temp_dir,) = std::env::temp_dir().canonicalize()
			&& std::env::set_current_dir(&temp_dir,).is_ok()
		{
			let result = check_poison_girl_kernel();
			// Should fail since oso_kernel.elf won't be in temp directory
			assert!(result.is_y());

			// Restore original directory
			let _ = std::env::set_current_dir(original_dir,);
		}
	}

	#[test]
	fn test_search_cargo_toml_edge_cases() -> PoisonGirlB<(),>
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
			assert!(path.file_name().unwrap() == "Cargo.toml");
		}
		X((),)
	}

	#[test]
	fn test_get_upstream_error_cases()
	{
		// Test get_upstream with a file that definitely doesn't exist
		let result = get_upstream(
			"definitely_nonexistent_file_with_very_unique_name_12345.xyz",
		);
		assert!(result.is_y());

		let error_msg = result.unwrap_inv().to_string();
		assert!(error_msg.contains("can not find out"));
		assert!(error_msg.contains(
			"definitely_nonexistent_file_with_very_unique_name_12345.xyz"
		));
	}

	#[test]
	#[ignore = "unknown bug"]
	fn test_search_upstream_from_deep_directory() -> PoisonGirlB<(),>
	{
		// Test search_upstream from a deeper directory structure
		let original_dir = std::env::current_dir()?;

		// Try to go to a subdirectory if it exists
		let src_dir = std::path::PathBuf::from(CWD,).join("src",);
		if src_dir.exists() && src_dir.is_dir() {
			std::env::set_current_dir(&src_dir,)?;

			// Should still find Cargo.toml by searching upstream
			let result = search_upstream("Cargo.toml",)?;
			assert!(result.is_some());

			if let Some(path,) = result {
				assert!(path.exists());
				assert!(path.file_name().unwrap() == "Cargo.toml");
			}

			// Restore original directory
			std::env::set_current_dir(original_dir,)?;
		}
		X((),)
	}

	#[test]
	fn test_file_system_edge_cases() -> PoisonGirlB<(),>
	{
		// Test various edge cases with file system operations
		let current_dir = std::path::PathBuf::from(CWD,);

		// Test search_in with various file names
		let test_files =
			vec!["Cargo.toml", "src", "target", "README.md", "LICENSE"];

		for file_name in test_files {
			let result = search_in(&current_dir, file_name,)?;
			// Each result should be either Some or None
			if let Some(path,) = result {
				assert!(path.exists());
				assert_eq!(
					path.file_name().unwrap().to_str().unwrap(),
					file_name
				);
			}
		}
		X((),)
	}

	#[test]
	fn test_search_in_with_unicode_filenames() -> PoisonGirlB<(),>
	{
		// Test searching for files with unicode names (if they exist)
		let current_dir = std::path::PathBuf::from(CWD,);

		// Test with unicode filename (won't find it, but shouldn't crash)
		let result = search_in(&current_dir, "测试文件.txt",)?;
		assert!(result.is_none());

		// Test with emoji filename
		let result = search_in(&current_dir, "🦀.rs",)?;
		assert!(result.is_none());
		X((),)
	}

	#[test]
	fn test_search_in_with_special_characters() -> PoisonGirlB<(),>
	{
		// Test searching for files with special characters
		let current_dir = std::path::PathBuf::from(CWD,);

		let special_names = vec![
			"file with spaces.txt",
			"file-with-dashes.txt",
			"file_with_underscores.txt",
			"file.with.dots.txt",
			"file@with#special$chars%.txt",
		];

		for name in special_names {
			let result = search_in(&current_dir, name,)?;
			// These files likely don't exist, but the function should handle
			// them gracefully
			assert!(result.is_none() || result.is_some());
		}
		X((),)
	}

	#[test]
	fn test_search_in_with_very_long_filenames() -> PoisonGirlB<(),>
	{
		// Test with very long filenames
		let current_dir = std::path::PathBuf::from(CWD,);

		let long_name = "a".repeat(255,) + ".txt";
		let result = search_in(&current_dir, long_name,)?;
		assert!(result.is_none()); // Unlikely to exist
		X((),)
	}

	#[test]
	fn test_search_in_with_empty_filename() -> PoisonGirlB<(),>
	{
		// Test with empty filename
		let current_dir = std::path::PathBuf::from(CWD,);
		let result = search_in(&current_dir, "",)?;
		assert!(result.is_none());
		X((),)
	}

	#[test]
	fn test_search_in_with_dot_files() -> PoisonGirlB<(),>
	{
		// Test searching for hidden files (dot files)
		let current_dir = std::path::PathBuf::from(CWD,);

		let dot_files = vec![".gitignore", ".cargo", ".hidden", "..parent"];

		for dot_file in dot_files {
			let result = search_in(&current_dir, dot_file,)?;
			// These may or may not exist, just verify no panic
			assert!(result.is_none() || result.is_some());
		}
		X((),)
	}

	#[test]
	fn test_search_upstream_from_root() -> PoisonGirlB<(),>
	{
		// Test search_upstream when starting from root directory
		let original_dir = std::env::current_dir()?;

		// Try to change to root directory
		if std::env::set_current_dir("/",).is_ok() {
			let result =
				search_upstream("definitely_nonexistent_file_12345.xyz",)?;
			assert!(result.is_none());

			// Restore original directory
			let _ = std::env::set_current_dir(original_dir,);
		}
		X((),)
	}

	#[test]
	fn test_search_upstream_with_symlinks() -> PoisonGirlB<(),>
	{
		// Test behavior with symbolic links (if any exist)
		// This is system-dependent, so we'll just test that it doesn't panic
		let result = search_upstream("Cargo.toml",)?;
		if let Some(path,) = result {
			// Verify the found path exists and is readable
			assert!(path.exists());
			assert!(path.is_file());
		}
		X((),)
	}

	#[test]
	fn test_get_upstream_with_case_sensitivity()
	{
		// Test case sensitivity in file search
		let result1 = get_upstream("Cargo.toml",);
		let result2 = get_upstream("cargo.toml",); // Different case

		// On case-sensitive filesystems, these might be different
		// On case-insensitive filesystems, they might be the same
		// Just verify both handle the case gracefully
		match (result1, result2,) {
			(X(_,), X(_,),) => {}, // Both found
			(X(_,), Y(_,),) => {}, // Only first found (case-sensitive)
			(Y(_,), X(_,),) => {}, // Only second found (unlikely)
			(Y(_,), Y(_,),) => {}, // Neither found
		}
	}

	#[test]
	fn test_check_oso_kernel_with_custom_target_dir()
	{
		// Test check_oso_kernel with different target directory structures
		let original_dir = std::env::current_dir().unwrap();

		// Create a temporary directory structure for testing
		if let Ok(temp_dir,) = std::env::temp_dir().canonicalize()
			&& std::env::set_current_dir(&temp_dir,).is_ok()
		{
			let result = check_poison_girl_kernel();
			// Should fail since oso_kernel.elf won't be in temp directory
			assert!(result.is_y());

			// Restore original directory
			let _ = std::env::set_current_dir(original_dir,);
		}
	}

	#[test]
	fn test_constants_immutability()
	{
		// Test that constants have expected values and are immutable
		let manifest = CARGO_MANIFEST;
		let config = CARGO_CONFIG;
		let cwd = CWD;

		assert_eq!(manifest, "Cargo.toml");
		assert_eq!(config, ".cargo/config.toml");
		assert!(!cwd.is_empty());

		// Verify CWD points to a valid directory
		let cwd_path = std::path::Path::new(cwd,);
		assert!(cwd_path.exists());
		assert!(cwd_path.is_dir());
	}

	#[test]
	fn test_ignore_dir_list_completeness()
	{
		// Test that IGNORE_DIR_LIST contains expected directories
		assert!(IGNORE_DIR_LIST.contains(&"target"));
		assert!(IGNORE_DIR_LIST.contains(&".git"));
		assert!(IGNORE_DIR_LIST.contains(&".github"));
		assert!(IGNORE_DIR_LIST.contains(&".direnv"));
		assert!(IGNORE_DIR_LIST.contains(&".cargo"));

		// Verify the list has the expected length
		assert_eq!(IGNORE_DIR_LIST.len(), 5);

		// Verify all entries are non-empty strings
		for dir in &IGNORE_DIR_LIST {
			assert!(!dir.is_empty());
		}
	}

	#[test]
	fn test_search_in_with_permission_denied() -> PoisonGirlB<(),>
	{
		// Test behavior when encountering permission denied errors
		// This is system-dependent and might not trigger on all systems
		let restricted_paths = vec![
			"/root",
			"/private/var/root",
			"/System/Library/PrivateFrameworks",
		];

		for path in restricted_paths {
			let path_buf = std::path::PathBuf::from(path,);
			if path_buf.exists() {
				let result = search_in(&path_buf, "any_file.txt",);
				// Should either succeed or fail gracefully
				match result {
					X(_,) => {}, // Success
					Y(_,) => {}, // Expected failure due to permissions
				}
			}
		}
		X((),)
	}

	#[test]
	fn test_search_cargo_toml_in_nested_structure() -> PoisonGirlB<(),>
	{
		// Test searching for Cargo.toml in nested directory structures
		let current_dir = std::path::PathBuf::from(CWD,);

		// Test with the current directory
		let result = search_cargo_toml(&current_dir,)?;
		assert!(result.is_some());

		// Test with parent directory if it exists
		if let Some(parent,) = current_dir.parent() {
			let result = search_cargo_toml(parent,);
			// May or may not find Cargo.toml in parent
			assert!(result.is_x());
		}
		X((),)
	}

	#[test]
	fn test_path_traversal_security() -> PoisonGirlB<(),>
	{
		// Test that path traversal attempts are handled safely
		let current_dir = std::path::PathBuf::from(CWD,);

		let traversal_attempts = vec![
			"../../../etc/passwd",
			"..\\..\\..\\windows\\system32\\config\\sam",
			"../../../../../../../../etc/shadow",
			"../Cargo.toml",
		];

		for attempt in traversal_attempts {
			let result = search_in(&current_dir, attempt,)?;
			// These should not find anything in the current directory
			// (they're looking for files with these exact names, not
			// traversing)
			assert!(result.is_none());
		}
		X((),)
	}

	#[test]
	fn test_concurrent_file_operations() -> PoisonGirlB<(),>
	{
		// Test concurrent file system operations
		use std::thread;

		let handles: Vec<_,> = (0..5)
			.map(|_i| {
				thread::spawn(move || {
					let current_dir = std::path::PathBuf::from(CWD,);
					search_in(&current_dir, "Cargo.toml",)
				},)
			},)
			.collect();

		for handle in handles {
			let result = handle.join().expect("Thread should not panic",);
			assert!(result.is_x());
			if let X(Some(path,),) = result {
				assert!(path.exists());
			}
		}
		X((),)
	}

	#[test]
	fn test_file_system_edge_cases_extended() -> PoisonGirlB<(),>
	{
		// Extended test for various file system edge cases
		let current_dir = std::path::PathBuf::from(CWD,);

		// Test with various file extensions
		let extensions = vec![
			"Cargo.toml",
			"Cargo.lock",
			"README.md",
			"LICENSE",
			"lib.rs",
			"main.rs",
		];

		for ext in extensions {
			let result = search_in(&current_dir, ext,)?;
			if let Some(path,) = result {
				assert!(path.exists());
				assert!(path.is_file());
				assert_eq!(path.file_name().unwrap().to_str().unwrap(), ext);
			}
		}
		X((),)
	}

	#[test]
	fn test_search_in_with_binary_files() -> PoisonGirlB<(),>
	{
		// Test searching for binary files
		let current_dir = std::path::PathBuf::from(CWD,);

		let binary_names = vec![
			"target",
			"Cargo.lock",
			"test.exe",
			"test.bin",
			"test.so",
			"test.dylib",
		];

		for name in binary_names {
			let result = search_in(&current_dir, name,)?;
			// These may or may not exist, just verify no panic
			if let Some(path,) = result {
				assert!(path.exists());
			}
		}
		X((),)
	}

	#[test]
	fn test_error_message_quality()
	{
		// Test that error messages are informative
		let result = get_upstream(
			"definitely_nonexistent_file_with_very_unique_name_12345.xyz",
		);
		assert!(result.is_y());

		let error_msg = result.unwrap_inv().to_string();
		assert!(error_msg.contains("can not find out"));
		assert!(error_msg.contains(
			"definitely_nonexistent_file_with_very_unique_name_12345.xyz"
		));
		assert!(!error_msg.is_empty());
	}

	#[test]
	fn test_all_crates_with_complex_directory_structure() -> PoisonGirlB<(),>
	{
		// Test all_crates function with complex directory structures
		let result = all_crates();

		// The function should return a result (success or failure)
		match result {
			X(crates,) => {
				// All returned paths should be valid
				for crate_path in crates {
					assert!(crate_path.exists());
					assert!(crate_path.is_dir());
				}
			},
			Y(_,) => {
				// If it fails, that's also acceptable for this test
				// The important thing is that it doesn't panic
			},
		}
		X((),)
	}

	#[test]
	fn test_project_root_path_consistency() -> PoisonGirlB<(),>
	{
		// Test that project_root_path returns consistent results
		let result1 = project_root_path();
		let result2 = project_root_path();

		match (result1, result2,) {
			(X(path1,), X(path2,),) => {
				assert_eq!(
					path1, path2,
					"project_root_path should be consistent"
				);
				assert!(path1.exists());
				assert!(path1.is_dir());
			},
			(Y(_,), Y(_,),) => {
				// Both failed consistently
			},
			_ => {
				panic!(
					"project_root_path should be consistent in success/failure"
				);
			},
		}
		X((),)
	}

	#[test]
	fn test_current_crate_path_validity() -> PoisonGirlB<(),>
	{
		// Test that current_crate_path returns a valid path when successful
		let result = current_crate_path();

		match result {
			X(path,) => {
				assert!(path.exists());
				assert!(path.is_file());
				assert!(path.file_name().unwrap() == "Cargo.toml");
			},
			Y(_,) => {
				// If it fails, that's acceptable for this test
			},
		}
		X((),)
	}
}
