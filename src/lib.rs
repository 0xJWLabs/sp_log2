mod config;
mod loggers;

pub use self::config::{
    format_description, Config, ConfigBuilder, FormatItem, LevelPadding, TargetPadding,
    ThreadLogMode, ThreadPadding, UtcOffset,
};

pub use self::loggers::{CombinedLogger, FileLogger, SimpleLogger, WriteLogger};
#[cfg(feature = "termcolor")]
pub use self::loggers::{TermLogger, TerminalMode};
#[cfg(feature = "termcolor")]
pub use termcolor::{Color, ColorChoice};

pub use log::{Level, LevelFilter};

use log::Log;

#[cfg(feature = "paris")]
#[doc(hidden)]
pub mod __private {
    pub use paris;
}

pub trait SharedLogger: Log {
    /// Returns the set Level for this Logger
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate sp_log;
    /// # use sp_log::*;
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
    /// # extern crate sp_log;
    /// # use sp_log::*;
    /// # fn main() {
    /// let logger = SimpleLogger::new(LevelFilter::Info, Config::default());
    /// println!("{:?}", logger.config());
    /// # }
    /// ```
    fn config(&self) -> Option<&Config>;

    /// Returns the logger as a Log trait object
    fn as_log(self: Box<Self>) -> Box<dyn Log>;
}
