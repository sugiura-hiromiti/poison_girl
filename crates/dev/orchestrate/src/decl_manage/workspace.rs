use {
	crate::decl_manage::crate_::{
		Crate, CrateAction, CrateCalled, CrateInfo, CrateSurvey,
	},
	poison_girl_dev_error::{
		PointerOperationFailed, PoisonGirlB, ReShape, X, poison_girl_err,
	},
	std::ffi::OsStr,
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
		self.cargo_xxx_at("build", at,)
	}

	fn test_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at("test", at,)
	}

	fn run_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at("run", at,)
	}

	fn check_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at("check", at,)
	}

	fn fmt_at(&self, at: impl CrateCalled,) -> PoisonGirlB<(),>
	where Self: WorkspaceSurvey
	{
		self.cargo_xxx_at("fmt", at,)
	}

	fn cargo_xxx_at(
		&self,
		cmd: impl AsRef<OsStr,>,
		at: impl CrateCalled,
	) -> PoisonGirlB<(),>
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
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("build", at, opt,)
	}

	fn test_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("test", at, opt,)
	}

	fn run_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("run", at, opt,)
	}

	fn check_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		self.cargo_xxx_at_with("check", at, opt,)
	}

	fn fmt_at_with(
		&self,
		at: impl CrateCalled,
		opt: &[impl AsRef<OsStr,>],
	) -> PoisonGirlB<(),>
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
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		let current = self.whoami();
		//  this operation is safe due to `&self` is valid
		let self_mut = unsafe { (self as *const Self).cast_mut().as_mut() }
			.reshape(poison_girl_err!(PointerOperationFailed),)?;
		self_mut.land_on(at,);
		self_mut.cargo_xxx_with(cmd, opt,)?;
		self_mut.land_on(current,);
		X((),)
	}
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
	fn members(&self,) -> Vec<impl Crate,>;

	fn members_with_target(
		&self,
		target: impl Into<String,> + Clone,
	) -> Vec<impl Crate,>;
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		crate::decl_manage::crate_::{
			CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart,
		},
	};

	#[test]
	fn test_workspace_survey_land_on()
	{
		let mut workspace =
			PoisonGirlCrate::from(PoisonGirlCrateChart::DevOrchestrate,);
		let target = PoisonGirlCrate::from(PoisonGirlCrateChart::DevFs,);
		let target_path = target.path();

		workspace.land_on(target,);

		assert_eq!(workspace.path(), target_path);
	}
}
