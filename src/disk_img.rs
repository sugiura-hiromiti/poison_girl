use {
	crate::{Xtask, sudo},
	hadris_fat::{
		FatDir, FatFs, FatFsWriteExt, FileEntry,
		format::{FatTypeSelection, FatVolumeFormatter, FormatOptions},
	},
	poison_girl_dev_cargo::Arch,
	poison_girl_dev_cli::Run,
	poison_girl_dev_error::{NotObedientPath, PoisonGirlB, X, poison_girl_err},
	poison_girl_dev_orchestrate::decl_manage::crate_::CrateInfo,
	std::{
		ffi::OsString,
		fs::File,
		io::Read,
		path::{Path, PathBuf},
		process::Command,
	},
};

/// relative path to directory build assets are put from target/
const XTASK_ASSETS_DIR: &str = "xtask";

/// ディスクイメージのフォーマットをrawにする
/// qemu-imgコマンドのオプション
const DISK_IMG_FMT: [&str; 2] = ["-f", "raw",];
/// ディスクイメージのサイズ(200mb)
const DISK_IMG_SIZE: &str = "200M";
/// ディスクイメージのファイル名前
const DISK_IMG_NAME: &str = "disk.img";

// fatファイルシステムのメタ情報、配置、種類を指定するオプション

/// mkfs.fatの-nオプション
/// ボリュームラベル(名前)をつける
/// NOTE: ラベルは最大11文字
/// NOTE: `Command`はシェルを経由しないのでクオートや変数展開は機能しない
///       ラベルをshellの時のようにシングルクオートしていないのはその為
const MKFS_FAT_OPT_N: [&str; 2] = ["-n", "POISON GIRL",];
/// mkfs.fatの-sオプション
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
const MKFS_FAT_OPT_S: [&str; 2] = ["-s", "2",];
/// mkfs.fatの-fオプション
/// FATテーブルの個数を指定
/// ファイルのクラスタ連鎖情報を持つ領域がfile allocation table,
/// FATテーブルと呼ばれている 冗長性のため複数である事が多く、デフォルトは2
/// 1にする意味は、理屈上メタデータ領域を減らせる
/// ただしメリットが薄いので特殊な組み込み、
/// 極小イメージを求められない限り2で良い
const MKFS_FAT_OPT_F: [&str; 2] = ["-f", "2",];
/// mkfs.fatの-Rオプション
/// 予約セクタ数を指定
/// fatの内部配置をざっくり表すと↓
/// ```
/// [ reserved sectors ][ FAT area ][ root directory area ][ data area ]
/// ```
/// 仕様によると:
/// - fat32では最低2つの予約セクタが必要でデフォルトは32
/// - それ以外ではデフォルト1(boot sectorのみ)
///
/// NOTE: このオプションで指定する値は最小値でありアライメントの為に増える事がある
/// 主に以下のような低レベル用途
/// - FAT32のboot sector / FSInfo / backup boot sector配置を調整したい
/// - ブートローダやファームウェア都合のレイアウトを作りたい
/// - 既存イメージとの互換性を合わせたい
/// - テスト用に正確なFATレイアウトを作りたい
const MKFS_FAT_OPT_CAP_R: [&str; 2] = ["-R", "32",];
/// mkfs.fatの-Fオプション
/// FAT12/FAT16/FAT32から指定
/// FAT12: フロッピーなど極小メディア
/// FAT16: 古い互換性用途
/// FAT32: USBメモリ、EFI System Partition(ESP)、SDカード等
/// NOTE: かなり小さいパーティションにFAT32を強制するとクラスタ数や予約領域を確保し切れない事がある為、
/// ESPなら数百MiB確保して32bit指定するのが無難
const MKFS_FAT_OPT_CAP_F: [&str; 2] = ["-F", "32",];

struct DiskImageBuilder
{
	disk_img:              PathBuf,
	boot_loader:           PathBuf,
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
		disk_img_file.set_len(Self::DISK_IMG_SIZE as u64,)?;

		X(disk_img_file,)
	}

	fn place_boot_loader(&self, disk_img_file: File,) -> PoisonGirlB<(),>
	{
		let options = FormatOptions::new(Self::DISK_IMG_SIZE as u64,)
			.with_fat_type(FatTypeSelection::Fat32,)
			.with_label(Self::VOLUME_LABEL,)
			.with_sectors_per_cluster(Self::SECTORS_PER_CLUSTER,)
			.with_fat_copies(Self::FAT_COPIES,);
		let fat_hndlr = FatVolumeFormatter::format(disk_img_file, options,)?;
		let fat_root = fat_hndlr.root_dir();

		let boot_loader_entry =
			self.ensure_boot_loader_entry(&fat_hndlr, &fat_root,)?;
		self.write_boot_loader(&fat_hndlr, &boot_loader_entry,)
	}

	fn ensure_boot_loader_entry(
		&self,
		fat_hndlr: &FatFs<File,>,
		fat_root: &FatDir<File,>,
	) -> PoisonGirlB<FileEntry,>
	{
		let boot_dir = Self::BOOT_DIR
			.split("/",)
			.try_fold(fat_root, |a, e| fat_hndlr.create_dir(a, e,),)?;
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
		let mut boot_loader_writer = fat_hndlr.write_file(boot_loader_entry,)?;
		std::io::copy(&mut boot_loader_file, &mut boot_loader_writer,)?;
		X((),)
	}
}

/// TODO: 将来的にはascii/space padding/forbidden charsも検査する
const fn check_volume_label(label: &str,) -> &str
{
	if label.len() > 11 {
		panic!()
	}

	label
}

impl Xtask
{
	pub(crate) fn build_boot_disk_img(&self,) -> PoisonGirlB<(),>
	{
		let disk_img = self.disk_img_path()?;
		let boot_loader = todo!();
		let boot_loader_file_name = todo!();
		let disk_img_bldr = DiskImageBuilder::new(
			disk_img,
			boot_loader,
			boot_loader_file_name,
		);
	}

	/// 起動用のディスクイメージへのパスを返す
	/// NOTE: 存在確認やセットアップはこの関数の責務ではない
	pub(crate) fn disk_img_path(&self,) -> PoisonGirlB<PathBuf,>
	{
		X(self.asset_dir()?.join(DISK_IMG_NAME,),)
	}

	fn asset_dir(&self,) -> PoisonGirlB<PathBuf,>
	{
		let path = self.ws.path().join("target",).join(XTASK_ASSETS_DIR,);
		std::fs::create_dir_all(&path,)?;
		X(path,)
	}
}
