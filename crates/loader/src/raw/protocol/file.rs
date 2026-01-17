use core::ffi::c_void;

use crate::Status;
use crate::raw::types::Char16;
use crate::raw::types::Guid;
use crate::raw::types::file::FileAttributes;
use crate::raw::types::file::FileIoToken;
use crate::raw::types::file::OpenMode;

#[repr(C)]
pub struct SimpleFileSystemProtocol {
	pub revision:    u64,
	pub open_volume: unsafe extern "efiapi" fn(
		this: *mut Self,
		root: *mut *mut FileProtocolV1,
	) -> Status,
}

/**
---
Opens a new file relative to the source directory’s location.

# Description

The Open()function opens the file or directory referred to by FileName
relative to the location of This and returns a NewHandle.
The FileName may include the following path modifiers:

“\”

If the filename starts with a “\” the relative location is the root directory
that This resides on; otherwise “" separates name components.
Each name component is opened in turn,
and the handle to the last file opened is returned.

“.”

Opens the current location.

“..”

Opens the parent directory for the current location.
If the location is the root directory the request will return an error,
as there is no parent directory for the root directory.

If EFI_FILE_MODE_CREATE is set, then the file is created in the directory.
If the final location of FileName does not refer to a directory,
then the operation fails.
If the file does not exist in the directory, then a new file is created.
If the file already exists in the directory, then the existing file is opened.

If the medium of the device changes,
all accesses (including the File handle) will result in EFI_MEDIA_CHANGED.
To access the new medium, the volume must be reopened.

# Params

***NewHandlee***
A pointer to the location to return the opened handle for the new file.
See the type EFI_FILE_PROTOCOL description.

***FileName***
The Null-terminated string of the name of the file to be opened.
The file name may contain the following path modifiers: “", “.”, and “..”.

***OpenMode***
The mode to open the file.
The only valid combinations that the file may be opened with are:
Read, Read/Write, or Create/Read/Write.
See “Related Definitions” below.

***Attributes***
Only valid for EFI_FILE_MODE_CREATE,
in which case these are the attribute bits for the newly created file.
See “Related Definitions” below.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The file was opened|
|EFI_NOT_FOUND |The specified file could not be found on the device|
|EFI_NO_MEDIA |The device has no medium|
|EFI_MEDIA_CHANGED |The device has a different medium in it or the medium is no longer supported|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_WRITE_PROTECTED |An attempt was made to create a file,|
| |or open a file for write when the media is write-protected|
|EFI_ACCESS_DENIED |The service denied access to the file|
|EFI_OUT_OF_RESOURCES |Not enough resources were available to open the file|
|EFI_VOLUME_FULL |The volume is full|
|EFI_INVALID_PARAMETER |This refers to a regular file, not a directory|

*/
type FileOpen = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV1,
	new_handle: *mut *mut FileProtocolV1,
	file_name: *const Char16,
	open_mode: OpenMode,
	attr: FileAttributes,
) -> Status;

/**
---
Closes a specified file handle.

# Description

The Close() function closes a specified file handle.
All “dirty” cached file data is flushed to the device,
and the file is closed. In all cases the handle is closed.
The operation will wait for all pending asynchronous I/O requests to complete before completing.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS|The file was closed|

*/
type FileClose =
	unsafe extern "efiapi" fn(this: *mut FileProtocolV1,) -> Status;

/**
---
Closes and deletes a file.

# Description

The Delete() function closes and deletes a file. In all cases the file handle is closed. If the file cannot be deleted, the warning code EFI_WARN_DELETE_FAILURE is returned, but the handle is still closed.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The file was closed and deleted, and the handle was closed|
|EFI_WARN_DELETE_FAILURE |The handle was closed, but the file was not deleted|

*/
type FileDelete =
	unsafe extern "efiapi" fn(this: *mut FileProtocolV1,) -> Status;

/**
---
The Read() function reads data from a file.

# Description

If This is not a directory,
the function reads the requested number of bytes from the file
at the file’s current position and returns them in Buffer.
If the read goes beyond the end of the file,
the read length is truncated to the end of the file.
The file’s current position is increased by the number of bytes returned.

If This is a directory,
the function reads the directory entry at the
file’s current position and returns the entry in Buffer.
If the Buffer is not large enough to hold the current directory entry,
then EFI_BUFFER_TOO_SMALL is returned and the current file position is not updated.
BufferSize is set to be the size of the buffer needed to read the entry. On success,
the current position is updated to the next directory entry.
If there are no more directory entries, the read returns a zero-length buffer.
EFI_FILE_INFO is the structure returned as the directory entry.

# Params

***BufferSize***
On input, the size of the Buffer. On output, the amount of data returned in Buffer.
In both cases, the size is measured in bytes.

***Buffer***
The buffer into which the data is read.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The data was read|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_DEVICE_ERROR |An attempt was made to read from a deleted file|
|EFI_DEVICE_ERROR |On entry, the current file position is beyond the end of the file|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_BUFFER_TOO_SMALL |The BufferSize is too small to read the current directory entry|
| |BufferSize has been updated with the size needed to complete the request|

*/
type FileRead = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV1,
	buf_size: *mut usize,
	buf: *mut c_void,
) -> Status;

/**
---
Writes data to a file.

# Description

The Write() function writes the specified number of bytes to the file at the current file position.
The current file position is advanced the actual number of bytes written,
which is returned in BufferSize.
Partial writes only occur when there has been a data error
during the write attempt (such as “file space full”).
The file is automatically grown to hold the data if required.

Direct writes to opened directories are not supported.

# Params

***BufferSize***
On input, the size of the Buffer.
On output, the amount of data actually written.
In both cases, the size is measured in bytes.

***Buffer***
The buffer of data to write.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The data was written|
|EFI_UNSUPPORT |Writes to open directory files are not supported|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_DEVICE_ERROR |An attempt was made to write to a deleted file|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_WRITE_PROTECTED |The file or medium is write-protected|
|EFI_ACCESS_DENIED |The file was opened read only|
|EFI_VOLUME_FULL |The volume is full|

*/
type FileWrite = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV1,
	buf_size: *mut usize,
	buf: *mut c_void,
) -> Status;

/**
---
Returns a file’s current position.

# Description

The GetPosition() function returns the current file position for the file handle.
For directories, the current file position has no meaning outside of the
file system driver and as such the operation is not supported.
An error is returned if This is a directory.

# Params

***Position***
The address to return the file’s current position value.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The position was returned|
|EFI_UNSUPPORTED |The request is not valid on open directories|
|EFI_DEVICE_ERROR |An attempt was made to get the position from a deleted file|

*/
type FileGetPosition = unsafe extern "efiapi" fn(
	this: *const FileProtocolV1,
	position: *mut u64,
) -> Status;

/**
---
Sets a file’s current position.

# Description

The SetPosition() function sets the current file position
for the handle to the position supplied.
With the exception of seeking to position 0xFFFFFFFFFFFFFFFF,
only absolute positioning is supported,
and seeking past the end of the file is allowed (a subsequent write would grow the file).
Seeking to position 0xFFFFFFFFFFFFFFFF
causes the current position to be set to the end of the file.

If This is a directory, the only position that may be set is zero.
This has the effect of starting the read process of the directory entries over.

# Params

***Position***
The byte position from the start of the file to set.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The position was set|
|EFI_UNSUPPORTED |The seek request for nonzero is not valid on open directories|
|EFI_DEVICE_ERROR |An attempt was made to set the position of a deleted file|

*/
type FileSetPosition = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV1,
	position: u64,
) -> Status;

/**
---
Returns information about a file.

# Description

The GetInfo() function returns information of type InformationType for the requested file.
If the file does not support the requested information type,
then EFI_UNSUPPORTED is returned. If the buffer is not large enough to fit the requested structure,
EFI_BUFFER_TOO_SMALL is returned and the BufferSize is set to the
size of buffer that is required to make the request.

The information types defined by this specification are required information types
that all file systems must support.

# Params

***InformationType***
The type identifier for the information being requested.
Type EFI_GUID is defined on page 181.
See the EFI_FILE_INFO and EFI_FILE_SYSTEM_INFO descriptions for the related GUID definitions.

***BufferSize***
On input, the size of Buffer. On output, the amount of data returned in Buffer.
In both cases, the size is measured in bytes.

***Buffer***
A pointer to the data buffer to return. The buffer’s type is indicated by InformationType.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The information was set|
|EFI_UNSUPPORTED |The InformationType is not known|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_BUFFER_TOO_SMALL |The BufferSize is too small to read the current directory entry|
| |BufferSize has been updated with the size needed to complete the request|

*/
type FileGetInfo = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV1,
	info_type: *const Guid,
	buf_size: *mut usize,
	buf: *mut c_void,
) -> Status;

/**
---
Sets information about a file.

# Description

The SetInfo() function sets information of type InformationType on the requested file.
Because a read-only file can be opened only in read-only mode,
an InformationType of EFI_FILE_INFO_ID can be used with a read-only file
because this method is the only one that can be used to convert a read-only file to a read-write file.
In this circumstance, only the Attribute field of the EFI_FILE_INFO structure may be modified.
One or more calls to SetInfo() to change the Attribute field are permitted before it is closed.
The file attributes will be valid the next time the file is opened with Open().

An InformationType of EFI_FILE_SYSTEM_INFO_ID or EFI_FILE_SYSTEM_VOLUME_LABEL_ID
may not be used on read-only media.

# Params

***InformationType***
The type identifier for the information being set.
Type EFI_GUID is defined in page 181.
See the EFI_FILE_INFO and EFI_FILE_SYSTEM_INFO descriptions in this section for the related GUID definitions.

***BufferSize***
The size, in bytes, of Buffer

***Buffer***
A pointer to the data buffer to write. The buffer’s type is indicated by InformationType.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The information was set|
|EFI_UNSUPPORTED |The InformationType is not known|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_WRITE_PROTECTED |InformationType is EFI_FILE_INFO_ID and the media is read-only|
|EFI_WRITE_PROTECTED |InformationType is EFI_FILE_PROTOCOL_SYSTEM_INFO_ID and the media is read only|
|EFI_WRITE_PROTECTED |InformationType is EFI_FILE_SYSTEM_VOLUME_LABEL_ID and the media is read-only|
|EFI_ACCESS_DENIED |An attempt is made to change the name of a file to a file that is already present|
|EFI_ACCESS_DENIED |An attempt is being made to change the EFI_FILE_DIRECTORY Attribute|
|EFI_ACCESS_DENIED |An attempt is being made to change the size of a directory|
|EFI_ACCESS_DENIED |InformationType is EFI_FILE_INFO_ID and the file was opened read-only and|
| |an attempt is being made to modify a field other than Attribute|
|EFI_VOLUME_FULL |The volume is full|
|EFI_BAD_BUFFER_SIZE |BufferSize is smaller than the size of the type indicated by InformationType|

*/
type FileSetInfo = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV1,
	info_type: *const Guid,
	buf_size: usize,
	buf: *mut c_void,
) -> Status;

/**
---
Flushes all modified data associated with a file to a device.

# Description

The Flush() function flushes all modified data associated with a file to a device.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |The data was flushed|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_WRITE_PROTECTED |The file or medium is write-protected|
|EFI_ACCESS_DENIED |The file was opened read-only|
|EFI_VOLUME_FULL |The volume is full|

*/
type FileFlush =
	unsafe extern "efiapi" fn(this: *mut FileProtocolV1,) -> Status;

/**
---
Opens a new file relative to the source directory’s location.

# Description

The OpenEx() function opens the file or directory referred to by FileName
relative to the location of This and returns a NewHandle.
The FileName may include the path modifiers described previously in Open().

If EFI_FILE_MODE_CREATE is set, then the file is created in the directory.
If the final location of FileName does not refer to a directory, then the operation fails.
If the file does not exist in the directory, then a new file is created.
If the file already exists in the directory, then the existing file is opened.

If the medium of the device changes,
all accesses (including the File handle) will result in EFI_MEDIA_CHANGED.
To access the new medium, the volume must be reopened.

If an error is returned from the call to OpenEx() and non-blocking I/O is being requested,
the Event associated with this request will not be signaled.
If the call to OpenEx() succeeds then the Event will be signaled upon completion of the open
or if an error occurs during the processing of the request.
The status of the read request can be determined from the Status field
of the Token once the event is signaled.

# Params

***NewHandle***
A pointer to the location to return the opened handle for the new file.
See the type EFI_FILE_PROTOCOL description. For asynchronous I/O,
this pointer must remain valid for the duration of the asynchronous operation.

***FileName***
The Null-terminated string of the name of the file to be opened.
The file name may contain the following path modifiers: “", “.”, and “..”.

***OpenMode***
The mode to open the file. The only valid combinations that the file may be opened with are: Read,
Read/Write, or Create/Read/Write. See “Related Definitions” below.

***Attributes***
Only valid for EFI_FILE_MODE_CREATE, in which case these are the attribute bits for the

***Token***
A pointer to the token associated with the transaction.
Type EFI_FILE_IO_TOKEN is defined in “Related Definitions” below.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |Returned from the call OpenEx()|
| |If Event is NULL (blocking I/O): The file was opened successfully|
| |If Event is not NULL (asynchronous I/O):|
| |The request was successfully queued for processing|
| |Event will be signaled upon completion Returned in the token after signaling Event|
| |The file was opened successfully|
|EFI_NOT_FOUND |The specified file could not be found on the device|
|EFI_NO_MEDIA |The device has no medium|
|EFI_VOLUME_CORRUPTED | The file system structures are corrupted|
|EFI_WRITE_PROTECTED |An attempt was made to create a file,|
| |or open a file for write when the media is write-protected|
|EFI_ACCESS_DENIED |The service denied access to the file|
|EFI_OUT_OF_RESOURCES |Unable to queue the request or open the file due to lack of resources|
|EFI_VOLUME_FULL |The volume is full|
|EFI_INVALID_PARAMETER |This refers to a regular file, not a directory|

*/
type FileOpenEx = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV2,
	new_handle: *mut *mut FileProtocolV2,
	file_name: *const Char16,
	open_mode: OpenMode,
	attrs: FileAttributes,
	token: *mut FileIoToken,
) -> Status;

/**
---
Reads data from a file.

# Description

The ReadEx() function reads data from a file.

If This is not a directory,
the function reads the requested number of bytes from the file
at the file’s current position and returns them in Buffer.
If the read goes beyond the end of the file, the read length is truncated to the end of the file.
The file’s current position is increased by the number of bytes returned.

If This is a directory,
the function reads the directory entry at the file’s current position and returns the entry in* Buffer.
If the Buffer is not large enough to hold the current directory entry,
then EFI_BUFFER_TOO_SMALL is returned and the current file position is not updated.
BufferSize is set to be the size of the buffer needed to read the entry.
On success, the current position is updated to the next directory entry.
If there are no more directory entries, the read returns a zero-length buffer.
EFI_FILE_INFO is the structure returned as the directory entry.

If non-blocking I/O is used the file pointer will be advanced based on the order
that read requests were submitted.

If an error is returned from the call to ReadEx() and non-blocking I/O is being requested,
the Event associated with this request will not be signaled.
If the call to ReadEx() succeeds then the Event will be signaled upon completion
of the read or if an error occurs during the processing of the request.
The status of the read request can be determined from the Status field of the Token once the event is signaled.

# Params

- Token

A pointer to the token associated with the transaction.
Type EFI_FILE_IO_TOKEN is defined in “Related Definitions” below.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |Returned from the call ReadEx()|
| |If Event is NULL (blocking I/O):|
| |The data was read successfully|
| |If Event is not NULL (asynchronous I/O):|
| |The request was successfully queued for processing|
| |Event will be signaled upon completion|
| |Returned in the token after signaling Event|
| |The data was read successfully|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_DEVICE_ERROR |An attempt was made to read from a deleted file|
|EFI_DEVICE_ERROR |On entry, the current file position is beyond the end of the file|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_OUT_OF_RESOURCES |Unable to queue the request due to lack of resources|

*/
type FileReadEx = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV2,
	token: *mut FileIoToken,
) -> Status;

/**
---
Writes data to a file.

# Description

The WriteEx() function writes the specified number of bytes to the file at the current file position.
The current file position is advanced the actual number of bytes written,
which is returned in BufferSize.
Partial writes only occur when there has been a data error
during the write attempt (such as “file space full”).
The file is automatically grown to hold the data if required.

Direct writes to opened directories are not supported.

If non-blocking I/O is used the file pointer will be advanced
based on the order that write requests were submitted.

If an error is returned from the call to WriteEx() and non-blocking I/O is being requested,
the Event associated with this request will not be signaled.
If the call to WriteEx() succeeds then the Event will be signaled
upon completion of the write or if an error occurs during the processing of the request.
The status of the write request can be determined
from the Status field of the Token once the event is signaled.

# Params

***Token***
A pointer to the token associated with the transaction.
Type EFI_FILE_IO_TOKEN is defined in “Related Definitions” above.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |Returned from the call WriteEx()|
| |If Event is NULL (blocking I/O):|
| |The data was written successfully|
| |If Event is not NULL (asynchronous I/O):|
| |The request was successfully queued for processing|
| |Event will be signaled upon completion|
| |Returned in the token after signaling Event|
| |The data was written successfully|
|EFI_UNSUPPORTED |Writes to open directory files are not supported|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_DEVICE_ERROR |An attempt was made to write to a deleted file|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_WRITE_PROTECTED |The file or medium is write-protected|
|EFI_ACCESS_DENIED |The file was opened read only|
|EFI_VOLUME_FULL |The volume is full|
|EFI_OUT_OF_RESOURCES |Unable to queue the request due to lack of resources|

*/
type FileWriteEx = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV2,
	token: *mut FileIoToken,
) -> Status;

/**
---
Flushes all modified data associated with a file to a device.

# Description

The FlushEx() function flushes all modified data associated with a file to a device.
For non-blocking I/O all writes submitted before the flush request will be flushed.
If an error is returned from the call to FlushEx() and non-blocking I/O is being requested,
the Event associated with this request will not be signaled.

# Params

**Token**
A pointer to the token associated with the transaction.
Type EFI_FILE_IO_TOKEN is defined in “Related Definitions” above.
The BufferSize and Buffer fields are not used for a FlushEx operation.

# Return

|code|desc|
|:--|:--|
|EFI_SUCCESS |Returned from the call FlushEx()|
| |If Event is NULL (blocking I/O):|
| |The data was flushed successfully|
| |If Event is not NULL (asynchronous I/O):|
| |The request was successfully queued for processing|
| |Event will be signaled upon completion|
| |Returned in the token after signaling Event|
| |The data was flushed successfully|
|EFI_NO_MEDIA |The device has no medium|
|EFI_DEVICE_ERROR |The device reported an error|
|EFI_VOLUME_CORRUPTED |The file system structures are corrupted|
|EFI_WRITE_PROTECTED |The file or medium is write-protected|
|EFI_ACCESS_DENIED |The file was opened read-only|
|EFI_VOLUME_FULL |The volume is full|
|EFI_OUT_OF_RESOURCES |Unable to queue the request due to lack of resources|

*/
type FileFlushEx = unsafe extern "efiapi" fn(
	this: *mut FileProtocolV2,
	token: *mut FileIoToken,
) -> Status;

#[repr(C)]
pub struct FileProtocolV1 {
	pub revision:     u64,
	pub open:         FileOpen,
	pub close:        FileClose,
	pub delete:       FileDelete,
	pub read:         FileRead,
	pub write:        FileWrite,
	pub get_position: FileGetPosition,
	pub set_position: FileSetPosition,
	pub get_info:     FileGetInfo,
	pub set_info:     FileSetInfo,
	pub flush:        FileFlush,
}

#[repr(C)]
pub struct FileProtocolV2 {
	pub v1:    FileProtocolV1,
	pub open:  FileOpenEx,
	pub read:  FileReadEx,
	pub write: FileWriteEx,
	pub flush: FileFlushEx,
}
