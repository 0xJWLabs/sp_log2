//! Module providing the FileLogger Implementation

use super::logging::try_log;
use crate::{Config, SharedLogger};
use log::{set_boxed_logger, set_max_level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::fs::rename;
use std::fs::File;
use std::fs::OpenOptions;

/// The FileLogger struct. Provides a Logger implementation for structs implementing `Write`, e.g. File
pub struct FileLogger {
    level: LevelFilter,
    config: Config,
    writable: Mutex<File>,
    max_size: Option<u64>, // Maximum size in bytes, if specified
    file_path: String,
}

impl FileLogger {
    /// init function. Globally initializes the FileLogger as the one and only used log facility.
    ///
    /// Takes the desired `Level`, `Config` and `file_path` and `max_size` struct as arguments. They cannot be changed later on.
    /// Fails if another Logger was already initialized.
    ///
    /// # Examples
    /// ```
    /// # extern crate sp_log;
    /// # use sp_log::*;
    /// # fn main() {
    /// let _ = FileLogger::init(LevelFilter::Info, Config::default(), "my_rust_bin.log", Some(1024 * 1024 * 10));
    /// # }
    /// ```
    pub fn init(log_level: LevelFilter, config: Config, file_path: &str, max_size: Option<u64>) -> Result<(), SetLoggerError> {
        set_max_level(log_level);
        set_boxed_logger(Self::new(log_level, config, file_path, max_size))
    }

    /// Rotates the log file when it exceeds the maximum size.
    fn rotate(&self) {
        if let Some(max_size) = self.max_size {
            let writable = self.writable.lock().unwrap();

            // Check current log file size
            if let Ok(metadata) = writable.metadata() {
                if metadata.len() > max_size {
                    // Close current file by dropping the lock
                    drop(writable);

                    // Perform rotation
                    self.perform_rotation();

                    // Reopen log file
                    let new_file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&self.file_path)
                        .unwrap();

                    *self.writable.lock().unwrap() = new_file;
                }
            }
        }
    }

    /// Rename the current log file to the next available backup (e.g., `app.log.1`, `app.log.2`, etc.).
    fn perform_rotation(&self) {
        let path = Path::new(&self.file_path);
        let mut backup_number = 1;

        // Find the first available backup number
        while path.with_extension(backup_number.to_string()).exists() {
            backup_number += 1;
        }

        // Rename current log file to the first available backup
        let backup = path.with_extension(backup_number.to_string());
        if let Err(err) = rename(path, &backup) {
            eprintln!("Error rotating log file: {}", err);
        }
    }

    /// allows to create a new logger, that can be independently used, no matter what is globally set.
    ///
    /// no macros are provided for this case and you probably
    /// dont want to use this function, but `init()`, if you dont want to build a `CombinedLogger`.
    ///
    /// Takes the desired `Level`, `Config` and `Write` struct as arguments. They cannot be changed later on.
    ///
    /// # Examples
    /// ```
    /// # extern crate sp_log;
    /// # use sp_log::*;
    /// # fn main() {
    /// let file_logger = FileLogger::new(LevelFilter::Info, Config::default(), "my_rust_bin.log", Some(1024 * 1024 * 10));
    /// # }
    /// ```
    #[must_use]
    /// Creates a new instance of `FileLogger`.
    pub fn new(
        log_level: LevelFilter,
        config: Config,
        file_path: &str,
        max_size: Option<u64>,
    ) -> Box<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();

        Box::new(Self {
            level: log_level,
            config,
            writable: Mutex::new(file),
            max_size,
            file_path: file_path.to_string(),
        })
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            self.rotate();

            let mut write_lock = self.writable.lock().unwrap();
            let _ = try_log(&self.config, record, &mut *write_lock);
        }
    }

    fn flush(&self) {
        let _ = self.writable.lock().unwrap().flush();
    }
}

impl SharedLogger for FileLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&Config> {
        Some(&self.config)
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}
