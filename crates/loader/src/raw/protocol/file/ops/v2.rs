use {
	super::super::FileProtocolV2,
	crate::{
		Status,
		raw::types::{
			Char16,
			file::{FileAttributes, FileIoToken, OpenMode},
		},
	},
};

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
pub(in crate::raw::protocol::file) type FileOpenEx =
	unsafe extern "efiapi" fn(
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
pub(in crate::raw::protocol::file) type FileReadEx =
	unsafe extern "efiapi" fn(
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
pub(in crate::raw::protocol::file) type FileWriteEx =
	unsafe extern "efiapi" fn(
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
pub(in crate::raw::protocol::file) type FileFlushEx =
	unsafe extern "efiapi" fn(
		this: *mut FileProtocolV2,
		token: *mut FileIoToken,
	) -> Status;
