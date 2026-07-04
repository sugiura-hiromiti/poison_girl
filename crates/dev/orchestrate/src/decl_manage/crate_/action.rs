use {
	super::{CrateInfo, PoisonGirlCrate, PoisonGirlCrateChart},
	crate::{
		CliCommandDiscriminants, Policy, decl_manage::PoisonGirlCargoInterface,
	},
	poison_girl_dev_error::PoisonGirlB,
};

/// methods provided keeps environment e.g. current path
pub trait CrateAction: CrateInfo
{
	// actions for all packages

	fn build(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx(CliCommandDiscriminants::Build,)
	}

	fn test(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx(CliCommandDiscriminants::Test,)
	}

	fn run(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx(CliCommandDiscriminants::Run,)
	}

	fn clippy(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx(CliCommandDiscriminants::Clippy,)
	}

	fn fix(&self,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx(CliCommandDiscriminants::Fix,)
	}

	fn cargo_xxx(&self, cmd: CliCommandDiscriminants,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with(cmd, &Policy::from_cmd(cmd,),)
	}

	// actions for all packages with specific options

	fn build_with(&self, opt: &Policy,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with(CliCommandDiscriminants::Build, opt,)
	}

	fn test_with(&self, opt: &Policy,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with(CliCommandDiscriminants::Test, opt,)
	}

	fn run_with(&self, opt: &Policy,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with(CliCommandDiscriminants::Run, opt,)
	}

	fn clippy_with(&self, opt: &Policy,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with(CliCommandDiscriminants::Clippy, opt,)
	}

	fn fix_with(&self, opt: &Policy,) -> PoisonGirlB<(),>
	{
		self.cargo_xxx_with(CliCommandDiscriminants::Fix, opt,)
	}

	fn cargo_xxx_with(
		&self,
		cmd: CliCommandDiscriminants,
		opt: &Policy,
	) -> PoisonGirlB<(),>
	{
		let opt = opt.reuse_args(cmd,)?;
		let interface = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::from(self.path(),),
			opt,
		);
		interface.run()
	}
}

impl CrateAction for PoisonGirlCrate
{
}
