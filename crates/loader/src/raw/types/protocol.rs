use crate::c_style_enum;

c_style_enum! {
	pub enum InterfaceType: u32 => {
		NATIVE_INTERFACE = 0,
	}
}

c_style_enum! {
	pub enum DeviceType: u8 => {
		HARDWARE = 0x01,
		ACPI = 0x02,
		MESSAGING = 0x03,
		MEDIA = 0x04,
		BIOS_BOOT_SPEC = 0x05,
		END = 0x7f,
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,)]
#[repr(transparent)]
pub struct DeviceSubType(pub u8,);

impl DeviceSubType {
	/// ACPI Device Path.
	pub const ACPI: Self = Self(1,);
	/// ACPI _ADR Device Path.
	pub const ACPI_ADR: Self = Self(3,);
	/// Expanded ACPI Device Path.
	pub const ACPI_EXPANDED: Self = Self(2,);
	/// NVDIMM Device Path.
	pub const ACPI_NVDIMM: Self = Self(4,);
	/// BIOS Boot Specification Device Path.
	pub const BIOS_BOOT_SPECIFICATION: Self = Self(1,);
	/// End entire Device Path.
	pub const END_ENTIRE: Self = Self(0xff,);
	/// End this instance of a Device Path and start a new one.
	pub const END_INSTANCE: Self = Self(0x01,);
	/// BMC Device Path.
	pub const HARDWARE_BMC: Self = Self(6,);
	/// Controller Device Path.
	pub const HARDWARE_CONTROLLER: Self = Self(5,);
	/// Memory-mapped Device Path.
	pub const HARDWARE_MEMORY_MAPPED: Self = Self(3,);
	/// PCCARD Device Path.
	pub const HARDWARE_PCCARD: Self = Self(2,);
	/// PCI Device Path.
	pub const HARDWARE_PCI: Self = Self(1,);
	/// Vendor-Defined Device Path.
	pub const HARDWARE_VENDOR: Self = Self(4,);
	/// CD-ROM Media Device Path.
	pub const MEDIA_CD_ROM: Self = Self(2,);
	/// File Path Media Device Path.
	pub const MEDIA_FILE_PATH: Self = Self(4,);
	/// Hard Drive Media Device Path.
	pub const MEDIA_HARD_DRIVE: Self = Self(1,);
	/// PIWG Firmware File.
	pub const MEDIA_PIWG_FIRMWARE_FILE: Self = Self(6,);
	/// PIWG Firmware Volume.
	pub const MEDIA_PIWG_FIRMWARE_VOLUME: Self = Self(7,);
	/// Media Protocol Device Path.
	pub const MEDIA_PROTOCOL: Self = Self(5,);
	/// RAM Disk Device Path.
	pub const MEDIA_RAM_DISK: Self = Self(9,);
	/// Relative Offset Range.
	pub const MEDIA_RELATIVE_OFFSET_RANGE: Self = Self(8,);
	/// Vendor-Defined Media Device Path.
	pub const MEDIA_VENDOR: Self = Self(3,);
	/// 1394 Device Path.
	pub const MESSAGING_1394: Self = Self(4,);
	/// ATAPI Device Path.
	pub const MESSAGING_ATAPI: Self = Self(1,);
	/// Bluetooth Device Path.
	pub const MESSAGING_BLUETOOTH: Self = Self(27,);
	/// BluetoothLE Device Path.
	pub const MESSAGING_BLUETOOTH_LE: Self = Self(30,);
	/// Device Logical Unit.
	pub const MESSAGING_DEVICE_LOGICAL_UNIT: Self = Self(17,);
	/// DNS Device Path.
	pub const MESSAGING_DNS: Self = Self(31,);
	/// eMMC (Embedded Multi-Media Card) Device Path.
	pub const MESSAGING_EMMC: Self = Self(29,);
	/// Fibre Channel Device Path.
	pub const MESSAGING_FIBRE_CHANNEL: Self = Self(3,);
	/// Fibre Channel Ex Device Path.
	pub const MESSAGING_FIBRE_CHANNEL_EX: Self = Self(21,);
	/// I2O Device Path.
	pub const MESSAGING_I2O: Self = Self(6,);
	/// Infiniband Device Path.
	pub const MESSAGING_INFINIBAND: Self = Self(9,);
	/// IPV4 Device Path.
	pub const MESSAGING_IPV4: Self = Self(12,);
	/// IPV6 Device Path.
	pub const MESSAGING_IPV6: Self = Self(13,);
	/// iSCSI Device Path node (base information).
	pub const MESSAGING_ISCSI: Self = Self(19,);
	/// MAC Address Device Path.
	pub const MESSAGING_MAC_ADDRESS: Self = Self(11,);
	/// NVDIMM Namespace Device Path.
	pub const MESSAGING_NVDIMM_NAMESPACE: Self = Self(32,);
	/// NVM Express Namespace Device Path.
	pub const MESSAGING_NVME_NAMESPACE: Self = Self(23,);
	/// NVME over Fabric (NVMe-oF) Namespace Device Path.
	pub const MESSAGING_NVME_OF_NAMESPACE: Self = Self(34,);
	/// REST Service Device Path.
	pub const MESSAGING_REST_SERVICE: Self = Self(33,);
	/// SATA Device Path.
	pub const MESSAGING_SATA: Self = Self(18,);
	/// SCSI Device Path.
	pub const MESSAGING_SCSI: Self = Self(2,);
	/// Serial Attached SCSI (SAS) Ex Device Path.
	pub const MESSAGING_SCSI_SAS_EX: Self = Self(22,);
	/// SD (Secure Digital) Device Path.
	pub const MESSAGING_SD: Self = Self(26,);
	/// UART Device Path.
	pub const MESSAGING_UART: Self = Self(14,);
	/// UFS Device Path.
	pub const MESSAGING_UFS: Self = Self(25,);
	/// Uniform Resource Identifiers (URI) Device Path.
	pub const MESSAGING_URI: Self = Self(24,);
	/// USB Device Path.
	pub const MESSAGING_USB: Self = Self(5,);
	/// USB Class Device Path.
	pub const MESSAGING_USB_CLASS: Self = Self(15,);
	/// USB WWID Device Path.
	pub const MESSAGING_USB_WWID: Self = Self(16,);
	/// Vendor-Defined Device Path.
	pub const MESSAGING_VENDOR: Self = Self(10,);
	/// VLAN Device Path node.
	pub const MESSAGING_VLAN: Self = Self(20,);
	/// Wi-Fi Device Path.
	pub const MESSAGING_WIFI: Self = Self(28,);
}
