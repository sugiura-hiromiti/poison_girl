use {crate::cli_interface::TargetKind, poison_girl_dev_cargo::Runtime};

pub struct ExecutionPolicy
{
	runtime:     Runtime,
	target_kind: TargetKind,
}
