use crate::decl_manage::crate_::Crate;
use crate::decl_manage::crate_::CrateAction;
use crate::decl_manage::crate_::CrateCalled;
use crate::decl_manage::crate_::CrateInfo;
use crate::decl_manage::crate_::CrateSurvey;
use anyhow::Result as Rslt;
use std::ffi::OsStr;

pub trait Workspace: WorkspaceAction + WorkspaceSurvey {
	fn as_action(&self,) -> &impl WorkspaceAction {
		self
	}

	fn as_survey(&self,) -> &impl WorkspaceSurvey {
		self
	}
}

pub trait WorkspaceAction: WorkspaceInfo + CrateAction {
	// actions for specific package

	fn build_at(&self, at: impl CrateCalled,) -> Rslt<(),>
	where Self: WorkspaceSurvey {
		self.cargo_xxx_at("build", at,)
	}
	fn test_at(&self, at: impl CrateCalled,) -> Rslt<(),>
	where Self: WorkspaceSurvey {
		self.cargo_xxx_at("test", at,)
	}
	fn run_at(&self, at: impl CrateCalled,) -> Rslt<(),>
	where Self: WorkspaceSurvey {
		self.cargo_xxx_at("run", at,)
	}
	fn check_at(&self, at: impl CrateCalled,) -> Rslt<(),>
	where Self: WorkspaceSurvey {
		self.cargo_xxx_at("check", at,)
	}
	fn fmt_at(&self, at: impl CrateCalled,) -> Rslt<(),>
	where Self: WorkspaceSurvey {
		self.cargo_xxx_at("fmt", at,)
	}
	fn cargo_xxx_at(
		&self,
		cmd: impl AsRef<OsStr,>,
		at: impl CrateCalled,
	) -> Rslt<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with(cmd, at, &["",],)
	}

	// actions for specific package with specific options

	fn build_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> Rslt<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("build", at, opt,)
	}
	fn test_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> Rslt<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("test", at, opt,)
	}
	fn run_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> Rslt<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("run", at, opt,)
	}
	fn check_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> Rslt<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("check", at, opt,)
	}
	fn fmt_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> Rslt<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("fmt", at, opt,)
	}
	fn cargo_xxx_at_with(
		&self,
		cmd: impl AsRef<OsStr,>,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> Rslt<(),>
	where
		Self: WorkspaceSurvey,
	{
		let current = self.whoami();
		//  this operation is safe due to `&self` is valid
		let self_mut =
			unsafe { (self as *const Self).cast_mut().as_mut().unwrap() };
		self_mut.land_on(at,);
		self_mut.cargo_xxx_with(cmd, opt,)?;
		self_mut.land_on(current,);
		Ok((),)
	}
}

pub trait WorkspaceSurvey: WorkspaceInfo + CrateSurvey {}

/// Trait for managing OSO workspace operations
///
/// This trait provides an interface for workspace management operations
/// including root directory access and crate enumeration. It's designed to work
/// with multi-crate Rust workspaces and provides a consistent API for workspace
/// operations.
///
/// # Type Parameters
///
/// * `'a` - Lifetime parameter for borrowed path references
///
/// # Examples
///
/// ```rust,ignore
/// use oso_dev_util::OsoWorkspace;
///
/// fn process_workspace<W: OsoWorkspace>(workspace: &W) {
///     let root = workspace.root();
///     println!("Processing workspace at: {}", root.display());
///
///     for crate_path in workspace.crates() {
///         println!("Found crate: {}", crate_path.display());
///     }
/// }
/// ```
pub trait WorkspaceInfo: Sized + CrateInfo {
	/// Returns a slice of paths to all crates in the workspace
	///
	/// # Returns
	///
	/// A slice of [`Path`] references, each pointing to a crate directory
	/// within the workspace. These paths are relative to the workspace root.
	///
	/// # Examples
	///
	/// ```rust,ignore
	/// let crates = workspace.crates();
	/// for crate_path in crates {
	///     let cargo_toml = crate_path.join("Cargo.toml");
	///     assert!(cargo_toml.exists());
	/// }
	/// ```
	fn members(&self,) -> Vec<impl Crate,>;

	fn members_with_target(
		&self,
		target: impl Into<String,> + Clone,
	) -> Vec<impl Crate,>;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::decl_manage::crate_::CrateInfo;
	use crate::decl_manage::crate_::OsoCrate;
	use std::path::PathBuf;

	#[test]
	fn test_workspace_action_at_methods() {
		// Test workspace action methods that operate on specific crates
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let workspace = OsoCrate::from(current_dir,);
		let target_crate = OsoCrate::from(parent_dir,);

		// Test action methods (they will likely fail in test environment)
		// Note: These methods require WorkspaceSurvey bound, which OsoCrate
		// implements ignore `test_at` method because running it in test cause
		// infinity loop ignore `run_at` too because this crate is library
		// crate. nothing to run.
		let _build_result = workspace.build_at(target_crate.clone(),);
		let _check_result = workspace.check_at(target_crate.clone(),);
		let _fmt_result = workspace.fmt_at(target_crate,);

		// If we get here without compilation errors, the methods exist
	}

	#[test]
	fn test_workspace_action_at_with_methods() {
		// Test workspace action methods with options
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let workspace = OsoCrate::from(current_dir,);
		let target_crate = OsoCrate::from(parent_dir,);

		let opts = ["--verbose",];

		// Test action methods with options
		// ignore `test_at_with` method because running it in test cause
		// infinity loop ignore `run_at_with` too because this crate is
		// library crate. nothing to run.
		let _build_result =
			workspace.build_at_with(target_crate.clone(), &opts,);
		let _check_result =
			workspace.check_at_with(target_crate.clone(), &opts,);
		let _fmt_result = workspace.fmt_at_with(target_crate, &["--all",],);

		// If we get here without compilation errors, the methods exist
	}

	#[test]
	fn test_workspace_action_cargo_xxx_at() {
		// Test the generic cargo_xxx_at method
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let workspace = OsoCrate::from(current_dir,);
		let target_crate = OsoCrate::from(parent_dir,);

		// Test generic cargo command
		let _result = workspace.cargo_xxx_at("clippy", target_crate,);

		// If we get here without compilation errors, the method exists
	}

	#[test]
	fn test_workspace_action_cargo_xxx_at_with() {
		// Test the generic cargo_xxx_at_with method
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let workspace = OsoCrate::from(current_dir,);
		let target_crate = OsoCrate::from(parent_dir,);

		let opts = ["--all-targets",];

		// Test generic cargo command with options
		let _result =
			workspace.cargo_xxx_at_with("clippy", target_crate, &opts,);

		// If we get here without compilation errors, the method exists
	}

	#[test]
	fn test_workspace_trait_as_methods() {
		// Test the as_action and as_survey methods
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test as_action returns something implementing WorkspaceAction
		let action = crate_obj.as_action();
		let _build_result = action.build();

		// Test as_survey returns something implementing WorkspaceSurvey
		let survey = crate_obj.as_survey();
		let _members = survey.members();
	}

	#[test]
	fn test_workspace_info_members_signature() {
		// Test that members method has the correct signature
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test the method signature
		let members = crate_obj.members();

		// Should return a Vec of items implementing Crate
		for _member in members {
			// Can't use trait objects due to object safety, but we can verify
			// the type
		}
	}

	#[test]
	fn test_workspace_info_members_with_target_signature() {
		// Test that members_with_target method has the correct signature
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		// Test with string literal
		let members1 = crate_obj.members_with_target("aarch64-unknown-linux",);

		// Test with String
		let target = String::from("riscv64-unknown-oso",);
		let members2 = crate_obj.members_with_target(target,);

		// Both should return Vec of Crate implementors
		for _member in members1.into_iter().chain(members2,) {
			// Can't use trait objects due to object safety, but we can verify
			// the type
		}
	}

	#[test]
	fn test_workspace_survey_land_on_signature() {
		// Test that land_on method has the correct signature
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let mut workspace = OsoCrate::from(current_dir,);
		let target = OsoCrate::from(parent_dir.clone(),);

		// Test land_on method
		workspace.land_on(target,);

		// Should have changed to target path
		assert_eq!(workspace.path(), parent_dir);
	}

	#[test]
	fn test_trait_bounds_compilation() {
		// Test that all trait bounds compile correctly
		fn test_workspace<W: Workspace,>(workspace: &W,) {
			let _action = workspace.as_action();
			let _survey = workspace.as_survey();
		}

		fn test_workspace_action<WA: WorkspaceAction,>(action: &WA,) {
			let _build_result = action.build();
		}

		fn test_workspace_survey<WS: WorkspaceSurvey,>(survey: &WS,) {
			let _members = survey.members();
		}

		fn test_workspace_info<WI: WorkspaceInfo,>(info: &WI,) {
			let _path = info.path();
			let _members = info.members();
		}

		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		test_workspace(&crate_obj,);
		test_workspace_action(&crate_obj,);
		test_workspace_survey(&crate_obj,);
		test_workspace_info(&crate_obj,);
	}

	#[test]
	fn test_workspace_integration() {
		// Test that Workspace trait integrates all functionality
		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let parent_dir =
			current_dir.parent().unwrap_or(&current_dir,).to_path_buf();

		let workspace = OsoCrate::from(current_dir.clone(),);
		let target = OsoCrate::from(parent_dir,);

		// Test Workspace functionality directly (not as trait object since not
		// object-safe)

		// Should have access to action methods through as_action
		let action = workspace.as_action();
		let _build_result = action.build();
		// Note: build_at requires WorkspaceSurvey bound, test it directly on
		// workspace
		let _build_at_result = workspace.build_at(target.clone(),);

		// Should have access to survey methods through as_survey
		let survey = workspace.as_survey();
		let _members = survey.members();
		let _target_members = survey.members_with_target("test-target",);

		// Should have access to info methods (inherited through action/survey)
		assert_eq!(workspace.path(), current_dir);
	}

	#[test]
	fn test_generic_constraints() {
		// Test that generic constraints work correctly
		fn work_with_workspace<W,>(workspace: W,)
		where W: Workspace + Clone {
			let _cloned = workspace.clone();
			let _action = workspace.as_action();
			let _survey = workspace.as_survey();
		}

		let current_dir =
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".",),);
		let crate_obj = OsoCrate::from(current_dir,);

		work_with_workspace(crate_obj,);
	}
}
