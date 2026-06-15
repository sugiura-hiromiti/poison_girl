use {
	crate::Xtask,
	hadris_fat::{
		FatDir, FatFs, FatFsWriteExt, FileEntry,
		format::{FatTypeSelection, FatVolumeFormatter, FormatOptions},
	},
	poison_girl_dev_error::{PoisonGirlB, X},
	poison_girl_dev_orchestrate::decl_manage::{
		OrchestrationResolver, PoisonGirlCargoInterface,
		crate_::{CrateInfo, PoisonGirlCrateChart},
	},
	std::{fs::File, io::Read, path::PathBuf},
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
}

impl DiskImageBuilder
{
	/// boot loaderが置かれる(fat内の)パス
	const BOOT_DIR: &str = "efi/boot";
	/// 200MiB
	const DISK_IMG_SIZE: u64 = 200 * 1024 * 1024;
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
	/// クラスタを大きくすると:
	/// - fatテーブルが小さくなる
	/// - 大きいファイル中心の時に効率が良い
	///
	/// クラスタを小さくすると:
	/// - fatテーブルが大きくなる
	/// - 小さいファイルの無駄が減る
	/// - fat種別との兼ね合いで作成できない場合がある
	///
	/// NOTE: 指定しない場合は適切な値が自動選択される
	const SECTORS_PER_CLUSTER: u8 = 2;
	/// ラベルは最大11文字
	const VOLUME_LABEL: &str = check_volume_label("POISON GIRL",);

	pub fn new(
		disk_img: impl Into<PathBuf,>,
		boot_loader: impl Into<PathBuf,>,
		boot_loader_file_name: impl Into<String,>,
	) -> Self
	{
		Self {
			disk_img:              disk_img.into(),
			boot_loader:           boot_loader.into(),
			boot_loader_file_name: boot_loader_file_name.into(),
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
		self.place_boot_loader(disk_img_file,)?;
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
		disk_img_file.set_len(Self::DISK_IMG_SIZE,)?;

		X(disk_img_file,)
	}

	fn place_boot_loader(&self, disk_img_file: File,) -> PoisonGirlB<(),>
	{
		let options = FormatOptions::new(Self::DISK_IMG_SIZE,)
			.with_fat_type(FatTypeSelection::Fat32,)
			.with_label(Self::VOLUME_LABEL,)
			.with_sectors_per_cluster(Self::SECTORS_PER_CLUSTER,)
			.with_fat_copies(Self::FAT_COPIES,);
		let fat_hndlr = FatVolumeFormatter::format(disk_img_file, options,)?;
		let fat_root = fat_hndlr.root_dir();

		let boot_loader_entry =
			self.ensure_boot_loader_entry(&fat_hndlr, fat_root,)?;
		self.write_boot_loader(&fat_hndlr, &boot_loader_entry,)
	}

	fn ensure_boot_loader_entry(
		&self,
		fat_hndlr: &FatFs<File,>,
		fat_root: FatDir<File,>,
	) -> PoisonGirlB<FileEntry,>
	{
		let boot_dir = Self::BOOT_DIR
			.split("/",)
			.try_fold(fat_root, |a, e| fat_hndlr.create_dir(&a, e,),)?;
		let boot_loader_entry =
			fat_hndlr.create_file(&boot_dir, &self.boot_loader_file_name,)?;
		X(boot_loader_entry,)
	}

	fn write_boot_loader(
		&self,
		fat_hndlr: &FatFs<File,>,
		boot_loader_entry: &FileEntry,
	) -> PoisonGirlB<(),>
	{
		let mut boot_loader_file =
			std::fs::OpenOptions::new().read(true,).open(&self.boot_loader,)?;
		// NOTE: consider Vec::with_capacity for performance optimization.
		let mut buf = vec![];
		boot_loader_file.read_to_end(&mut buf,)?;
		let mut boot_loader_writer = fat_hndlr.write_file(boot_loader_entry,)?;

		let size = buf.len();
		let mut written_size = 0;
		while written_size < size {
			let written = boot_loader_writer.write(&buf[written_size..],)?;
			written_size += written;
		}

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
			self.interface.task().clone(),
		);
		let boot_loader = boot_loader_crate.build_artifact()?.path();
		let boot_loader_file_name =
			self.opts().arch.boot_file_name().to_string();

		// TODO: copy kernel binary to disk image
		let disk_img_bldr = DiskImageBuilder::new(
			disk_img.clone(),
			boot_loader,
			boot_loader_file_name,
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
