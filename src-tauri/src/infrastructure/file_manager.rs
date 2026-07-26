use std::{path::Path, process::Command};

use crate::application::{ExternalPathOpener, FileManager, NotesError, SourceLocationError};

pub(crate) struct SystemFileManager;

impl SystemFileManager {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl FileManager for SystemFileManager {
    fn open_directory(&self, directory: &Path) -> Result<(), SourceLocationError> {
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = Command::new("explorer.exe");
            command.arg(directory);
            command
        };

        #[cfg(target_os = "macos")]
        let mut command = {
            let mut command = Command::new("open");
            command.arg(directory);
            command
        };

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        return Err(SourceLocationError::LaunchFailed);

        command
            .spawn()
            .map(|_| ())
            .map_err(|_| SourceLocationError::LaunchFailed)
    }
}

impl ExternalPathOpener for SystemFileManager {
    fn open_path(&self, path: &Path) -> Result<(), NotesError> {
        if path.is_dir() {
            return self
                .open_directory(path)
                .map_err(|_| NotesError::LaunchFailed);
        }

        #[cfg(target_os = "windows")]
        {
            use std::{ffi::c_void, os::windows::ffi::OsStrExt};

            #[link(name = "shell32")]
            unsafe extern "system" {
                fn ShellExecuteW(
                    window: *mut c_void,
                    operation: *const u16,
                    file: *const u16,
                    parameters: *const u16,
                    directory: *const u16,
                    show_command: i32,
                ) -> isize;
            }
            let operation: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
            let file: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            // SAFETY: operation and file are valid null-terminated UTF-16 strings.
            let result = unsafe {
                ShellExecuteW(
                    std::ptr::null_mut(),
                    operation.as_ptr(),
                    file.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    1,
                )
            };
            if result > 32 {
                Ok(())
            } else {
                Err(NotesError::LaunchFailed)
            }
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(path)
                .spawn()
                .map(|_| ())
                .map_err(|_| NotesError::LaunchFailed)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        Err(NotesError::LaunchFailed)
    }
}
