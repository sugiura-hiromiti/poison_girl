use {
	crate::{
		CliCommandDiscriminants, Policy,
		decl_manage::crate_::{
			Crate, CrateAction, CrateCalled, CrateInfo, CrateSurvey,
			PoisonGirlCrateChart,
		},
	},
	poison_girl_dev_error::{PoisonGirlB, X},
};

pub trait Workspace: WorkspaceAction + WorkspaceSurvey
{
	fn as_action(&self,) -> &impl WorkspaceAction
	{
		self
	}

	fn as_survey(&self,) -> &impl WorkspaceSurvey
	{
		self
	}
}

pub trait WorkspaceAction: WorkspaceInfo + CrateAction
{
	// actions for specific package

	fn build_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at(CliCommandDiscriminants::Build, at,)
	}

	fn test_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at(CliCommandDiscriminants::Test, at,)
	}

	fn run_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at(CliCommandDiscriminants::Run, at,)
	}

	fn clippy_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at(CliCommandDiscriminants::Clippy, at,)
	}

	fn fix_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at(CliCommandDiscriminants::Fix, at,)
	}

	fn cargo_xxx_at(
		&self,
		cmd: CliCommandDiscriminants,
		at: impl CrateCalled,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey;

	// actions for specific package with specific options

	fn build_at_with(
		&self,
		at: impl CrateCalled,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with(CliCommandDiscriminants::Build, at, opt,)
	}

	fn test_at_with(
		&self,
		at: impl CrateCalled,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with(CliCommandDiscriminants::Test, at, opt,)
	}

	fn run_at_with(
		&self,
		at: impl CrateCalled,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with(CliCommandDiscriminants::Run, at, opt,)
	}

	/// Kernel and loader use custom targets for production code, but their
	/// unit tests need the host target because they depend on `std`.
	fn clippy_at_with(
		&self,
		at: impl CrateCalled,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		let chart = PoisonGirlCrateChart::from(at.path_buf(),);
		if chart.uses_custom_runtime() && opt.clippy_lints_all_targets() {
			let custom_lib = opt.clone().with_clippy_custom_target_lib()?;
			self.cargo_xxx_at_with(
				CliCommandDiscriminants::Clippy,
				at.clone(),
				&custom_lib,
			)?;

			let host_tests = opt.clone().with_clippy_host_tests()?;
			self.cargo_xxx_at_with(
				CliCommandDiscriminants::Clippy,
				at,
				&host_tests,
			)?;
			return X((),);
		}

		self.cargo_xxx_at_with(CliCommandDiscriminants::Clippy, at, opt,)
	}

	fn fix_at_with(
		&self,
		at: impl CrateCalled,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with(CliCommandDiscriminants::Fix, at, opt,)
	}

	fn cargo_xxx_at_with(
		&self,
		cmd: CliCommandDiscriminants,
		at: impl CrateCalled,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey;
}

pub trait WorkspaceSurvey: WorkspaceInfo + CrateSurvey
{
}

/// Trait for managing poison_girl workspace operations
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
/// use poison_girl_dev_util::PoisonGirlWorkspace;
///
/// fn process_workspace<W: PoisonGirlWorkspace>(workspace: &W) {
///     let root = workspace.root();
///     println!("Processing workspace at: {}", root.display());
///
///     for crate_path in workspace.crates() {
///         println!("Found crate: {}", crate_path.display());
///     }
/// }
/// ```
pub trait WorkspaceInfo: Sized + CrateInfo
{
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
	fn members(&self,) -> PoisonGirlB<Vec<impl Crate,>,>;

	fn members_with_target(
		&self,
		target: impl Into<String,> + Clone,
	) -> PoisonGirlB<Vec<impl Crate,>,>;
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::decl_manage::crate_::{
			CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart,
		},
		poison_girl_dev_test::{PoisonGirlTestB, success},
	};

	#[test]
	fn test_workspace_survey_land_on() -> PoisonGirlTestB
	{
		let mut workspace =
			PoisonGirlCrate::from(PoisonGirlCrateChart::DevOrchestrate,);
		let target = PoisonGirlCrate::from(PoisonGirlCrateChart::DevFs,);
		let target_path = target.path();

		workspace.land_on(target,)?;

		assert_eq!(workspace.path(), target_path);
		success!()
	}
}
