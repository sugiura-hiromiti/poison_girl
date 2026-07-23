use {
	crate::Xtask,
	hadris_fat::{
		FatDir, FatFs, FatFsWriteExt, FileEntry,
		format::{FatTypeSelection, FatVolumeFormatter, FormatOptions},
	},
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_orchestrate::{
		BuildArtifactPolicyResolver,
		decl_manage::{
			PoisonGirlCargoInterface,
			crate_::{CrateInfo, PoisonGirlCrateChart},
		},
	},
	poison_girl_no_std::KERNEL_FILE_NAME,
	std::{
		fs::File,
		io::Read,
		path::{Path, PathBuf},
	},
};

/// relative path to directory build assets are put from target/
const XTASK_ASSETS_DIR: &str = "xtask";

/// ディスクイメージのファイル名前
const DISK_IMG_NAME: &str = "disk.img";

struct DiskImageBuilder
{
	/// path to disk_img **file**. not directory.
	disk_img:              PathBuf,
	/// path to boot loader **file**. not directory.
	boot_loader:           PathBuf,
	/// name of boot loader in **disk image**.
	/// Keep in mind that this is not the name of build artifact of loader
	/// crate.
	boot_loader_file_name: String,
	/// path to kernel file. not directory
	kernel:                PathBuf,
	/// name of kernel file in **disk image**
	kernel_file_name:      String,
	options:               DiskImageOptions,
}

#[derive(Clone, Copy,)]
struct DiskImageOptions
{
	size_bytes:          u64,
	fat_type:            FatTypeSelection,
	fat_copies:          u8,
	sectors_per_cluster: Option<u8,>,
	volume_label:        &'static str,
}

impl DiskImageOptions
{
	/// 200MiB
	const BOOT_IMAGE_SIZE: u64 = 200 * 1024 * 1024;
	/// FATテーブルの個数を指定
	/// ファイルのクラスタ連鎖情報を持つ領域がfile allocation table,
	/// FATテーブルと呼ばれている 冗長性のため複数である事が多く、デフォルトは2
	/// 1にする意味は、理屈上メタデータ領域を減らせる
	/// ただしメリットが薄いので特殊な組み込み、
	/// 極小イメージを求められない限り2で良い
	const FAT_COPIES: u8 = 2;
	/// 1クラスタあたりのセクタ数を指定
	/// 値は2の羃である必要がある
	/// クラスタサイズ = 論理セクタサイズ * クラスタ毎のセクタ数
	/// なので、セクタ数を増やせばクラスタのサイズが大きくなる
	/// NOTE: 指定しない場合は適切な値が自動選択される
	const SECTORS_PER_CLUSTER: u8 = 2;
	/// ラベルは最大11文字
	const VOLUME_LABEL: &str = check_volume_label("POISON GIRL",);

	fn default_boot() -> Self
	{
		Self {
			size_bytes:          Self::BOOT_IMAGE_SIZE,
			fat_type:            FatTypeSelection::Fat32,
			fat_copies:          Self::FAT_COPIES,
			sectors_per_cluster: Some(Self::SECTORS_PER_CLUSTER,),
			volume_label:        Self::VOLUME_LABEL,
		}
	}

	#[cfg(test)]
	fn small_test_image() -> Self
	{
		Self {
			size_bytes:          4 * 1024 * 1024,
			fat_type:            FatTypeSelection::Fat16,
			fat_copies:          Self::FAT_COPIES,
			sectors_per_cluster: Some(1,),
			volume_label:        Self::VOLUME_LABEL,
		}
	}

	fn format_options(&self,) -> FormatOptions
	{
		let options = FormatOptions::new(self.size_bytes,)
			.with_fat_type(self.fat_type,)
			.with_label(self.volume_label,)
			.with_fat_copies(self.fat_copies,);

		if let Some(sectors_per_cluster,) = self.sectors_per_cluster {
			options.with_sectors_per_cluster(sectors_per_cluster,)
		} else {
			options
		}
	}
}

impl DiskImageBuilder
{
	/// boot loaderが置かれる(fat内の)パス
	const BOOT_DIR: &str = "efi/boot";
	/// kernelが置かれる(fat内の)パス
	const KERNEL_DIR: &str = "";

	pub fn new(
		disk_img: impl Into<PathBuf,>,
		boot_loader: impl Into<PathBuf,>,
		boot_loader_file_name: impl Into<String,>,
		kernel: impl Into<PathBuf,>,
		kernel_file_name: impl Into<String,>,
	) -> Self
	{
		Self::with_options(
			disk_img,
			boot_loader,
			boot_loader_file_name,
			kernel,
			kernel_file_name,
			DiskImageOptions::default_boot(),
		)
	}

	fn with_options(
		disk_img: impl Into<PathBuf,>,
		boot_loader: impl Into<PathBuf,>,
		boot_loader_file_name: impl Into<String,>,
		kernel: impl Into<PathBuf,>,
		kernel_file_name: impl Into<String,>,
		options: DiskImageOptions,
	) -> Self
	{
		Self {
			disk_img: disk_img.into(),
			boot_loader: boot_loader.into(),
			boot_loader_file_name: boot_loader_file_name.into(),
			kernel: kernel.into(),
			kernel_file_name: kernel_file_name.into(),
			options,
		}
	}

	/// ```
	/// qemu-img create -f raw <disk image> 200M
	/// mkfs.fat -n 'POISON GIRL' -s 2 -f 2 -F 32 <disk image>
	/// mkdir -p mnt
	/// sudo mount -o loop <disk image> mnt
	/// sudo mkdir -p mnt/efi/boot
	/// sudo cp <boot loader> mnt/efi/boot/<boot loader>
	/// sudo umount mnt
	/// ```
	/// と同等の処理をする
	pub fn build_boot_disk_img(&self,) -> PoisonGirlB<(),>
	{
		let disk_img_file = self.create_disk_img_file()?;
		let fat_hndlr = self.get_hndlr(disk_img_file,)?;

		self.place_boot_loader(&fat_hndlr,)?;
		self.place_kernel(&fat_hndlr,)?;
		X((),)
	}

	fn create_disk_img_file(&self,) -> PoisonGirlB<std::fs::File,>
	{
		let disk_img_file = std::fs::OpenOptions::new()
			.write(true,)
			.read(true,)
			.create(true,)
			.truncate(true,)
			.open(&self.disk_img,)?;
		disk_img_file.set_len(self.options.size_bytes,)?;

		X(disk_img_file,)
	}

	fn get_hndlr(&self, disk_img_file: File,) -> PoisonGirlB<FatFs<File,>,>
	{
		let options = self.options.format_options();
		let fat_hndlr = FatVolumeFormatter::format(disk_img_file, options,)?;
		X(fat_hndlr,)
	}

	fn place_boot_loader(&self, fat_hndlr: &FatFs<File,>,) -> PoisonGirlB<(),>
	{
		let boot_loader_entry = self.ensure_boot_loader_entry(fat_hndlr,)?;
		self.write_boot_loader(fat_hndlr, &boot_loader_entry,)
	}

	fn ensure_boot_loader_entry(
		&self,
		fat_hndlr: &FatFs<File,>,
	) -> PoisonGirlB<FileEntry,>
	{
		let fat_root = fat_hndlr.root_dir();
		self.ensure_entry(
			fat_hndlr,
			fat_root,
			Self::BOOT_DIR,
			self.boot_loader_file_name,
		)
	}

	fn write_boot_loader(
		&self,
		fat_hndlr: &FatFs<File,>,
		boot_loader_entry: &FileEntry,
	) -> PoisonGirlB<(),>
	{
		self.write_to_entry(fat_hndlr, boot_loader_entry, &self.boot_loader,)
	}

	fn place_kernel(&self, fat_hndlr: &FatFs<File,>,) -> PoisonGirlB<(),>
	{
		let kernel_entry = self.ensure_kernel_entry(&fat_hndlr,)?;
		self.write_kernel(&fat_hndlr, kernel_entry,)
	}

	fn ensure_kernel_entry(
		&self,
		fat_hndlr: &FatFs<File,>,
	) -> PoisonGirlB<FileEntry,>
	{
		let fat_root = fat_hndlr.root_dir();
		self.ensure_entry(
			fat_hndlr,
			fat_root,
			Self::KERNEL_DIR,
			self.kernel_file_name,
		)
	}

	fn write_kernel(
		&self,
		fat_hndlr: &FatFs<File,>,
		kernel_entry: &FileEntry,
	) -> PoisonGirlB<(),>
	{
		self.write_to_entry(fat_hndlr, entry, &self.kernel,)
	}

	fn ensure_entry(
		&self,
		fat_hndlr: &FatFs<File,>,
		fat_root: FatDir<File,>,
		entry_path: impl AsRef<str,>,
		file_name: impl AsRef<str,>,
	) -> PoisonGirlB<FileEntry,>
	{
		let boot_dir = entry_path
			.as_ref()
			.split("/",)
			.try_fold(fat_root, |a, e| fat_hndlr.create_dir(&a, e,),)?;
		let entry = fat_hndlr.create_file(&boot_dir, file_name.as_ref(),)?;
		X(entry,)
	}

	fn write_to_entry(
		&self,
		fat_hndlr: &FatFs<File,>,
		entry: &FileEntry,
		file: impl AsRef<Path,>,
	) -> PoisonGirlB<(),>
	{
		let mut file = std::fs::OpenOptions::new().read(true,).open(file,)?;
		// NOTE: consider Vec::with_capacity for performance optimization.
		let mut buf = vec![];
		file.read_to_end(&mut buf,)?;
		let mut writer = fat_hndlr.write_file(entry,)?;

		let size = buf.len();
		let mut written_size = 0;
		while written_size < size {
			let written = writer.write(&buf[written_size..],)?;
			written_size += written;
		}
		writer.finish()?;

		X((),)
	}
}

/// TODO: 将来的にはascii/space padding/forbidden charsも検査する
const fn check_volume_label(label: &str,) -> &str
{
	if label.len() > 11 {
		return "POISON GIRL";
	}

	label
}

impl Xtask
{
	/// NOTE: この関数は副作用を持ちます。一度だけ呼ばれることを保証してください
	/// またこの挙動が修正されるべきかは考え中です
	pub(crate) fn build_boot_disk_img(&self,) -> PoisonGirlB<PathBuf,>
	{
		let disk_img = self.disk_img_path()?;
		let boot_loader_crate = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::LOADER,
			self.interface.policy().clone(),
		);
		let boot_loader = boot_loader_crate.build_artifact_policy()?.path();
		// file name in disk image
		let boot_loader_file_name =
			self.opts().arch().boot_file_name().to_string();

		let kernel_crate = PoisonGirlCargoInterface::new(
			PoisonGirlCrateChart::KERNEL,
			self.interface.policy().clone(),
		);
		let kernel = kernel_crate.build_artifact_policy()?.path();

		// TODO: copy kernel binary to disk image
		let disk_img_bldr = DiskImageBuilder::new(
			disk_img.clone(),
			boot_loader,
			boot_loader_file_name,
			kernel,
			KERNEL_FILE_NAME,
		);
		disk_img_bldr.build_boot_disk_img()?;
		X(disk_img,)
	}

	/// 起動用のディスクイメージへのパスを返す
	/// NOTE: 存在確認やセットアップはこの関数の責務ではない
	pub(crate) fn disk_img_path(&self,) -> PoisonGirlB<PathBuf,>
	{
		X(self.asset_dir()?.join(DISK_IMG_NAME,),)
	}

	fn asset_dir(&self,) -> PoisonGirlB<PathBuf,>
	{
		let path = self.ws().path().join("target",).join(XTASK_ASSETS_DIR,);
		std::fs::create_dir_all(&path,)?;
		X(path,)
	}
}

#[cfg(test)]
mod tests
{
	use {
		super::*,
		hadris_fat::FatFs,
		poison_girl_dev_test::{PoisonGirlTestB, success},
	};

	#[test]
	fn disk_img_builder_creates_bootable_fat_image() -> PoisonGirlTestB
	{
		let tmp = tempfile::tempdir()?;
		let disk_img = tmp.path().join("disk.img",);
		let boot_loader = tmp.path().join("loader.efi",);
		let boot_loader_bytes = b"fake uefi loader";
		std::fs::write(&boot_loader, boot_loader_bytes,)?;
		let options = DiskImageOptions::small_test_image();

		DiskImageBuilder::with_options(
			&disk_img,
			&boot_loader,
			"BOOTAA64.EFI",
			options,
		)
		.build_boot_disk_img()?;

		assert_eq!(std::fs::metadata(&disk_img,)?.len(), options.size_bytes);

		let disk_img_file = std::fs::File::open(&disk_img,)?;
		let fat = FatFs::open(disk_img_file,)?;
		let mut boot_loader_reader =
			fat.open_file_path("efi/boot/BOOTAA64.EFI",)?;
		let actual = boot_loader_reader.read_to_vec()?;

		assert_eq!(actual, boot_loader_bytes);
		success!()
	}
}
