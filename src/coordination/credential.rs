use crate::policy;
use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};

pub(super) fn open_read(path: &Path, limit: u64) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options
        .open(path)
        .map_err(|_| "Cannot open local capability file")?;
    let meta = file
        .metadata()
        .map_err(|_| "Cannot inspect local capability file")?;
    if !meta.is_file() || policy::is_link(&meta) || meta.len() > limit {
        return Err("Invalid local capability file".into());
    }
    Ok(file)
}

pub(super) fn read(path: &Path, limit: u64) -> Result<String, String> {
    let file = open_read(path, limit)?;
    #[cfg(unix)]
    {
        let meta = file
            .metadata()
            .map_err(|_| "Cannot inspect local capability permissions")?;
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err("Local capability file requires owner-only permissions".into());
        }
    }
    let mut raw = String::new();
    file.take(limit + 1)
        .read_to_string(&mut raw)
        .map_err(|_| "Cannot read local capability file")?;
    if raw.len() as u64 > limit {
        return Err("Local capability file exceeds its size limit".into());
    }
    Ok(raw)
}

#[cfg(unix)]
pub(crate) fn create(path: &Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "Cannot create private local capability file".into())
}

#[cfg(windows)]
pub(crate) fn create(path: &Path) -> Result<File, String> {
    use std::{
        ffi::c_void,
        os::windows::{ffi::OsStrExt, io::FromRawHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{GENERIC_WRITE, INVALID_HANDLE_VALUE, LocalFree},
        Security::SECURITY_ATTRIBUTES,
        Storage::FileSystem::{CREATE_NEW, CreateFileW, FILE_ATTRIBUTE_NORMAL},
    };
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            size: *mut u32,
        ) -> i32;
    }
    // Protect the new file's DACL from inherited access. Allow the file owner and SYSTEM.
    let sddl: Vec<u16> = "D:P(A;;GA;;;OW)(A;;GA;;;SY)"
        .encode_utf16()
        .chain([0])
        .collect();
    let mut descriptor = ptr::null_mut();
    // SDDL and the out-pointer are valid for the call. Windows allocates the descriptor.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err("Cannot prepare private capability file security".into());
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let filename: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    // The path, attributes, and descriptor stay allocated through CreateFileW.
    let handle = unsafe {
        let handle = CreateFileW(
            filename.as_ptr(),
            GENERIC_WRITE,
            0,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        );
        LocalFree(descriptor);
        handle
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err("Cannot create private local capability file".into());
    }
    // The valid owned handle has not been passed elsewhere.
    Ok(unsafe { File::from_raw_handle(handle) })
}
