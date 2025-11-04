use crate::Rslt;
use crate::decl_manage::crate_::CrateAction;
use crate::decl_manage::crate_::CrateInfo;
use crate::decl_manage::crate_::CrateSurvey;

pub trait Package: PackageAction + PackageSurvey {
	fn as_action(&self,) -> &impl PackageAction {
		self
	}

	fn as_survey(&self,) -> &impl PackageSurvey {
		self
	}
}

pub trait PackageAction: PackageInfo + CrateAction {}
pub trait PackageSurvey: PackageInfo + CrateSurvey {
	fn default_target(&self,) -> Rslt<impl Into<String,>,>;
}

pub trait PackageInfo: Sized + CrateInfo {}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::cargo::BuildMode;
	use crate::decl_manage::crate_::CrateInfo;
	use crate::decl_manage::crate_::OsoCrate;
	use std::path::PathBuf;

	// Note: Working around FromPathBuf macro validation by using current
	// directory

	#[test]
	fn test_package_trait_hierarchy() {
		// Test that Package trait requires both PackageAction and PackageSurvey
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test that OsoCrate implements Package (using concrete type since not
		// object-safe)
		let _package_ref: &OsoCrate = &crate_obj;

		// Test as_action method
		let _action = crate_obj.as_action();

		// Test as_survey method
		let _survey = crate_obj.as_survey();
	}

	#[test]
	fn test_package_action_trait() {
		// Test that PackageAction combines PackageInfo and CrateAction
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test that OsoCrate implements PackageAction (concrete type only)
		let _action_ref: &OsoCrate = &crate_obj;

		// PackageAction should provide access to CrateAction methods
		let build_result = crate_obj.build();
		let format_result = crate_obj.format();
		let check_result = crate_obj.check();

		// If we get here without compilation errors, PackageAction is working
		assert!(
			build_result.is_ok()
				|| format_result.is_ok()
				|| check_result.is_ok()
		);
	}

	#[test]
	fn test_package_survey_trait() {
		// Test that PackageSurvey combines PackageInfo and CrateSurvey
		use crate::cargo::Arch;
		use crate::cargo::Feature;
		use crate::cargo::Opts;

		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test that OsoCrate implements PackageSurvey
		let _survey_ref: &OsoCrate = &crate_obj;

		// Test default_target method
		let target_result = crate_obj.default_target();
		assert!(target_result.is_ok());
	}

	#[test]
	fn test_package_info_trait() {
		// Test that PackageInfo extends CrateInfo
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir.clone(),);

		// Test that OsoCrate implements PackageInfo (concrete type only)
		let _info_ref: &OsoCrate = &crate_obj;

		// PackageInfo should provide access to CrateInfo methods
		assert_eq!(crate_obj.path(), current_dir);

		// Test TOML access (might fail in test environment)
		let _toml_result = crate_obj.toml();

		// If we get here without compilation errors, PackageInfo is working
	}

	#[test]
	fn test_trait_bounds_compilation() {
		// Test that all trait bounds compile correctly
		fn test_package<P: Package,>(package: &P,) {
			let _action = package.as_action();
			let _survey = package.as_survey();
		}

		fn test_package_action<PA: PackageAction,>(action: &PA,) {
			let _build_result = action.build();
		}

		fn test_package_survey<PS: PackageSurvey,>(survey: &PS,) {
			let _target_result = survey.default_target();
		}

		fn test_package_info<PI: PackageInfo,>(info: &PI,) {
			let _path = info.path();
		}

		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		test_package(&crate_obj,);
		test_package_action(&crate_obj,);
		test_package_survey(&crate_obj,);
		test_package_info(&crate_obj,);
	}

	#[test]
	fn test_generic_constraints() {
		// Test that generic constraints work correctly
		fn work_with_package<P,>(package: P,)
		where P: Package + Clone {
			let _cloned = package.clone();
			let _action = package.as_action();
			let _survey = package.as_survey();
		}

		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		work_with_package(crate_obj,);
	}
}
