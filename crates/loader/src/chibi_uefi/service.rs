use {super::table::boot_services, poison_girl_no_std_error::PoisonGirlB};

pub fn exit_boot_services() -> PoisonGirlB<(),>
{
	let bs = boot_services()?;
	bs.exit_boot_services()
}
