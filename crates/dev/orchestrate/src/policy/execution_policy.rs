use {
	crate::cli_interface::{
		CargoInvocation, RenderCargoInvocation, TargetKind,
	},
	poison_girl_dev_cargo::Runtime,
	poison_girl_dev_error::B::{self, X, Y},
};

pub struct ExecutionPolicy
{
	runtime:     Runtime,
	target_kind: TargetKind,
}

impl ExecutionPolicy
{
	pub(crate) fn new(runtime: Runtime, target_kind: TargetKind,) -> Self
	{
		Self { runtime, target_kind, }
	}

	pub(crate) fn runtime(&self,) -> Runtime
	{
		self.runtime
	}

	pub(crate) fn target_kind(&self,) -> TargetKind
	{
		self.target_kind
	}
}

impl TargetKind
{
	fn target(&self,) -> B<&str, (),>
	{
		match self {
			Self::Auto | Self::Bin => Y((),),
			Self::Lib => X("--lib",),
			Self::Test => X("--test",),
		}
	}
}

impl RenderCargoInvocation for ExecutionPolicy
{
	fn render(&self,) -> CargoInvocation
	{
		let mut invocation = CargoInvocation::default();
		let target_kind = match self.target_kind.target() {
			X(t,) => t,
			Y(_,) => return invocation,
		};

		invocation.extend(CargoInvocation::from_cargo_args(vec![
			target_kind.to_string(),
		],),);
		invocation
	}
}
