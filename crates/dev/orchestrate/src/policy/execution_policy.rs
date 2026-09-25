use {crate::cli_interface::TargetKind, poison_girl_dev_cargo::Runtime};

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
