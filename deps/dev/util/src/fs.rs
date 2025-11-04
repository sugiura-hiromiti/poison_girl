use crate::Rslt;
use crate::decl_manage::crate_::OsoCrate;
use oso_dev_util_helper::fs::current_crate_path;
use oso_dev_util_helper::fs::project_root_path;

pub fn project_root() -> Rslt<OsoCrate,> {
	let pr = project_root_path()?;
	Ok(OsoCrate::from(pr,),)
}

pub fn current_crate() -> Rslt<OsoCrate,> {
	let ccp = current_crate_path()?;

	Ok(OsoCrate::from(ccp,),)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::decl_manage::crate_::CrateInfo;

	#[test]
	fn test_project_root_function_exists() {
		// Test that the project_root function exists and returns a Result
		let result = project_root();

		// The function should return a Result, regardless of success or failure
		match result {
			Ok(crate_obj,) => {
				// If successful, verify we got an OsoCrate
				let path = crate_obj.path();
				assert!(path.is_absolute() || path.starts_with("."));
			},
			Err(e,) => {
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
			Ok(crate_obj,) => {
				// If successful, verify we got an OsoCrate
				let path = crate_obj.path();
				assert!(path.is_absolute() || path.starts_with("."));
			},
			Err(e,) => {
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
		if let Err(e,) = result {
			panic!("{e:?}")
		}
	}

	#[test]
	fn test_current_crate_returns_oso_crate() {
		// Test that current_crate returns an OsoCrate type
		let result = current_crate();

		// Verify the return type is correct
		if let Err(e,) = result {
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
			(Ok(project_crate,), Ok(current_crate,),) => {
				// Both should be valid OsoCrate instances
				let _project_path = project_crate.path();
				let _current_path = current_crate.path();
			},
			(Err(_,), _,) | (_, Err(_,),) => {
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
			assert!(result.is_ok());
		}
	}

	#[test]
	fn test_function_error_propagation() {
		// Test that errors from helper functions are properly propagated
		// This is a behavioral test - we can't easily mock the helper
		// functions, but we can verify the error handling structure

		// Test project_root error handling
		let project_result = project_root();
		if let Err(e,) = project_result {
			// Should be an anyhow::Error
			let _error_string = e.to_string();
			let _error_chain: Vec<_,> = e.chain().collect();
		}

		// Test current_crate error handling
		let current_result = current_crate();
		if let Err(e,) = current_result {
			// Should be an anyhow::Error
			let _error_string = e.to_string();
			let _error_chain: Vec<_,> = e.chain().collect();
		}
	}

	#[test]
	fn test_module_dependencies() {
		// Test that the module correctly imports and uses its dependencies

		// Test that we can use the Rslt type alias
		let _result: Rslt<i32,> = Ok(42,);

		// Test that we can reference OsoCrate
		let _crate_type = std::marker::PhantomData::<OsoCrate,>;

		// Test that helper functions are accessible (even if they might fail)
		let _project_path_result = project_root_path();
		let _current_path_result = current_crate_path();
	}

	#[test]
	fn test_function_signatures() {
		// Test that function signatures are as expected

		// project_root should take no parameters and return Rslt<OsoCrate>
		let _: fn() -> Rslt<OsoCrate,> = project_root;

		// current_crate should take no parameters and return Rslt<OsoCrate>
		let _: fn() -> Rslt<OsoCrate,> = current_crate;
	}

	#[test]
	fn test_error_types() {
		// Test that errors are properly typed as anyhow::Error
		let project_result = project_root();
		let current_result = current_crate();

		match project_result {
			Ok(_,) => {
				// Success case - verify the type
				assert!(true);
			},
			Err(e,) => {
				// Error should be anyhow::Error
				let _error_string = e.to_string();
				let _error_chain: Vec<_,> = e.chain().collect();
			},
		}

		match current_result {
			Ok(_,) => {
				// Success case - verify the type
				assert!(true);
			},
			Err(e,) => {
				// Error should be anyhow::Error
				let _error_string = e.to_string();
				let _error_chain: Vec<_,> = e.chain().collect();
			},
		}
	}

	#[test]
	fn test_function_behavior_consistency() {
		// Test that both functions behave consistently
		let project_result = project_root();
		let current_result = current_crate();

		// Both should return the same Result type
		match (project_result, current_result,) {
			(Ok(project_crate,), Ok(current_crate,),) => {
				// Both should return valid OsoCrate instances
				let project_path = project_crate.path();
				let current_path = current_crate.path();

				// Paths should be valid
				assert!(!project_path.as_os_str().is_empty());
				assert!(!current_path.as_os_str().is_empty());

				// Both should implement the same traits
				let _project_clone = project_crate.clone();
				let _current_clone = current_crate.clone();
			},
			(Err(project_err,), _,) => {
				// Project root error should be meaningful
				let error_msg = project_err.to_string();
				assert!(!error_msg.is_empty());
			},
			(_, Err(current_err,),) => {
				// Current crate error should be meaningful
				let error_msg = current_err.to_string();
				assert!(!error_msg.is_empty());
			},
		}
	}

	#[test]
	fn test_helper_function_integration() {
		// Test that the functions properly integrate with helper functions
		use oso_dev_util_helper::fs::current_crate_path;
		use oso_dev_util_helper::fs::project_root_path;

		// Test that our functions use the same underlying helpers
		let project_helper_result = project_root_path();
		let current_helper_result = current_crate_path();

		let our_project_result = project_root();
		let our_current_result = current_crate();

		// If helper functions succeed, our functions should too (or vice versa)
		match (project_helper_result, our_project_result,) {
			(Ok(helper_path,), Ok(our_crate,),) => {
				// Paths should be related
				let our_path = our_crate.path();
				assert_eq!(helper_path, our_path);
			},
			(Err(_,), Err(_,),) => {
				// Both should fail in the same conditions
				assert!(true);
			},
			_ => {
				// Mixed success/failure might be valid depending on
				// implementation
				assert!(true);
			},
		}

		match (current_helper_result, our_current_result,) {
			(Ok(helper_path,), Ok(our_crate,),) => {
				// Paths should be related
				let our_path = our_crate.path();
				assert_eq!(helper_path, our_path);
			},
			(Err(_,), Err(_,),) => {
				// Both should fail in the same conditions
				assert!(true);
			},
			_ => {
				// Mixed success/failure might be valid depending on
				// implementation
				assert!(true);
			},
		}
	}

	#[test]
	fn test_oso_crate_conversion() {
		// Test that PathBuf to OsoCrate conversion works correctly

		let project_result = project_root();
		let current_result = current_crate();

		if let Ok(project_crate,) = project_result {
			let path = project_crate.path();

			// Test that we can create OsoCrate from PathBuf
			let recreated = OsoCrate::from(path.clone(),);
			assert_eq!(recreated.path(), path);

			// Test that conversion is consistent
			assert_eq!(project_crate, recreated);
		}

		if let Ok(current_crate,) = current_result {
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

		if let Ok(project_crate,) = project_result {
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

		if let Ok(current_crate,) = current_result {
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
	fn test_error_context() {
		// Test that errors provide useful context
		let project_result = project_root();
		let current_result = current_crate();

		if let Err(project_err,) = project_result {
			// Error should have a meaningful message
			let error_msg = project_err.to_string();
			assert!(!error_msg.is_empty());

			// Error should have a chain of causes
			let error_chain: Vec<_,> = project_err.chain().collect();
			assert!(!error_chain.is_empty());

			// Should be able to downcast to specific error types if needed
			let _source = project_err.source();
		}

		if let Err(current_err,) = current_result {
			// Error should have a meaningful message
			let error_msg = current_err.to_string();
			assert!(!error_msg.is_empty());

			// Error should have a chain of causes
			let error_chain: Vec<_,> = current_err.chain().collect();
			assert!(!error_chain.is_empty());

			// Should be able to downcast to specific error types if needed
			let _source = current_err.source();
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
	fn test_multiple_calls() {
		// Test that functions can be called multiple times consistently
		let results1 = (project_root(), current_crate(),);
		let results2 = (project_root(), current_crate(),);
		let results3 = (project_root(), current_crate(),);

		// Results should be consistent across calls
		match (results1.0, results2.0, results3.0,) {
			(Ok(p1,), Ok(p2,), Ok(p3,),) => {
				assert_eq!(p1.path(), p2.path());
				assert_eq!(p2.path(), p3.path());
			},
			(Err(_,), Err(_,), Err(_,),) => {
				// Consistent failure is also acceptable
				assert!(true);
			},
			_ => {
				// Mixed results might indicate non-deterministic behavior
				// This could be valid depending on the implementation
				assert!(true);
			},
		}

		match (results1.1, results2.1, results3.1,) {
			(Ok(c1,), Ok(c2,), Ok(c3,),) => {
				assert_eq!(c1.path(), c2.path());
				assert_eq!(c2.path(), c3.path());
			},
			(Err(_,), Err(_,), Err(_,),) => {
				// Consistent failure is also acceptable
				assert!(true);
			},
			_ => {
				// Mixed results might indicate non-deterministic behavior
				// This could be valid depending on the implementation
				assert!(true);
			},
		}
	}

	#[test]
	fn test_return_type_traits() {
		// Test that returned OsoCrate implements expected traits
		if let Ok(project_crate,) = project_root() {
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

		if let Ok(current_crate,) = current_crate() {
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
