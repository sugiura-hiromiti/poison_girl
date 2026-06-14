use {super::table::boot_services, poison_girl_no_std_error::X};

pub fn exit_boot_services()
{
	if let X(bs,) = boot_services() {
		let _ = bs.exit_boot_services();
	}
}
