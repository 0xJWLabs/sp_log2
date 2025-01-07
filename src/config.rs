use log::Level;
use log::LevelFilter;

use std::borrow::Cow;
use termcolor2::Color;

#[derive(Debug, Clone, Copy)]
/// Defines how padding should be applied to the logging level in the log output.
pub enum LevelPadding {
    /// Pad the logging level with spaces to the left.
    Left,

    /// Pad the logging level with spaces to the right.
    Right,

    /// No padding applied to the logging level.
    Off,
}

#[derive(Debug, Clone, Copy)]
/// Defines how padding should be applied to the thread information in the log output.
pub enum ThreadPadding {
    /// Pad the thread information with spaces to the left, with a specified width.
    Left(usize),

    /// Pad the thread information with spaces to the right, with a specified width.
    Right(usize),

    /// No padding applied to the thread information.
    Off,
}

#[derive(Debug, Clone, Copy)]
/// Padding to be used for logging the thread id/name
pub enum TargetPadding {
    /// Add spaces on the left side, up to usize many
    Left(usize),
    /// Add spaces on the right side, up to usize many
    Right(usize),
    /// Do not pad the thread id/name
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Mode for logging the thread name or id or both.
pub enum ThreadLogMode {
    /// Log thread ids only
    IDs,
    /// Log the thread names only
    Names,
    /// If this thread is named, log the name. Otherwise, log the thread id.
    Both,
}

#[derive(Debug, Clone)]
pub(crate) enum TimeFormat {
    Rfc2822,
    Rfc3339,
    Custom(&'static str),
}

#[allow(non_upper_case_globals, non_snake_case)]
pub mod Format {
    /// Flag to include the time in the log format.
    pub const Time: u8 = 1;

    /// Flag to include the log level (e.g., INFO, ERROR) in the log format.
    pub const LevelFlag: u8 = 2;

    /// Flag to include the thread information in the log format.
    pub const Thread: u8 = 4;

    /// Flag to include the file location (e.g., file name, line number) in the log format.
    pub const FileLocation: u8 = 8;

    /// Flag to include the target (e.g., module or crate) in the log format.
    pub const Target: u8 = 16;

    /// Flag to include the module name in the log format.
    pub const Module: u8 = 32;
}

/// UTF-8 end of line character sequences
pub enum LineEnding {
    /// Line feed
    LF,
    /// Carriage return
    CR,
    /// Carriage return + Line feed
    Crlf,
    /// Vertical tab
    VT,
    /// Form feed
    FF,
    /// Next line
    Nel,
    /// Line separator
    LS,
    /// Paragraph separator
    PS,
}

/// Configuration for the Loggers
///
/// All loggers print the message in the following form:
/// `00:00:00 [LEVEL] crate::module: [lib.rs::100] your_message`
/// Every space delimited part except the actual message is optional.
///
/// Pass this struct to your logger to change when these information shall
/// be logged.
///
/// Construct using [`Default`](Config::default) or using [`ConfigBuilder`]
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) format: u8,
    pub(crate) level_padding: LevelPadding,
    pub(crate) thread_log_mode: ThreadLogMode,
    pub(crate) thread_padding: ThreadPadding,
    pub(crate) target_padding: TargetPadding,
    pub(crate) min_level: LevelFilter,
    pub(crate) max_level: LevelFilter,
    pub(crate) time_format: TimeFormat,
    pub(crate) filter_allow: Cow<'static, [Cow<'static, str>]>,
    pub(crate) filter_ignore: Cow<'static, [Cow<'static, str>]>,
    pub(crate) level_color: [Option<Color>; 6],
    pub(crate) enable_colors: bool,
    pub(crate) line_ending: String,
    pub(crate) formatter: Option<String>,
}

impl Config {
    /// Create a new default `ConfigBuilder`
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    /// Creates a new `ConfigBuilder` with default configuration values.
    pub fn new() -> ConfigBuilder {
        ConfigBuilder(Config::default())
    }

    /// Sets a custom line ending for the logger.
    ///
    /// The supported options are:
    /// - `LF` (Line Feed)
    /// - `CR` (Carriage Return)
    /// - `Crlf` (Carriage Return + Line Feed)
    /// - `VT` (Vertical Tab)
    /// - `FF` (Form Feed)
    /// - `Nel` (Next Line)
    /// - `LS` (Line Separator)
    /// - `PS` (Paragraph Separator)
    pub fn set_line_ending(&mut self, line_ending: LineEnding) -> &mut ConfigBuilder {
        match line_ending {
            LineEnding::LF => self.0.line_ending = String::from("\u{000A}"),
            LineEnding::CR => self.0.line_ending = String::from("\u{000D}"),
            LineEnding::Crlf => self.0.line_ending = String::from("\u{000D}\u{000A}"),
            LineEnding::VT => self.0.line_ending = String::from("\u{000B}"),
            LineEnding::FF => self.0.line_ending = String::from("\u{000C}"),
            LineEnding::Nel => self.0.line_ending = String::from("\u{0085}"),
            LineEnding::LS => self.0.line_ending = String::from("\u{2028}"),
            LineEnding::PS => self.0.line_ending = String::from("\u{2029}"),
        }
        self
    }

    /// Sets the logging format.
    ///
    /// The `format` value is an unsigned 8-bit integer that determines the format of the log entries.
    pub fn set_format(&mut self, format: u8) -> &mut ConfigBuilder {
        self.0.format = format;
        self
    }

    /// Sets the custom formatter for the logs.
    ///
    /// The `formatter` is an optional string representing the format to be used. If `None`, the default format is applied.
    pub fn set_formatter(&mut self, formatter: Option<&str>) -> &mut ConfigBuilder {
        self.0.formatter = formatter.map(|s| s.to_string());
        self
    }

    /// Sets the minimum log level filter.
    ///
    /// The `level` value specifies the minimum level of logs to be displayed. Logs with a level lower than this will be ignored.
    pub fn set_min_level(&mut self, level: LevelFilter) -> &mut ConfigBuilder {
        self.0.min_level = level;
        self
    }

    /// Sets the maximum log level filter.
    ///
    /// The `level` value specifies the maximum level of logs to be displayed. Logs with a level higher than this will be ignored.
    pub fn set_max_level(&mut self, level: LevelFilter) -> &mut ConfigBuilder {
        self.0.max_level = level;
        self
    }

    /// Enables or disables the use of colors in the logs.
    ///
    /// The `enable` flag determines whether colors should be used in the log output.
    pub fn set_enable_colors(&mut self, enable: bool) -> &mut ConfigBuilder {
        self.0.enable_colors = enable;
        self
    }

    /// Sets the padding for the target field in the log output.
    ///
    /// The `padding` value determines how the target field should be padded.
    pub fn set_target_padding(&mut self, padding: TargetPadding) -> &mut ConfigBuilder {
        self.0.target_padding = padding;
        self
    }

    /// Sets the padding for the log level field.
    ///
    /// The `padding` value determines how the level field should be padded when logging. Default is `Off`.
    pub fn set_level_padding(&mut self, padding: LevelPadding) -> &mut ConfigBuilder {
        self.0.level_padding = padding;
        self
    }

    /// Sets the padding for the thread field in the log output.
    ///
    /// The `padding` value determines how the thread field should be padded.
    pub fn set_thread_padding(&mut self, padding: ThreadPadding) -> &mut ConfigBuilder {
        self.0.thread_padding = padding;
        self
    }

    /// Sets the mode for logging thread information.
    ///
    /// The `mode` value determines how the thread field is logged.
    pub fn set_thread_mode(&mut self, mode: ThreadLogMode) -> &mut ConfigBuilder {
        self.0.thread_log_mode = mode;
        self
    }

    /// Sets the color used for logging the log level.
    ///
    /// If `color` is `None`, the default foreground color is used.
    /// This is useful when customizing the log output appearance based on log levels.
    pub fn set_level_color(&mut self, level: Level, color: Option<Color>) -> &mut ConfigBuilder {
        self.0.level_color[level as usize] = color;
        self
    }

    /// Sets the time format to a custom representation.
    ///
    /// *Note*: The default time format is `%H:%M:%S`.
    ///
    /// The syntax for the format can be found in the
    /// [`strftime` crate book](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).
    ///
    /// # Usage
    ///
    /// ```rust
    /// use sp_log2::ConfigBuilder;
    /// let config = ConfigBuilder::new()
    ///     .set_time_format_custom("%Y-%m-%d %H:%M:%S")
    ///     .build();
    /// ```
    pub fn set_time_format_custom(&mut self, time_format: &str) -> &mut ConfigBuilder {
        self.0.time_format =
            TimeFormat::Custom(Box::leak(time_format.to_string().into_boxed_str()));
        self
    }

    /// Sets the time format to RFC 2822.
    ///
    /// This format is typically used for email headers and specifies a standard date-time representation.
    pub fn set_time_format_rfc2822(&mut self) -> &mut ConfigBuilder {
        self.0.time_format = TimeFormat::Rfc2822;
        self
    }

    /// Sets the time format to RFC 3339.
    ///
    /// This format is a common representation for timestamps in log entries.
    pub fn set_time_format_rfc3339(&mut self) -> &mut ConfigBuilder {
        self.0.time_format = TimeFormat::Rfc3339;
        self
    }

    /// Add allowed target filters.
    /// If any are specified, only records from targets matching one of these entries will be printed
    ///
    /// For example, `add_filter_allow_str("tokio::uds")` would allow only logging from the `tokio` crates `uds` module.
    pub fn add_filter_allow_str(&mut self, filter_allow: &'static str) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_allow);
        list.push(Cow::Borrowed(filter_allow));
        self.0.filter_allow = Cow::Owned(list);
        self
    }

    /// Adds an allowed target filter with a dynamic string.
    ///
    /// This specifies that only log entries from the given target will be printed.
    /// For example, `add_filter_allow(format!("{}::{}","tokio", "uds"))` will allow logs only from the `tokio::uds` target.
    pub fn add_filter_allow(&mut self, filter_allow: String) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_allow);
        list.push(Cow::Owned(filter_allow));
        self.0.filter_allow = Cow::Owned(list);
        self
    }

    /// Clears all allowed target filters.
    ///
    /// This removes any previously set filters and allows logs from all targets.
    pub fn clear_filter_allow(&mut self) -> &mut ConfigBuilder {
        self.0.filter_allow = Cow::Borrowed(&[]);
        self
    }

    /// Add denied target filters.
    /// If any are specified, records from targets matching one of these entries will be ignored
    ///
    /// For example, `add_filter_ignore_str("tokio::uds")` would deny logging from the `tokio` crates `uds` module.
    pub fn add_filter_ignore_str(&mut self, filter_ignore: &'static str) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_ignore);
        list.push(Cow::Borrowed(filter_ignore));
        self.0.filter_ignore = Cow::Owned(list);
        self
    }

    /// Add denied target filters.
    /// If any are specified, records from targets matching one of these entries will be ignored
    ///
    /// For example, `add_filter_ignore(format!("{}::{}","tokio", "uds"))` would deny logging from the `tokio` crates `uds` module.
    pub fn add_filter_ignore(&mut self, filter_ignore: String) -> &mut ConfigBuilder {
        let mut list = Vec::from(&*self.0.filter_ignore);
        list.push(Cow::Owned(filter_ignore));
        self.0.filter_ignore = Cow::Owned(list);
        self
    }

    /// Clears all denied target filters.
    ///
    /// This removes any previously set filters and does not filter out any targets.
    pub fn clear_filter_ignore(&mut self) -> &mut ConfigBuilder {
        self.0.filter_ignore = Cow::Borrowed(&[]);
        self
    }

    /// Builds and returns the final `Config` instance.
    ///
    /// This applies all the configurations set in the builder and returns the complete `Config`.
    pub fn build(&mut self) -> Config {
        self.0.clone()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder::new()
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            format: Format::LevelFlag | Format::Time | Format::Thread | Format::Target,
            level_padding: LevelPadding::Off,
            thread_log_mode: ThreadLogMode::IDs,
            thread_padding: ThreadPadding::Off,
            target_padding: TargetPadding::Off,
            time_format: TimeFormat::Custom("%H:%M:%S"),
            filter_allow: Cow::Borrowed(&[]),
            filter_ignore: Cow::Borrowed(&[]),
            enable_colors: true,
            max_level: LevelFilter::Error,
            min_level: LevelFilter::Trace,
            formatter: None,
            level_color: [
                None,                // Default foreground
                Some(Color::Red),    // Error
                Some(Color::Yellow), // Warn
                Some(Color::Blue),   // Info
                Some(Color::Cyan),   // Debug
                Some(Color::White),  // Trace
            ],

            line_ending: String::from("\u{000A}"),
        }
    }
}
