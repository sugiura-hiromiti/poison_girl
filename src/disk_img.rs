//! TODO: CLI依存を無くす
//!       hadris-fatクレートで置きかえる
use {
	crate::{Xtask, sudo},
	poison_girl_dev_cargo::Arch,
	poison_girl_dev_cli::Run,
	poison_girl_dev_error::{NotObedientPath, PoisonGirlB, X, poison_girl_err},
	poison_girl_dev_orchestrate::decl_manage::crate_::CrateInfo,
	std::{
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

/// RAII風のマウントポイント管理の責務を持つ
/// NOTE: Resource Acquisition Is Initialization
struct MountGuard
{
	mounted:     bool,
	mount_point: PathBuf,
}

impl MountGuard
{
	/// Directory path for EFI boot files from mounting point
	const BOOT_DIR: &str = "efi/boot";
	/// mounting point path under target/
	const MOUNT_DIR: &str = "mnt";

	fn new(asset_dir: impl AsRef<Path,>,) -> PoisonGirlB<Self,>
	{
		let mount_point = asset_dir.as_ref().join(Self::MOUNT_DIR,);
		std::fs::create_dir_all(&mount_point,)?;
		X(Self { mounted: false, mount_point, },)
	}

	pub fn mount_disk_img(
		&mut self,
		disk_img_path: impl AsRef<Path,>,
	) -> PoisonGirlB<(),>
	{
		sudo()
		// - mountは基本的にblock deviceをマウントする
		// - ディスクイメージはカーネルから見るとただのファイルなので
		//   仮想的なブロックデバイスと対応付ける事でブロックデバイスとして扱う事が出来る
	   // - その仮想的なブロックデバイスの事をloop deviceと呼ぶ
			.args(["mount", "-o", "loop",],)
			.arg(disk_img_path.as_ref(),)
			.arg(&self.mount_point,)
			.run()?;
		self.mounted = true;
		X((),)
	}

	/// uefiではboot_loaderへのデフォルトパスが決まっており、
	/// boot loaderの名称はarchitecture毎に決まっている
	/// boot loaderの名前解決は`MountGuard`の責務外なので外部から注入する
	/// NOTE: https://uefi.org/specs/UEFI/2.10/03_Boot_Manager.html#uefi-image-types
	pub fn copy_boot_loader(
		&self,
		boot_loader: impl AsRef<Path,>,
		boot_file_name: &str,
	) -> PoisonGirlB<(),>
	{
		let boot_dir = self.ensure_boot_dir()?;
		let boot_loader = boot_loader.as_ref();
		std::fs::copy(boot_loader, boot_dir.join(boot_file_name,),)?;
		X((),)
	}

	fn ensure_boot_dir(&self,) -> PoisonGirlB<PathBuf,>
	{
		let boot_dir = self.mount_point.join(Self::BOOT_DIR,);
		std::fs::create_dir_all(&boot_dir,)?;
		X(boot_dir,)
	}
}

impl Xtask
{
	/// 起動用のディスクイメージをセットアップしpathを返す
	pub(crate) fn build_boot_disk_img(&self,) -> PoisonGirlB<PathBuf,>
	{
		let asset_dir = self.asset_dir()?;
		// path.push(DISK_IMG_NAME,);
		let disk_img_path = asset_dir.join(DISK_IMG_NAME,);

		create_disk_img(&disk_img_path,)?;
		fmt_as_fat(&disk_img_path,)?;

		// マウント作業開始
		let mut mount = MountGuard::new(asset_dir,)?;
		mount.mount_disk_img(&disk_img_path,)?;
		X(disk_img_path,)
	}

	fn asset_dir(&self,) -> PoisonGirlB<PathBuf,>
	{
		let path = self.ws.path().join("target",).join(XTASK_ASSETS_DIR,);
		std::fs::create_dir_all(&path,)?;
		X(path,)
	}
}

fn create_disk_img(file_path: impl AsRef<Path,>,) -> PoisonGirlB<(),>
{
	// NOTE: qemu-img create
	// でディスクイメージを生成する際、
	// 既存のディスクイメージが既に存在する場合は上書きする為、
	// 上書きしたくない場合は注意
	Command::new("qemu-img",)
		.arg("create",)
		.args(DISK_IMG_FMT,)
		.arg(file_path.as_ref(),)
		.arg(DISK_IMG_SIZE,)
		.run()
}

fn fmt_as_fat(file_path: impl AsRef<Path,>,) -> PoisonGirlB<(),>
{
	Command::new("mkfs.fat",)
		.args(
			MKFS_FAT_OPT_N
				.into_iter()
				.chain(MKFS_FAT_OPT_S,)
				.chain(MKFS_FAT_OPT_F,)
				.chain(MKFS_FAT_OPT_CAP_R,)
				.chain(MKFS_FAT_OPT_CAP_F,),
		)
		.arg(file_path.as_ref(),)
		.run()
}
