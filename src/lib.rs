mod config;
mod loggers;

pub use self::config::{
    format_description, Config, ConfigBuilder, FormatItem, LevelPadding, TargetPadding,
    ThreadLogMode, ThreadPadding, UtcOffset, Format
};

pub use self::loggers::{CombinedLogger, FileLogger, SimpleLogger, WriteLogger};
pub use self::loggers::{TermLogger, TerminalMode};
pub use termcolor2::{Color, ColorChoice};

pub use log::{Level, LevelFilter};

use log::Log;

pub trait SharedLogger: Log {
    /// Returns the set Level for this Logger
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate sp_log2;
    /// # use sp_log2::*;
    /// # fn main() {
    /// let logger = SimpleLogger::new(LevelFilter::Info, Config::default());
    /// println!("{}", logger.level());
    /// # }
    /// ```
    fn level(&self) -> LevelFilter;

    /// Inspect the config of a running Logger
    ///
    /// An Option is returned, because some Logger may not contain a Config
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate sp_log2;
    /// # use sp_log2::*;
    /// # fn main() {
    /// let logger = SimpleLogger::new(LevelFilter::Info, Config::default());
    /// println!("{:?}", logger.config());
    /// # }
    /// ```
    fn config(&self) -> Option<&Config>;

    /// Returns the logger as a Log trait object
    fn as_log(self: Box<Self>) -> Box<dyn Log>;
}
