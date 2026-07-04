use crate::decl_manage::{package::Package, workspace::Workspace};

mod action;
mod identity;
mod info;
mod package_impl;
mod survey;
mod workspace_impl;

pub use self::{
	action::CrateAction,
	identity::{CrateCalled, PoisonGirlCrate, PoisonGirlCrateChart},
	info::CrateInfo,
	survey::CrateSurvey,
};

pub trait Crate: Workspace + Package
{
	fn as_pkg(&self,) -> &impl Package
	{
		self
	}

	fn as_wrkspc(&self,) -> &impl Workspace
	{
		self
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		poison_girl_dev_test::{PoisonGirlTestB, success},
		std::path::PathBuf,
	};

	// Note: The FromPathBuf macro validates paths and panics on non-existent
	// paths This is a suspected program bug - tests should be able to use mock
	// paths Working around this by using the current directory which should
	// exist

	#[test]
	fn test_poison_girl_crate_default()
	{
		let default_crate = PoisonGirlCrate::default();
		let default_path = default_crate.path();
		// Default should create an empty PathBuf
		assert_eq!(default_path, PathBuf::new());
	}

	#[test]
	fn test_poison_girl_crate_creation_with_current_dir()
	{
		// Use current directory which should exist
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);
		assert_eq!(crate_obj.path(), current_dir);
	}

	#[test]
	fn test_poison_girl_crate_clone_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let original = PoisonGirlCrate::from(current_dir.clone(),);
		let cloned = original.clone();

		assert_eq!(original.path(), cloned.path());
		assert_eq!(original, cloned);
	}

	#[test]
	fn test_poison_girl_crate_equality_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate1 = PoisonGirlCrate::from(current_dir.clone(),);
		let crate2 = PoisonGirlCrate::from(current_dir.clone(),);

		assert_eq!(crate1, crate2);
	}

	#[test]
	fn test_crate_info_path_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test CrateInfo::path method
		assert_eq!(crate_obj.path(), current_dir);
	}

	#[test]
	fn test_crate_called_whoami_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test CrateCalled::whoami method
		let whoami_result = crate_obj.whoami();
		assert_eq!(whoami_result.path_buf(), current_dir);
	}

	#[test]
	fn test_crate_called_path_buf_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test CrateCalled::path_buf method
		assert_eq!(crate_obj.path_buf(), current_dir);
	}

	#[test]
	fn test_from_pathbuf_conversion_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);

		// Test From<PathBuf> implementation
		let crate_obj: PoisonGirlCrate = current_dir.clone().into();
		assert_eq!(crate_obj.path(), current_dir);

		// Test explicit From::from
		let crate_obj2 = PoisonGirlCrate::from(current_dir.clone(),);
		assert_eq!(crate_obj2.path(), current_dir);

		// Both should be equal
		assert_eq!(crate_obj, crate_obj2);
	}

	#[test]
	fn test_poison_girl_crate_chart_conversion_with_current_dir()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir.clone(),);

		// Test that we can get the chart representation
		let chart = crate_obj.whoami();

		// Chart should convert back to the same path
		assert_eq!(chart.path_buf(), current_dir);

		let crate_from_chart = PoisonGirlCrate::from(chart,);
		assert_eq!(crate_from_chart.path(), current_dir);
	}

	#[test]
	fn test_debug_implementation()
	{
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = PoisonGirlCrate::from(current_dir,);

		// Test that Debug is implemented
		let debug_string = format!("{:?}", crate_obj);
		assert!(debug_string.contains("PoisonGirlCrate"));
		assert!(debug_string.contains("path"));
	}

	#[test]
	fn test_workspace_survey_land_on() -> PoisonGirlTestB
	{
		let mut crate_obj =
			PoisonGirlCrate::from(PoisonGirlCrateChart::DevOrchestrate,);
		let target_crate = PoisonGirlCrate::from(PoisonGirlCrateChart::DevFs,);
		let target_path = target_crate.path();
		let cwd_before = std::env::current_dir()?;

		crate_obj.land_on(target_crate,)?;

		assert_eq!(crate_obj.path(), target_path);
		assert_eq!(std::env::current_dir()?, cwd_before);
		success!()
	}
}
