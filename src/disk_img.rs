use {
	crate::Xtask,
	poison_girl_dev_cli::Run,
	poison_girl_dev_error::{
		PathIsNotValidUtf8, PoisonGirlB, X, poison_girl_err,
	},
	poison_girl_dev_orchestrate::decl_manage::crate_::CrateInfo,
	std::{
		env::set_current_dir,
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
const MKFS_FAT_OPT_N: [&str; 2] = ["-n", "'POISON GIRL'",];
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

struct MountGuard
{
	mounted: bool,
}

impl MountGuard
{
	/// Directory path for EFI boot files from mounting point
	const BOOT_DIR: &str = "efi/boot";
	/// mounting point path under target/
	const MOUNT_DIR: &str = "mnt";

	fn new() -> Self
	{
		Self { mounted: false, }
	}

	fn make_mount_point(&self, asset_dir: &Path,) -> PoisonGirlB<PathBuf,>
	{
		let mut mnt_dir = asset_dir.to_path_buf();
		mnt_dir.push(Self::MOUNT_DIR,);
		let mnt_dir = mnt_dir;
		Command::new("mkdir",).arg("-p",).arg(&mnt_dir,).run()?;
		X(mnt_dir,)
	}
}

impl Xtask
{
	/// 起動用のディスクイメージをセットアップしpathを返す
	pub(crate) fn disk_img_path(&self,) -> PoisonGirlB<PathBuf,>
	{
		let mut path = self.asset_dir()?;
		path.push(DISK_IMG_NAME,);
		let file_path = path;

		create_disk_img(&file_path,)?;
		fmt_as_fat(&file_path,)?;
		X(file_path,)
	}

	fn asset_dir(&self,) -> PoisonGirlB<PathBuf,>
	{
		let mut path = self.ws.path();
		path.push("target",);
		path.push(XTASK_ASSETS_DIR,);

		if path.exists() {
			X(path,)
		} else {
			let path_to_create =
				path.to_str().ok_or(poison_girl_err!(PathIsNotValidUtf8),)?;
			Command::new("mkdir",).args(["-p", path_to_create,],).run()?;
			X(path,)
		}
	}
}

fn create_disk_img(file_path: &Path,) -> PoisonGirlB<(),>
{
	let args = ["create",].into_iter().chain(DISK_IMG_FMT,).chain([
		file_path.to_str().ok_or(poison_girl_err!(PathIsNotValidUtf8),)?,
		DISK_IMG_SIZE,
	],);
	// NOTE: qemu-img create
	// でディスクイメージを生成する際、
	// 既存のディスクイメージが既に存在する場合は上書きする為、
	// 上書きしたくない場合は注意
	Command::new("qemu-img",).args(args,).run()?;
	X((),)
}

fn fmt_as_fat(file_path: &Path,) -> PoisonGirlB<(),>
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
		.arg(file_path,)
		.run()
}
