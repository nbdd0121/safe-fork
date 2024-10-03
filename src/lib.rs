use std::io::{Error, Result};
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

/// Ensures the current process is single-threaded.
pub fn ensure_single_threaded() -> Result<()> {
    // SAFETY: `unshare` does not have special safety requirements.
    if unsafe { libc::unshare(libc::CLONE_VM) } == 0 {
        Ok(())
    } else {
        Err(Error::last_os_error())
    }
}

/// Check if the current process is single-threaded.
pub fn is_single_threaded() -> bool {
    ensure_single_threaded().is_ok()
}

/// Representation of a forked child process.
///
/// This is a thin wrapper of the raw PID to provide the `join` helper function.
pub struct Child {
    pid: libc::pid_t,
}

impl Child {
    /// Returns the OS-assigned process identifier associated with this child.
    pub fn pid(&self) -> u32 {
        self.pid as _
    }

    /// Waits for the child to exit completely, returning the status that it
    /// exited with.
    pub fn join(self) -> Result<ExitStatus> {
        // SAFETY: `waitpid` does not have special safety requirements.
        let mut status = 0;
        let ret = unsafe { libc::waitpid(self.pid, &mut status, 0) };
        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(ExitStatus::from_raw(status))
    }
}

/// Fork the current process.
///
/// The forking process must be single-threaded. Otherwise, this call will fail.
pub fn fork() -> Result<Option<Child>> {
    ensure_single_threaded()?;

    // SAFETY: fork is safe for single-threaded process.
    match unsafe { libc::fork() } {
        -1 => Err(std::io::Error::last_os_error()),
        0 => Ok(None),
        pid => Ok(Some(Child { pid })),
    }
}

/// Fork the current process, and execute the provided closure within child process.
pub fn fork_spawn(f: impl FnOnce() -> i32) -> Result<Child> {
    Ok(match fork()? {
        Some(c) => c,
        None => {
            std::process::exit(f());
        }
    })
}

/// Fork the current process, and execute the provided closure within child process, and wait for it to complete.
pub fn fork_join(f: impl FnOnce() -> i32) -> Result<i32> {
    let exit = fork_spawn(f)?.join()?;
    Ok(exit
        .code()
        .or_else(|| exit.signal().map(|x| x + 128))
        .unwrap_or(1))
}
