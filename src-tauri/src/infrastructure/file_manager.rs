use std::{path::Path, process::Command};

use crate::application::{FileManager, SourceLocationError};

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
