use {
	super::{Crate, CrateCalled, CrateInfo, PoisonGirlCrate},
	crate::{
		CliCommandDiscriminants, Policy,
		decl_manage::{
			PoisonGirlCargoInterface,
			package::PackageSurvey,
			workspace::{
				Workspace, WorkspaceAction, WorkspaceInfo, WorkspaceSurvey,
			},
		},
	},
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_fs::all_crates_in,
};

impl Crate for PoisonGirlCrate
{
}

impl Workspace for PoisonGirlCrate
{
}

impl WorkspaceAction for PoisonGirlCrate
{
	fn cargo_xxx_at(
		&self,
		cmd: CliCommandDiscriminants,
		at: impl CrateCalled,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		let target_crate = PoisonGirlCargoInterface {
			ws:     PoisonGirlCrate::from(at.path_buf(),),
			policy: Policy::from_cmd(cmd,),
		};
		target_crate.run()
	}

	fn cargo_xxx_at_with(
		&self,
		cmd: CliCommandDiscriminants,
		at: impl CrateCalled,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	where
		Self: WorkspaceSurvey,
	{
		let opt = opt.reuse_args(cmd,)?;
		let target_crate = PoisonGirlCargoInterface {
			ws:     PoisonGirlCrate::from(at.path_buf(),),
			policy: opt.clone(),
		};
		target_crate.run()
	}
}

impl WorkspaceSurvey for PoisonGirlCrate
{
}

impl WorkspaceInfo for PoisonGirlCrate
{
	#[allow(refining_impl_trait)]
	fn members(&self,) -> PoisonGirlB<Vec<PoisonGirlCrate,>,>
	{
		X(all_crates_in(&self.path(),)?
			.iter()
			.map(|p| PoisonGirlCrate::from(p.clone(),),)
			.collect(),)
	}

	#[allow(refining_impl_trait)]
	fn members_with_target(
		&self,
		target: impl Into<String,> + Clone,
	) -> PoisonGirlB<Vec<PoisonGirlCrate,>,>
	{
		let target: String = target.into();
		let mut members = vec![];
		for c in self.members()? {
			let dflt_target: String = c.default_target()?.into();
			if dflt_target == target {
				members.push(c,);
			}
		}
		X(members,)
	}
}
