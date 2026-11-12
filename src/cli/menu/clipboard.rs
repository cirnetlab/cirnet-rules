use std::fmt;

pub(super) const MAX_BYTES: usize = 256 * 1024;

pub(super) trait Clipboard {
    fn get_text(&mut self) -> Result<String, ClipboardError>;
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ClipboardError {
    Unsupported,
    #[cfg(any(windows, test))]
    Unavailable,
    #[cfg(any(windows, test))]
    PermissionDenied,
    TooLarge,
    InvalidText,
}

impl ClipboardError {
    pub(super) fn code(self) -> &'static str {
        match self {
            Self::Unsupported => "CLIPBOARD_UNSUPPORTED",
            #[cfg(any(windows, test))]
            Self::Unavailable => "CLIPBOARD_UNAVAILABLE",
            #[cfg(any(windows, test))]
            Self::PermissionDenied => "CLIPBOARD_PERMISSION_DENIED",
            Self::TooLarge => "CLIPBOARD_TOO_LARGE",
            Self::InvalidText => "CLIPBOARD_INVALID_TEXT",
        }
    }
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

pub(super) struct SystemClipboard;

fn native_supported(windows: bool, ssh_session: bool) -> bool {
    windows && !ssh_session
}

impl SystemClipboard {
    pub(super) fn supported() -> bool {
        native_supported(
            cfg!(windows),
            ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"]
                .iter()
                .any(|variable| std::env::var_os(variable).is_some()),
        )
    }
}

impl Clipboard for SystemClipboard {
    fn get_text(&mut self) -> Result<String, ClipboardError> {
        #[cfg(windows)]
        if Self::supported() {
            return windows::get_text();
        }
        Err(ClipboardError::Unsupported)
    }

    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        if !Self::supported() {
            return Err(ClipboardError::Unsupported);
        }
        let units = encode_text(text)?;
        #[cfg(windows)]
        {
            windows::set_text(&units)
        }
        #[cfg(not(windows))]
        {
            let _ = units;
            Err(ClipboardError::Unsupported)
        }
    }
}

fn validate_text(text: &str) -> Result<(), ClipboardError> {
    if text.len() > MAX_BYTES {
        return Err(ClipboardError::TooLarge);
    }
    if text
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\t' | '\n' | '\r'))
    {
        return Err(ClipboardError::InvalidText);
    }
    Ok(())
}

fn encode_text(text: &str) -> Result<Vec<u16>, ClipboardError> {
    validate_text(text)?;
    Ok(text.encode_utf16().chain([0]).collect())
}

#[cfg(any(windows, test))]
fn decode_text(units: &[u16]) -> Result<String, ClipboardError> {
    let units = &units[..units.len().min(MAX_BYTES + 1)];
    let end = units.iter().position(|unit| *unit == 0).ok_or({
        if units.len() > MAX_BYTES {
            ClipboardError::TooLarge
        } else {
            ClipboardError::InvalidText
        }
    })?;
    let text = String::from_utf16(&units[..end]).map_err(|_| ClipboardError::InvalidText)?;
    if text.len() > MAX_BYTES {
        return Err(ClipboardError::TooLarge);
    }
    Ok(text)
}

#[cfg(windows)]
mod windows {
    use super::{ClipboardError, MAX_BYTES, decode_text};
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::{
        Foundation::{ERROR_ACCESS_DENIED, GetLastError, GlobalFree, HGLOBAL, HWND},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, GetOpenClipboardWindow,
                IsClipboardFormatAvailable, OpenClipboard, SetClipboardData,
            },
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
            Ole::CF_UNICODETEXT,
        },
        UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, HWND_MESSAGE},
    };

    const UNICODE_TEXT: u32 = CF_UNICODETEXT as u32;

    fn api_error() -> ClipboardError {
        // SAFETY: reads the thread-local Win32 error from the immediately preceding call.
        if unsafe { GetLastError() } == ERROR_ACCESS_DENIED {
            ClipboardError::PermissionDenied
        } else {
            ClipboardError::Unavailable
        }
    }

    struct Open;

    impl Open {
        fn clipboard(owner: HWND) -> Result<Self, ClipboardError> {
            // SAFETY: owner is either null for reading or a live window on this thread.
            if unsafe { OpenClipboard(owner) } != 0 {
                return Ok(Self);
            }
            let error = api_error();
            // SAFETY: this query returns a borrowed handle used only for a null check.
            if unsafe { GetOpenClipboardWindow() }.is_null() {
                Err(error)
            } else {
                Err(ClipboardError::Unavailable)
            }
        }
    }

    impl Drop for Open {
        fn drop(&mut self) {
            // SAFETY: Open exists only after a successful OpenClipboard on this thread.
            unsafe { CloseClipboard() };
        }
    }

    struct Owner(HWND);

    impl Owner {
        fn new() -> Result<Self, ClipboardError> {
            // SAFETY: STATIC is a null-terminated system class and HWND_MESSAGE
            // creates a message-only window owned by the calling thread.
            let window = unsafe {
                CreateWindowExW(
                    0,
                    windows_sys::w!("STATIC"),
                    null(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    HWND_MESSAGE,
                    null_mut(),
                    null_mut(),
                    null(),
                )
            };
            if window.is_null() {
                Err(api_error())
            } else {
                Ok(Self(window))
            }
        }
    }

    impl Drop for Owner {
        fn drop(&mut self) {
            // SAFETY: the owned window was created on this thread and is destroyed once.
            unsafe { DestroyWindow(self.0) };
        }
    }

    struct Allocation(HGLOBAL);

    impl Drop for Allocation {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: a non-null handle here is an allocation still owned by CNR.
                unsafe { GlobalFree(self.0) };
            }
        }
    }

    struct Locked {
        allocation: HGLOBAL,
        units: *mut u16,
    }

    impl Locked {
        fn new(allocation: HGLOBAL) -> Result<Self, ClipboardError> {
            // SAFETY: the handle comes from GlobalAlloc or the open clipboard.
            let units = unsafe { GlobalLock(allocation) }.cast::<u16>();
            if units.is_null() {
                Err(api_error())
            } else {
                Ok(Self { allocation, units })
            }
        }
    }

    impl Drop for Locked {
        fn drop(&mut self) {
            // SAFETY: this guard owns one successful GlobalLock.
            unsafe { GlobalUnlock(self.allocation) };
        }
    }

    pub(super) fn get_text() -> Result<String, ClipboardError> {
        let _open = Open::clipboard(null_mut())?;
        // SAFETY: the clipboard remains open until the function returns.
        let allocation = unsafe {
            if IsClipboardFormatAvailable(UNICODE_TEXT) == 0 {
                return Err(ClipboardError::Unavailable);
            }
            GetClipboardData(UNICODE_TEXT)
        };
        if allocation.is_null() {
            return Err(api_error());
        }
        // SAFETY: CF_UNICODETEXT uses a global memory handle owned by the clipboard.
        let byte_len = unsafe { GlobalSize(allocation) };
        let units = (byte_len / size_of::<u16>()).min(MAX_BYTES + 1);
        if units == 0 {
            return Err(ClipboardError::InvalidText);
        }
        let locked = Locked::new(allocation)?;
        // SAFETY: the clipboard-owned block stays locked and its size bounds this slice.
        decode_text(unsafe { std::slice::from_raw_parts(locked.units, units) })
    }

    pub(super) fn set_text(units: &[u16]) -> Result<(), ClipboardError> {
        // SAFETY: the allocation size includes a UTF-16 terminator and is bounded by MAX_BYTES.
        let mut allocation =
            Allocation(unsafe { GlobalAlloc(GMEM_MOVEABLE, std::mem::size_of_val(units)) });
        if allocation.0.is_null() {
            return Err(api_error());
        }
        {
            let locked = Locked::new(allocation.0)?;
            // SAFETY: the locked destination was sized for exactly this non-overlapping copy.
            unsafe {
                locked
                    .units
                    .copy_from_nonoverlapping(units.as_ptr(), units.len())
            };
        }
        let owner = Owner::new()?;
        let _open = Open::clipboard(owner.0)?;
        // SAFETY: the owned window is active for this clipboard operation and the block
        // is unlocked. Successful SetClipboardData transfers its ownership to Windows.
        if unsafe { EmptyClipboard() } == 0
            || unsafe { SetClipboardData(UNICODE_TEXT, allocation.0) }.is_null()
        {
            return Err(api_error());
        }
        allocation.0 = null_mut();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_unicode_preserves_multiline_text_for_the_editor() {
        let text = "Tiếng Việt 👩‍🔬 漢字\r\nnext\tline";
        let mut units = encode_text(text).unwrap();
        assert_eq!(units.last(), Some(&0));
        units.extend([0, 0]); // Allocator padding can follow the terminator.
        assert_eq!(decode_text(&units).unwrap(), text);
    }

    #[test]
    fn clipboard_rejects_invalid_text_and_excess_bytes() {
        assert_eq!(
            encode_text("first\0second"),
            Err(ClipboardError::InvalidText)
        );
        assert_eq!(decode_text(&[0xD800, 0]), Err(ClipboardError::InvalidText));
        assert_eq!(
            decode_text(&[b'x' as u16]),
            Err(ClipboardError::InvalidText)
        );
        assert_eq!(
            encode_text(&"x".repeat(MAX_BYTES + 1)),
            Err(ClipboardError::TooLarge)
        );
        let oversized_utf8 = ['漢' as u16].repeat(MAX_BYTES / 3 + 1);
        let mut units = oversized_utf8;
        units.push(0);
        assert_eq!(decode_text(&units), Err(ClipboardError::TooLarge));
    }

    #[test]
    fn clipboard_diagnostics_are_stable_and_contain_no_payload() {
        assert_eq!(
            ClipboardError::Unavailable.to_string(),
            "CLIPBOARD_UNAVAILABLE"
        );
        assert_eq!(
            ClipboardError::PermissionDenied.code(),
            "CLIPBOARD_PERMISSION_DENIED"
        );
        assert_eq!(ClipboardError::Unsupported.code(), "CLIPBOARD_UNSUPPORTED");
    }

    #[test]
    fn clipboard_native_capability_respects_remote_sessions() {
        assert!(native_supported(true, false));
        assert!(!native_supported(true, true));
        assert!(!native_supported(false, false));
    }
}
