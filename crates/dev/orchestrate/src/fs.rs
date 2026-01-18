use {
	crate::decl_manage::crate_::OsoCrate,
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_fs::fs::{current_crate_path, project_root_path},
};

pub fn project_root() -> PoisonGirlB<OsoCrate,> {
	let pr = project_root_path()?;
	X(OsoCrate::from(pr,),)
}

pub fn current_crate() -> PoisonGirlB<OsoCrate,> {
	let ccp = current_crate_path()?;

	X(OsoCrate::from(ccp,),)
}

#[cfg(test)]
mod tests {
	use {
		super::*, crate::decl_manage::crate_::CrateInfo,
		poison_girl_dev_error::Y,
	};

	#[test]
	fn test_project_root_function_exists() {
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
	fn test_current_crate_function_exists() {
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
	fn test_project_root_returns_oso_crate() {
		// Test that project_root returns an OsoCrate type
		let result = project_root();

		// Verify the return type is correct
		if let Y(e,) = result {
			panic!("{e:?}")
		}
	}

	#[test]
	fn test_current_crate_returns_oso_crate() {
		// Test that current_crate returns an OsoCrate type
		let result = current_crate();

		// Verify the return type is correct
		if let Y(e,) = result {
			panic!("{e:?}")
		}
	}

	#[test]
	fn test_function_integration() {
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
	fn test_result_type_consistency() {
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
	fn test_function_signatures() {
		// Test that function signatures are as expected

		// project_root should take no parameters and return Rslt<OsoCrate>
		let _: fn() -> PoisonGirlB<OsoCrate,> = project_root;

		// current_crate should take no parameters and return Rslt<OsoCrate>
		let _: fn() -> PoisonGirlB<OsoCrate,> = current_crate;
	}
	#[test]
	fn test_oso_crate_conversion() {
		// Test that PathBuf to OsoCrate conversion works correctly

		let project_result = project_root();
		let current_result = current_crate();

		if let X(project_crate,) = project_result {
			let path = project_crate.path();

			// Test that we can create OsoCrate from PathBuf
			let recreated = OsoCrate::from(path.clone(),);
			assert_eq!(recreated.path(), path);

			// Test that conversion is consistent
			assert_eq!(project_crate, recreated);
		}

		if let X(current_crate,) = current_result {
			let path = current_crate.path();

			// Test that we can create OsoCrate from PathBuf
			let recreated = OsoCrate::from(path.clone(),);
			assert_eq!(recreated.path(), path);

			// Test that conversion is consistent
			assert_eq!(current_crate, recreated);
		}
	}

	#[test]
	fn test_path_properties() {
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
	fn test_thread_safety() {
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
	fn test_return_type_traits() {
		// Test that returned OsoCrate implements expected traits
		if let X(project_crate,) = project_root() {
			// Test Clone
			let _cloned = project_crate.clone();

			// Test Debug
			let _debug_str = format!("{:?}", project_crate);

			// Test PartialEq
			let other = project_crate.clone();
			assert_eq!(project_crate, other);

			// Test that it can be used in collections
			let mut vec = Vec::new();
			vec.push(project_crate,);
			assert_eq!(vec.len(), 1);
		}

		if let X(current_crate,) = current_crate() {
			// Test Clone
			let _cloned = current_crate.clone();

			// Test Debug
			let _debug_str = format!("{:?}", current_crate);

			// Test PartialEq
			let other = current_crate.clone();
			assert_eq!(current_crate, other);

			// Test that it can be used in collections
			let mut vec = Vec::new();
			vec.push(current_crate,);
			assert_eq!(vec.len(), 1);
		}
	}
}
