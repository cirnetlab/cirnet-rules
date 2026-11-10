use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

static INTERRUPTS: AtomicUsize = AtomicUsize::new(0);
static ACTIVE: Mutex<Active> = Mutex::new(Active {
    users: 0,
    #[cfg(unix)]
    previous: None,
});

struct Active {
    users: usize,
    #[cfg(unix)]
    previous: Option<libc::sigaction>,
}

pub(super) struct Cancellation(usize);

impl Cancellation {
    pub(super) fn begin() -> Result<Self, String> {
        let observed = INTERRUPTS.load(Ordering::Relaxed);
        let mut active = ACTIVE
            .lock()
            .map_err(|_| "Cannot acquire process cancellation")?;
        if active.users == 0 {
            active.install()?;
        }
        active.users += 1;
        Ok(Self(observed))
    }

    pub(super) fn requested(&self) -> bool {
        self.0 != INTERRUPTS.load(Ordering::Relaxed)
    }
}

impl Drop for Cancellation {
    fn drop(&mut self) {
        if let Ok(mut active) = ACTIVE.lock() {
            active.users -= 1;
            if active.users == 0 {
                active.restore();
            }
        }
    }
}

#[cfg(windows)]
impl Active {
    fn install(&mut self) -> Result<(), String> {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        // SAFETY: the process-wide callback has static lifetime and uses only atomics.
        if unsafe { SetConsoleCtrlHandler(Some(on_control), 1) } == 0 {
            return Err("Unable to handle console cancellation".into());
        }
        Ok(())
    }

    fn restore(&mut self) {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        // SAFETY: remove the same static callback registered by install.
        unsafe { SetConsoleCtrlHandler(Some(on_control), 0) };
    }
}

#[cfg(windows)]
unsafe extern "system" fn on_control(event: u32) -> i32 {
    use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT};
    if matches!(event, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        INTERRUPTS.fetch_add(1, Ordering::Relaxed);
        1
    } else {
        0
    }
}

#[cfg(unix)]
impl Active {
    fn install(&mut self) -> Result<(), String> {
        // SAFETY: both actions use initialized C structs. The handler performs one atomic operation.
        let previous = unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            let mut previous: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = on_interrupt as *const () as libc::sighandler_t;
            libc::sigemptyset(&mut action.sa_mask);
            if libc::sigaction(libc::SIGINT, &action, &mut previous) != 0 {
                return Err("Unable to handle terminal cancellation".into());
            }
            previous
        };
        self.previous = Some(previous);
        Ok(())
    }

    fn restore(&mut self) {
        if let Some(previous) = self.previous.take() {
            // SAFETY: install initialized the action with sigaction before saving it.
            unsafe { libc::sigaction(libc::SIGINT, &previous, std::ptr::null_mut()) };
        }
    }
}

#[cfg(unix)]
extern "C" fn on_interrupt(_: libc::c_int) {
    INTERRUPTS.fetch_add(1, Ordering::Relaxed);
}
