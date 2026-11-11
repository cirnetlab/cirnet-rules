use std::process::{Child, Command, ExitStatus};

#[cfg(windows)]
mod platform {
    use super::{Child, Command, ExitStatus};
    use std::{
        mem,
        os::windows::{io::AsRawHandle, process::CommandExt},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First,
                Thread32Next,
            },
            JobObjects::{
                AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
                JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
                QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
            },
            Threading::{
                CREATE_NO_WINDOW, CREATE_SUSPENDED, GetProcessIdOfThread, OpenThread, ResumeThread,
                THREAD_QUERY_LIMITED_INFORMATION, THREAD_SUSPEND_RESUME,
            },
        },
    };

    struct Handle(HANDLE);
    impl Handle {
        fn new(raw: HANDLE, reason: &'static str) -> Result<Self, String> {
            if raw.is_null() || raw == INVALID_HANDLE_VALUE {
                Err(reason.into())
            } else {
                Ok(Self(raw))
            }
        }
    }
    impl Drop for Handle {
        fn drop(&mut self) {
            // SAFETY: only validated owned handles are stored here.
            unsafe { CloseHandle(self.0) };
        }
    }

    pub(in crate::execution::process) struct Tree(Handle);

    impl Tree {
        fn new() -> Result<Self, String> {
            // SAFETY: null security attributes make the handle non-inheritable;
            // a null name creates a private Job Object.
            let job = Handle::new(
                unsafe { CreateJobObjectW(ptr::null(), ptr::null()) },
                "Unable to create the process Job Object",
            )?;
            // SAFETY: zero denotes default values for all fields of this native C struct.
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { mem::zeroed() };
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            // SAFETY: the initialized structure and its length match the selected class.
            let configured = unsafe {
                SetInformationJobObject(
                    job.0,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    mem::size_of_val(&limits) as u32,
                )
            };
            if configured == 0 {
                return Err("Unable to configure process-tree cleanup".into());
            }
            Ok(Self(job))
        }

        fn assign(&self, child: &Child) -> Result<(), String> {
            // SAFETY: the handle belongs to the suspended child just spawned.
            let assigned = unsafe { AssignProcessToJobObject(self.0.0, child.as_raw_handle()) };
            if assigned == 0 {
                return Err("The host cannot put this child in an owned Job Object".into());
            }
            Ok(())
        }

        pub(in crate::execution::process) fn stop(&self) -> Result<(), String> {
            // SAFETY: this job handle is non-inheritable and owned by the runner.
            if unsafe { TerminateJobObject(self.0.0, 1) } == 0 {
                return Err("Unable to confirm process Job Object termination".into());
            }
            let deadline = std::time::Instant::now() + super::super::CLEANUP_LIMIT;
            loop {
                // SAFETY: the initialized output buffer matches the queried class.
                let mut stats: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { mem::zeroed() };
                let queried = unsafe {
                    QueryInformationJobObject(
                        self.0.0,
                        JobObjectBasicAccountingInformation,
                        (&mut stats as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                        mem::size_of_val(&stats) as u32,
                        ptr::null_mut(),
                    )
                };
                if queried == 0 {
                    return Err("Unable to observe the owned Job Object".into());
                }
                if stats.ActiveProcesses == 0 {
                    return Ok(());
                }
                if std::time::Instant::now() >= deadline {
                    return Err("Unable to confirm that the owned Job Object has drained".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        pub(in crate::execution::process) fn poll(
            &self,
            child: &mut Child,
        ) -> Result<Option<ExitStatus>, String> {
            let status = child
                .try_wait()
                .map_err(|_| "Unable to observe the owned process")?;
            if status.is_some() {
                self.stop()?;
            }
            Ok(status)
        }
    }

    fn resume(child: &Child) -> Result<(), String> {
        // SAFETY: TH32CS_SNAPTHREAD ignores the second argument.
        let snapshot = Handle::new(
            unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) },
            "Unable to inspect the suspended child's primary thread",
        )?;
        // SAFETY: this native entry is initialized and its required size is set.
        let mut entry: THREADENTRY32 = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<THREADENTRY32>() as u32;
        // SAFETY: snapshot remains valid and entry points to writable memory.
        let mut more = unsafe { Thread32First(snapshot.0, &mut entry) } != 0;
        let mut owned = None;
        while more {
            if entry.th32OwnerProcessID == child.id() && owned.replace(entry.th32ThreadID).is_some()
            {
                return Err("Unexpected additional thread in the suspended child".into());
            }
            entry.dwSize = mem::size_of::<THREADENTRY32>() as u32;
            // SAFETY: same validated handles and entry as above.
            more = unsafe { Thread32Next(snapshot.0, &mut entry) } != 0;
        }
        let tid = owned.ok_or("The suspended child has no identifiable primary thread")?;
        // SAFETY: request only the rights needed to recheck ownership and resume.
        let thread = Handle::new(
            unsafe {
                OpenThread(
                    THREAD_QUERY_LIMITED_INFORMATION | THREAD_SUSPEND_RESUME,
                    0,
                    tid,
                )
            },
            "Unable to open the suspended child thread",
        )?;
        // SAFETY: validated thread handle, then recheck the exact spawned process.
        if unsafe { GetProcessIdOfThread(thread.0) } != child.id() {
            return Err("The suspended thread no longer belongs to the owned child".into());
        }
        // SAFETY: the only suspend count is expected to be from CREATE_SUSPENDED.
        if unsafe { ResumeThread(thread.0) } != 1 {
            return Err("Unable to start the contained child".into());
        }
        Ok(())
    }

    pub(in crate::execution::process) fn spawn(
        command: &mut Command,
        hide_window: bool,
    ) -> Result<(Child, Tree), String> {
        let tree = Tree::new()?;
        let flags = CREATE_SUSPENDED | if hide_window { CREATE_NO_WINDOW } else { 0 };
        let mut child = command
            .creation_flags(flags)
            .spawn()
            .map_err(|_| "Unable to start the approved executable")?;
        if let Err(error) = tree.assign(&child).and_then(|_| resume(&child)) {
            // The primary thread has not been given permission to run on the
            // assignment failure path. The private job covers resume failures.
            let _ = child.kill();
            let _ = super::super::wait_bounded(&mut child);
            return Err(error);
        }
        Ok((child, tree))
    }

    pub(in crate::execution::process) const OWNERSHIP: &str = "windows_job_object";
}

#[cfg(unix)]
mod platform {
    use super::{Child, Command, ExitStatus};
    use std::os::unix::process::CommandExt;

    pub(in crate::execution::process) struct Tree(libc::pid_t);
    impl Tree {
        pub(in crate::execution::process) fn poll(
            &self,
            child: &mut Child,
        ) -> Result<Option<ExitStatus>, String> {
            // Observe exit without reaping. This reserves the leader PID
            // while its private group is stopped, avoiding a reused group ID.
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            let result = unsafe {
                libc::waitid(
                    libc::P_PID,
                    self.0 as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            if result != 0 {
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                    return Ok(None);
                }
                return Err("Unable to observe the owned process group".into());
            }
            if info.si_signo == 0 {
                return Ok(None);
            }
            self.stop()?;
            super::super::wait_bounded(child).map(Some)
        }

        pub(in crate::execution::process) fn stop(&self) -> Result<(), String> {
            // SAFETY: spawn created a private process group whose ID equals
            // its leader's PID. Negative ID targets this owned group only.
            if unsafe { libc::kill(-self.0, libc::SIGKILL) } != 0
                && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
            {
                return Err("Unable to stop the owned process group".into());
            }
            Ok(())
        }
    }

    pub(in crate::execution::process) fn spawn(
        command: &mut Command,
        _hide_window: bool,
    ) -> Result<(Child, Tree), String> {
        let mut child = command
            .process_group(0)
            .spawn()
            .map_err(|_| "Unable to start the approved executable")?;
        let pid: libc::pid_t = match child.id().try_into() {
            Ok(pid) => pid,
            Err(_) => {
                let _ = child.kill();
                let _ = super::super::wait_bounded(&mut child);
                return Err("Unexpected owned process identifier".into());
            }
        };
        Ok((child, Tree(pid)))
    }

    pub(in crate::execution::process) const OWNERSHIP: &str = "unix_process_group";
}

pub(super) use platform::{OWNERSHIP, spawn};
