use {
	super::{
		Handle, drop_uefi_cleanup_result, image_handle, table::boot_services,
	},
	crate::{
		guid,
		raw::{
			protocol::{
				device_path::DevicePathProtocol,
				file::SimpleFileSystemProtocol,
				graphic::GraphicsOutputProtocol, text::TextOutputProtocol,
			},
			service::BootServices,
			types::{
				Guid, UnsafeHandle,
				file::{FileInfo, FileSystemInfo, FileSystemVolumeLabel},
			},
		},
	},
	core::{
		ffi::c_void,
		ptr::{self, NonNull},
	},
	poison_girl_no_std_error::{PoisonGirlB, UefiError, X, Y, poison_girl_err},
};

pub trait Protocol
{
	const GUID: Guid;
}

impl Protocol for TextOutputProtocol
{
	const GUID: Guid = guid!("387477c2-69c7-11d2-8e39-00a0c969723b");
}

impl Protocol for DevicePathProtocol
{
	const GUID: Guid = guid!("09576e91-6d3f-11d2-8e39-00a0c969723b");
}

impl Protocol for SimpleFileSystemProtocol
{
	const GUID: Guid = guid!("964e5b22-6459-11d2-8e39-00a0c969723b");
}

impl Protocol for FileInfo
{
	const GUID: Guid = guid!("09576e92-6d3f-11d2-8e39-00a0c969723b");
}

impl Protocol for FileSystemInfo
{
	const GUID: Guid = guid!("09576e93-6d3f-11d2-8e39-00a0c969723b");
}

impl Protocol for FileSystemVolumeLabel
{
	const GUID: Guid = guid!("db47d7d3-fe81-11d3-9a35-0090273fC14d");
}

impl Protocol for GraphicsOutputProtocol
{
	const GUID: Guid = guid!("9042a9de-23dc-4a38-96fb-7aded080516a");
}

impl BootServices
{
	/// # Safety
	pub unsafe fn locate_handle_buffer(
		&self,
		ty: HandleSearchType,
	) -> PoisonGirlB<&mut [UnsafeHandle],>
	{
		let (ty, guid, key,) = match ty {
			HandleSearchType::AllHandles => (0, ptr::null(), ptr::null(),),
			HandleSearchType::ByRegisterNotify(protocol_search_key,) => {
				(1, ptr::null(), protocol_search_key.0.as_ptr().cast_const(),)
			},
			HandleSearchType::ByProtocol(guid,) => {
				(2, ptr::from_ref(guid,), ptr::null(),)
			},
		};

		let mut num_handles: usize = 0;
		let mut buffer: *mut UnsafeHandle = ptr::null_mut();
		unsafe {
			(self.locate_handle_buffer)(
				ty,
				guid,
				key,
				&mut num_handles,
				&mut buffer,
			)
		}
		.x_or()?;

		let handler_range =
			unsafe { core::slice::from_raw_parts_mut(buffer, num_handles,) };

		X(handler_range,)
	}

	/// return guid to search protocol
	pub fn protocol_for<P: Protocol,>(&'_ self,) -> HandleSearchType<'_,>
	{
		HandleSearchType::ByProtocol(&P::GUID,)
	}

	/// # Safety
	pub unsafe fn handles_for_protocol<P: Protocol,>(
		&self,
	) -> PoisonGirlB<&mut [UnsafeHandle],>
	{
		let search_ty = self.protocol_for::<P>();
		unsafe { self.locate_handle_buffer(search_ty,) }
	}

	/// # Safety
	pub unsafe fn handle_for_protocol<P: Protocol,>(
		&self,
	) -> PoisonGirlB<Handle,>
	{
		let handles = unsafe { self.handles_for_protocol::<P>() }?;
		let first_handle = *handles.first().ok_or(poison_girl_err!(
			UefiError::Custom("length of handles is 0")
		),)?;
		let hndl = unsafe { Handle::from_ptr(first_handle,) }
			.ok_or(poison_girl_err!(UefiError::Custom("handle is null")),)?;
		X(hndl,)
	}

	/// # Parms
	///
	/// ***handle***
	/// 開きたいプロトコルのインターフェースハンドラ
	///
	/// ***agent***
	/// プロトコルを開くためのエージェントのハンドル
	/// agentがUEFI Driver
	/// Modelに従っている場合この引数はEFI_DRIVER_BINDING_PROTOCOLのハンドラということになる
	/// EFI_DRIVER_BINDING_PROTOCOLのインスタンスはUEFIドライバによって生成される
	/// UEFIアプリケーションの場合、これはイメージハンドラにあたる
	///
	/// ***controller***
	/// agentがUEFI Driver Modelに従っている場合この引数はagentのハンドラとなる
	/// そうでない場合はこの引数はnullでも良い
	///
	/// ***attr***
	/// プロトコルをどの様に開くかを指定する
	/// 詳細は引数の型定義参照
	///
	/// # Safety
	pub unsafe fn open_protocol<P: Protocol,>(
		&self,
		necessity: OpenProtoNecessity,
		attr: OpenProtoAttr,
	) -> PoisonGirlB<ProtocolInterface<P,>,>
	{
		let mut interface = ptr::null_mut();
		unsafe {
			(self.open_protocol)(
				necessity.handle.as_ptr(),
				&P::GUID,
				&mut interface,
				necessity.agent.as_ptr(),
				Handle::opt_to_ptr(necessity.controller.clone(),),
				attr.0,
			)
			.x_or_with(|_| ProtocolInterface {
				interface: if interface.is_null() {
					None
				} else {
					Some(NonNull::new_unchecked(interface.cast(),),)
				},
				handles:   necessity,
			},)
		}
	}

	pub fn open_protocol_exclusive<P: Protocol,>(
		&self,
		handle: Handle,
	) -> PoisonGirlB<ProtocolInterface<P,>,>
	{
		let necessity = OpenProtoNecessity::for_app(handle,)?;
		unsafe { self.open_protocol(necessity, OpenProtoAttr::EXCULSIVE,) }
	}

	pub fn open_protocol_with<P: Protocol,>(
		&self,
	) -> PoisonGirlB<ProtocolInterface<P,>,>
	{
		let bs = boot_services()?;
		let handle = unsafe { bs.handle_for_protocol::<P>() }?;
		let necessity = OpenProtoNecessity::for_app(handle,)?;
		let attr = OpenProtoAttr::GET_PROTOCOL;

		unsafe { bs.open_protocol(necessity, attr,) }
	}

	pub fn handle_protocol<P: Protocol,>(
		&self,
		handle: Handle,
	) -> PoisonGirlB<NonNull<ProtocolInterface<P,>,>,>
	{
		let mut interface = ptr::null_mut();
		unsafe {
			(self.handle_protocol)(handle.as_ptr(), &P::GUID, &mut interface,)
				.x_or()?;
		}
		let interface = interface.cast::<ProtocolInterface<P,>>();
		match NonNull::new(interface,) {
			Some(interface,) => X(interface,),
			None => {
				Y(poison_girl_err!(UefiError::Custom("interface is null",)),)
			},
		}
	}
}

#[derive(Debug,)]
pub enum HandleSearchType<'g,>
{
	/// return all handles present on the system
	AllHandles,
	/// return all handles that implement a protocol when an intereface for that
	/// protocol is (re)installed
	ByRegisterNotify(ProtocolSearchKey,),
	/// returns all handles supporting a certain protocol, specified by its guid
	ByProtocol(&'g Guid,),
}

#[derive(Clone, Debug,)]
#[repr(transparent)]
pub struct ProtocolSearchKey(pub(crate) NonNull<c_void,>,);

#[repr(transparent)]
pub struct OpenProtoAttr(u32,);

impl OpenProtoAttr
{
	/// busドライバに使用される
	/// このフラグが立っている場合、再帰的にchild controllerに接続しようとする
	pub const BY_CHILD_CONTROLLER: Self = Self(0x8,);
	/// ドライバがプロトコルインターフェースのアクセスを得る為に使用される
	/// このフラグが立っている場合、プロトコルインターフェースが削除、
	/// 再インストールされる際にドライバが停止する
	/// 一度プロトコルインターフェースがドライバを用いて、
	/// そしてこのフラグをオンにして開かれた場合、
	/// 他のドライバはこのフラグを立てて同じプロトコルインターフェースを開くことが許可されない
	pub const BY_DRIVER: Self = Self(0x10,);
	/// boot_services.handle_protocolで使用される
	pub const BY_HANDLE_PROTOCOL: Self = Self(0x1,);
	/// UEFIアプリケーションがプロトコルインターフェースの排他的アクセスを得る際に使用される
	/// BY_DRIVERフラグでプロトコルインターフェースを開いているドライバがある場合、
	/// ドライバを停止する試みがなされる
	pub const EXCULSIVE: Self = Self(0x20,);
	pub const GET_PROTOCOL: Self = Self(0x2,);
	pub const TEST_PROTOCOL: Self = Self(0x4,);
}

/// protocol interface representation which is designed as safe(automatically
/// closed on drop)
pub struct ProtocolInterface<P: Protocol,>
{
	interface: Option<NonNull<P,>,>,
	handles:   OpenProtoNecessity,
}

impl<P: Protocol,> ProtocolInterface<P,>
{
	pub fn interface(&self,) -> PoisonGirlB<NonNull<P,>,>
	{
		match self.interface {
			Some(interface,) => X(interface,),
			None => Y(poison_girl_err!(UefiError::Custom(
				"protocol interface is null",
			)),),
		}
	}
}

impl<P: Protocol,> Drop for ProtocolInterface<P,>
{
	fn drop(&mut self,)
	{
		if let X(bt,) = boot_services() {
			let rslt = unsafe {
				(bt.close_protocol)(
					self.handles.handle_ptr(),
					&P::GUID,
					self.handles.agent_ptr(),
					self.handles.controller_ptr(),
				)
			}
			.x_or();
			drop_uefi_cleanup_result(rslt,);
		}
	}
}

pub struct OpenProtoNecessity
{
	handle:     Handle,
	agent:      Handle,
	controller: Option<Handle,>,
}

impl OpenProtoNecessity
{
	pub fn for_app(handle: Handle,) -> PoisonGirlB<Self,>
	{
		let agent = image_handle()?;
		X(Self { handle, agent, controller: None, },)
	}

	pub fn handle_ptr(&self,) -> UnsafeHandle
	{
		self.handle.as_ptr()
	}

	pub fn agent_ptr(&self,) -> UnsafeHandle
	{
		self.agent.as_ptr()
	}

	/// may null
	pub fn controller_ptr(&self,) -> UnsafeHandle
	{
		Handle::opt_to_ptr(self.controller.clone(),)
	}
}
