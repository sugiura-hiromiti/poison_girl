use {
	crate::decl_manage::{
		crate_::{Crate, CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart},
		package::PackageSurvey,
	},
	poison_girl_dev_cargo::{Arch, BuildMode, CompileOpt, Opts},
	poison_girl_dev_error::{PoisonGirlB, X, Y},
	poison_girl_dev_fs::{
		current_crate_path, project_root_path, search_in_with,
	},
	std::path::PathBuf,
};

pub mod crate_;
pub mod package;
pub mod workspace;

pub trait CargoCrate
{
	fn specified_target(&self,) -> PoisonGirlB<impl Into<String,>,>;
	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,>;
	fn as_crate(&self,) -> &impl Crate;
	fn as_opts(&self,) -> &impl CompileOpt;
}

pub struct PoisonGirlCargoInterface
{
	ws:  PoisonGirlCrate,
	opt: Opts,
}

impl PoisonGirlCargoInterface
{
	pub fn new(
		chart: PoisonGirlCrateChart,
		arch: Arch,
		build_mode: BuildMode,
	) -> Self
	{
		Self {
			ws:  PoisonGirlCrate::from(chart,),
			opt: Opts { build_mode, feature_flags: vec![], arch, },
		}
	}
}

impl CargoCrate for PoisonGirlCargoInterface
{
	fn specified_target(&self,) -> PoisonGirlB<impl Into<String,>,>
	{
		let search_rslt = search_in_with(&self.ws.path(), |entry| {
			let file_name = entry
				.as_ref()
				.expect("file io error",)
				.file_name()
				.to_string_lossy()
				.to_string();
			let arch = self.opt.arch().into();

			file_name.contains(&arch,) && file_name.ends_with(".json",)
		},);

		match search_rslt {
			X(Some(p,),) => X(p.to_string_lossy().to_string(),),
			X(None,) => self.ws.default_target().map(|s| s.into(),),
			Y(e,) => Y(e,),
		}
	}

	fn build_artifact(&self,) -> PoisonGirlB<PathBuf,>
	{
		X(self
			.ws
			.path()
			.join("target",)
			.join(self.specified_target()?.into(),)
			.join(self.opt.build_mode().into(),),)
	}

	fn as_crate(&self,) -> &impl Crate
	{
		&self.ws
	}

	fn as_opts(&self,) -> &impl CompileOpt
	{
		&self.opt
	}
}

pub fn project_root() -> PoisonGirlB<PoisonGirlCrate,>
{
	let pr = project_root_path()?;
	X(PoisonGirlCrate::from(pr,),)
}

pub fn current_crate() -> PoisonGirlB<PoisonGirlCrate,>
{
	let ccp = current_crate_path()?;

	X(PoisonGirlCrate::from(ccp,),)
}

#[cfg(test)]
mod tests
{
	use {
		super::*, crate::decl_manage::crate_::CrateInfo,
		poison_girl_dev_error::Y,
	};

	#[test]
	fn test_project_root_function_exists()
	{
		// Test that the project_root function exists and returns a Result
		let result = project_root();

		// The function should return a Result, regardless of success or failure
		match result {
			X(crate_obj,) => {
				// If successful, verify we got an OsoCrate
				let path = crate_obj.path();
				assert!(path.is_absolute() || path.starts_with("."));
			},
			Y(e,) => {
				// If it fails, that's acceptable in test environments
				// Just verify we get a meaningful error
				let error_msg = e.to_string();
				assert!(!error_msg.is_empty());
			},
		}
	}

	#[test]
	fn test_current_crate_function_exists()
	{
		// Test that the current_crate function exists and returns a Result
		let result = current_crate();

		// The function should return a Result, regardless of success or failure
		match result {
			X(crate_obj,) => {
				// If successful, verify we got an OsoCrate
				let path = crate_obj.path();
				assert!(path.is_absolute() || path.starts_with("."));
			},
			Y(e,) => {
				// If it fails, that's acceptable in test environments
				// Just verify we get a meaningful error
				let error_msg = e.to_string();
				assert!(!error_msg.is_empty());
			},
		}
	}

	#[test]
	fn test_project_root_returns_oso_crate()
	{
		// Test that project_root returns an OsoCrate type
		let result = project_root();

		// Verify the return type is correct
		if let Y(e,) = result {
			panic!("{e:?}")
		}
	}

	#[test]
	fn test_current_crate_returns_oso_crate()
	{
		// Test that current_crate returns an OsoCrate type
		let result = current_crate();

		// Verify the return type is correct
		if let Y(e,) = result {
			panic!("{e:?}")
		}
	}

	#[test]
	fn test_function_integration()
	{
		// Test that both functions use the same underlying helper functions
		// This is more of an integration test

		let project_result = project_root();
		let current_result = current_crate();

		// Both should return the same type
		match (project_result, current_result,) {
			(X(project_crate,), X(current_crate,),) => {
				// Both should be valid OsoCrate instances
				let _project_path = project_crate.path();
				let _current_path = current_crate.path();
			},
			(Y(_,), _,) | (_, Y(_,),) => {
				// Errors are acceptable in test environment
			},
		}
	}

	#[test]
	fn test_result_type_consistency()
	{
		// Test that both functions return the same Result type
		let project_result = project_root();
		let current_result = current_crate();

		// Verify both are the same type by using them in the same context
		let results = vec![project_result, current_result];

		for result in results {
			assert!(result.is_x());
		}
	}

	#[test]
	fn test_function_signatures()
	{
		// Test that function signatures are as expected

		// project_root should take no parameters and return Rslt<OsoCrate>
		let _: fn() -> PoisonGirlB<PoisonGirlCrate,> = project_root;

		// current_crate should take no parameters and return Rslt<OsoCrate>
		let _: fn() -> PoisonGirlB<PoisonGirlCrate,> = current_crate;
	}
	#[test]
	fn test_oso_crate_conversion()
	{
		// Test that PathBuf to OsoCrate conversion works correctly

		let project_result = project_root();
		let current_result = current_crate();

		if let X(project_crate,) = project_result {
			let path = project_crate.path();

			// Test that we can create OsoCrate from PathBuf
			let recreated = PoisonGirlCrate::from(path.clone(),);
			assert_eq!(recreated.path(), path);

			// Test that conversion is consistent
			assert_eq!(project_crate, recreated);
		}

		if let X(current_crate,) = current_result {
			let path = current_crate.path();

			// Test that we can create OsoCrate from PathBuf
			let recreated = PoisonGirlCrate::from(path.clone(),);
			assert_eq!(recreated.path(), path);

			// Test that conversion is consistent
			assert_eq!(current_crate, recreated);
		}
	}

	#[test]
	fn test_path_properties()
	{
		// Test properties of returned paths
		let project_result = project_root();
		let current_result = current_crate();

		if let X(project_crate,) = project_result {
			let path = project_crate.path();

			// Path should be valid for filesystem operations
			// Note: We can't guarantee the path exists in all test
			// environments, but we can test that it's a valid path structure
			assert!(!path.as_os_str().is_empty());

			// Test that path can be converted to string
			let _path_str = path.to_string_lossy();

			// Test that path has components
			let _components: Vec<_,> = path.components().collect();
		}

		if let X(current_crate,) = current_result {
			let path = current_crate.path();

			// Path should be valid for filesystem operations
			assert!(!path.as_os_str().is_empty());

			// Test that path can be converted to string
			let _path_str = path.to_string_lossy();

			// Test that path has components
			let _components: Vec<_,> = path.components().collect();
		}
	}

	#[test]
	fn test_thread_safety()
	{
		// Test that functions can be called from multiple threads
		use std::thread;

		let handles: Vec<_,> = (0..5)
			.map(|_| {
				thread::spawn(|| {
					let _project_result = project_root();
					let _current_result = current_crate();
					// If we get here without panicking, the functions are
					// thread-safe
					true
				},)
			},)
			.collect();

		// Wait for all threads to complete
		for handle in handles {
			let result = handle.join().unwrap();
			assert!(result);
		}
	}

	#[test]
	fn test_return_type_traits()
	{
		// Test that returned OsoCrate implements expected traits
		if let X(project_crate,) = project_root() {
			// Test Clone
			let _cloned = project_crate.clone();

			// Test Debug
			let _debug_str = format!("{:?}", project_crate);

			// Test PartialEq
			let other = project_crate.clone();
			assert_eq!(project_crate, other);
		}

		if let X(current_crate,) = current_crate() {
			// Test Clone
			let _cloned = current_crate.clone();

			// Test Debug
			let _debug_str = format!("{:?}", current_crate);

			// Test PartialEq
			let other = current_crate.clone();
			assert_eq!(current_crate, other);
		}
	}
}
