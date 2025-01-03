use crate::config::{Format, TargetPadding, TimeFormat};
use crate::{Config, LevelPadding, ThreadLogMode, ThreadPadding};
use log::Record;
use std::any::Any;
use std::io::{Error, Write};
use std::str::FromStr;
use std::thread;
use termcolor::{BufferedStandardStream, Color, ColorSpec, WriteColor};

/// Attempts to log a message based on the provided configuration.
/// Writes the log message to the provided writer if it should not be skipped.
#[inline(always)]
pub fn try_log<W>(config: &Config, record: &Record<'_>, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized + Any,
{
    if should_skip(config, record) {
        return Ok(());
    }

    if record.level() > config.min_level || record.level() < config.max_level {
        return Ok(());
    }

    let mut level = String::new();
    let mut time = String::new();
    let mut thread = String::new();
    let mut target = String::new();
    let mut location = String::new();
    let mut module = String::new();

    if config.format & Format::Time != 0 {
        time = write_time(config)?;
    }

    if config.format & Format::LevelFlag != 0 {
        level = write_level(record, config)?;
    }

    if config.format & Format::Thread != 0 {
        thread = match config.thread_log_mode {
            ThreadLogMode::IDs => write_thread_id(config)?,
            ThreadLogMode::Names | ThreadLogMode::Both => write_thread_name(config)?,
        }
    }

    if config.format & Format::Target != 0 {
        target = write_target(record, config)?;
    }

    if config.format & Format::FileLocation != 0 {
        location = write_location(record)?;
    }

    if config.format & Format::Module != 0 {
        module = write_module(record)?;
    }

    let args = write_args(record, &config.line_ending)?;

    if config.formatter.is_some() {
        parse_and_format_log(
            write, config, &level, &time, &thread, &target, &location, &module, &args,
        )?;
    } else {
        if !time.is_empty() {
            write!(write, "{}", time)?;
        }

        if !level.is_empty() {
            write!(write, " [{}]", level)?;
        }

        if !thread.is_empty() {
            write!(write, " ({})", thread)?;
        }

        if !target.is_empty() {
            write!(write, " {}:", target)?;
        }

        write!(write, " {}", args)?;

        if !location.is_empty() {
            write!(write, " [{}]", location)?;
        }

        writeln!(write)?;
    }

    Ok(())
}

/// Writes the current time based on the configured format.
#[inline(always)]
pub fn write_time(config: &Config) -> Result<String, Error> {
    use time::error::Format;
    use time::format_description::well_known::*;

    let time = time::OffsetDateTime::now_utc().to_offset(config.time_offset);
    let formatted_time = match config.time_format {
        TimeFormat::Rfc2822 => time.format(&Rfc2822),
        TimeFormat::Rfc3339 => time.format(&Rfc3339),
        TimeFormat::Custom(format) => time.format(&format),
    };

    match formatted_time {
        Ok(time_string) => Ok(time_string),
        Err(Format::StdIo(err)) => Err(err),
        Err(err) => panic!("Invalid time format: {}", err),
    }
}

/// Writes the log level to a string based on the configured padding.
#[inline(always)]
pub fn write_level(record: &Record<'_>, config: &Config) -> Result<String, Error> {
    let level = match config.level_padding {
        LevelPadding::Left => format!("{: >5}", record.level()),
        LevelPadding::Right => format!("{: <5}", record.level()),
        LevelPadding::Off => record.level().to_string(),
    };

    let formatted_level = level.to_string();

    Ok(formatted_level)
}

/// Writes the target (module) of the log record based on the configured padding.
#[inline(always)]
pub fn write_target(record: &Record<'_>, config: &Config) -> Result<String, Error> {
    let target = match config.target_padding {
        TargetPadding::Left(pad) => format!("{:>pad$}", record.target(), pad = pad),
        TargetPadding::Right(pad) => format!("{:<pad$}", record.target(), pad = pad),
        TargetPadding::Off => record.target().to_string(),
    };
    Ok(target)
}

/// Writes the file and line number of the log record's source location.
#[inline(always)]
pub fn write_location(record: &Record<'_>) -> Result<String, Error> {
    let file = record.file().unwrap_or("<unknown>").replace("\\", "/");
    let location = if let Some(line) = record.line() {
        format!("{}:{}", file, line)
    } else {
        format!("{}:<unknown>", file)
    };
    Ok(location)
}

/// Writes the module path of the log record.
#[inline(always)]
pub fn write_module(record: &Record<'_>) -> Result<String, Error> {
    let module = record.module_path().unwrap_or("<unknown>");

    Ok(module.to_string())
}

/// Writes the current thread's name based on the configuration.
pub fn write_thread_name(config: &Config) -> Result<String, Error> {
    if let Some(name) = thread::current().name() {
        let thread_name = match config.thread_padding {
            ThreadPadding::Left { 0: qty } => {
                format!("{:>width$}", name, width = qty)
            }
            ThreadPadding::Right { 0: qty } => {
                format!("{:<width$}", name, width = qty)
            }
            ThreadPadding::Off => name.to_string(),
        };
        Ok(thread_name)
    } else if config.thread_log_mode == ThreadLogMode::Both {
        write_thread_id(config)
    } else {
        Ok(String::new())
    }
}

/// Writes the current thread's ID based on the configuration.
pub fn write_thread_id(config: &Config) -> Result<String, Error> {
    let id = format!("{:?}", thread::current().id())
        .replace("ThreadId(", "")
        .replace(")", "");
    let thread_id = match config.thread_padding {
        ThreadPadding::Left { 0: qty } => {
            format!("{:>width$}", id, width = qty)
        }
        ThreadPadding::Right { 0: qty } => {
            format!("{:<width$}", id, width = qty)
        }
        ThreadPadding::Off => id.to_string(),
    };
    Ok(thread_id)
}

/// Writes the arguments of the log record, appending a line ending.
#[inline(always)]
pub fn write_args(record: &Record<'_>, line_ending: &str) -> Result<String, Error> {
    Ok(format!("{}{}", record.args(), line_ending))
}

/// Determines whether the log record should be skipped based on the configuration's filters.
#[inline(always)]
pub fn should_skip(config: &Config, record: &Record<'_>) -> bool {
    // If a module path and allowed list are available
    match (record.target(), &*config.filter_allow) {
        (path, allowed) if !allowed.is_empty() => {
            // Check that the module path matches at least one allow filter
            if !allowed.iter().any(|v| path.starts_with(&**v)) {
                // If not, skip any further writing
                return true;
            }
        }
        _ => {}
    }

    // If a module path and ignore list are available
    match (record.target(), &*config.filter_ignore) {
        (path, ignore) if !ignore.is_empty() => {
            // Check that the module path does not match any ignore filters
            if ignore.iter().any(|v| path.starts_with(&**v)) {
                // If not, skip any further writing
                return true;
            }
        }
        _ => {}
    }

    false
}

#[inline]
fn parse_percent_or_255(s: &str) -> Option<(u8, bool)> {
    if s.ends_with('%') {
        s.strip_suffix('%')
            .and_then(|s| s.parse().ok())
            .map(|t: u8| (t, true))
    } else {
        s.parse().ok().map(|t: u8| (t, false))
    }
}

#[inline]
fn parse_hex(s: &str) -> Option<Color> {
    if !s.is_ascii() {
        return None;
    }

    let len = s.len();
    match len {
        3 => Some(Color::Rgb(
            u8::from_str_radix(&s[0..1], 16).ok()? * 17,
            u8::from_str_radix(&s[1..2], 16).ok()? * 17,
            u8::from_str_radix(&s[2..3], 16).ok()? * 17,
        )),
        6 => {
            // Handle 6-character hex (e.g., "ff00ff")
            Some(Color::Rgb(
                u8::from_str_radix(&s[0..2], 16).ok()?,
                u8::from_str_radix(&s[2..4], 16).ok()?,
                u8::from_str_radix(&s[4..6], 16).ok()?,
            ))
        }
        _ => None, // Return None for invalid hex lengths
    }
}

#[inline]
fn parse_rgb(rgb: &str) -> Option<Color> {
    let params: Vec<&str> = rgb.split(',').map(|s| s.trim()).collect();
    if params.len() != 3 {
        return None;
    }

    let r = parse_percent_or_255(params[0]);
    let g = parse_percent_or_255(params[1]);
    let b = parse_percent_or_255(params[2]);

    if let (Some((r, r_fmt)), Some((g, g_fmt)), Some((b, b_fmt))) = (r, g, b) {
        if r_fmt == g_fmt && g_fmt == b_fmt {
            return Some(Color::Rgb(r, g, b))
        }
    }
    None
}

#[inline]
fn apply_style(style: &str) -> Option<(Color, bool)> {
    if style.starts_with('#') || style.starts_with("bg#") {
        let prefix_len = if style.starts_with('#') { 1 } else { 3 };
        let hex = &style[prefix_len..style.len()];
        if let Some(color) = parse_hex(hex) {
            return Some((color, style.starts_with('#'))); // false indicates background
        }
    }

    if style.starts_with("rgb(") || style.starts_with("bgrgb(") {
        let prefix_len = if style.starts_with("rgb(") { 4 } else { 6 }; // Remove "rgb(" or "bgrgb("
        let rgb = &style[prefix_len..style.len() - 1];
        if let Some(color) = parse_rgb(rgb) {
            return Some((color, style.starts_with("rgb(")))
        }
    }

    if let Some(color_name) = style.strip_prefix("bg") {
        if let Ok(color) = Color::from_str(color_name) {
            return Some((color, false));
        }
    } else if let Ok(color) = Color::from_str(style) {
        return Some((color, true));
    }

    None
}

#[inline]
#[allow(clippy::too_many_arguments)]
pub fn parse_and_format_log_term(
    writer: &mut BufferedStandardStream,
    level_color: Option<Color>,
    config: &Config,
    level: &str,
    time: &str,
    thread: &str,
    target: &str,
    file: &str,
    module: &str,
    message: &str,
) -> Result<(), Error> {
    parse_and_format_log_internal(
        writer,
        level_color,
        config,
        level,
        time,
        thread,
        target,
        file,
        module,
        message,
        true,
    )
}

#[inline]
#[allow(clippy::too_many_arguments)]
pub fn parse_and_format_log<W>(
    writer: &mut W,
    config: &Config,
    level: &str,
    time: &str,
    thread: &str,
    target: &str,
    file: &str,
    module: &str,
    message: &str,
) -> Result<(), Error>
where
    W: Write + Sized + Any,
{
    parse_and_format_log_internal(
        writer, None, config, level, time, thread, target, file, module, message, false,
    )
}

#[allow(clippy::too_many_arguments)]
fn parse_and_format_log_internal<W>(
    writer: &mut W,
    level_color: Option<Color>,
    config: &Config,
    level: &str,
    time: &str,
    thread: &str,
    target: &str,
    file: &str,
    module: &str,
    message: &str,
    is_terminal: bool,
) -> Result<(), Error>
where
    W: Write + Sized + Any,
{
    let format_str = config.formatter.clone().unwrap();
    let mut last_end = 0; // Tracks the position of the last match's end

    // Process each part of the format string
    for (i, c) in format_str.char_indices() {
        if c == '{' {
            if let Some(end) = format_str[i..].find('}') {
                let end = i + end;

                // Write the part before the placeholder
                if last_end < i {
                    write!(writer, "{}", &format_str[last_end..i])?;
                }

                // Extract the key (and possibly the modifiers)
                let placeholder = &format_str[i + 1..end];
                let parts: Vec<&str> = placeholder.split(':').collect();
                let key = parts[0];

                let mut use_bracket_level = true;

                if is_terminal {
                    // Parse modifiers
                    let styles = if parts.len() > 1 { parts[1..].to_vec() } else { vec![] };

                    let mut fg_color = None;
                    let mut bg_color = None;
                    let mut bold = false;
                    let mut italic = false;
                    let mut dim = false;
                    let mut underline = false;
                    let mut strikethrough = false;

                    for style in styles {
                        match style.to_ascii_lowercase().as_str() {
                            "bold" => bold = true,
                            "italic" => italic = true,
                            "dim" => dim = true,
                            "underline" => underline = true,
                            "strikethrough" => strikethrough = true,
                            "nb" | "nobrackets" | "no_brackets" => {
                                if key == "level" {
                                    use_bracket_level = false;
                                }
                            }
                            _ => {
                                if let Some((color, is_fg)) = apply_style(style) {
                                    if is_fg {
                                        fg_color = fg_color.or(Some(color));
                                    } else {
                                        bg_color = bg_color.or(Some(color));
                                    }
                                }
                            }
                        }
                    }

                    if key == "level" {
                        fg_color = fg_color.or(level_color);
                    }

                    if config.enable_colors {
                        if let Some(writer) =
                            (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>()
                        {
                            writer.set_color(
                                ColorSpec::new()
                                    .set_fg(fg_color)
                                    .set_bg(bg_color)
                                    .set_bold(bold)
                                    .set_italic(italic)
                                    .set_dimmed(dim)
                                    .set_underline(underline)
                                    .set_strikethrough(strikethrough),
                            )?;
                        }
                    }
                }

                // Write the value for the placeholder
                match key {
                    "time" => write!(writer, "{}", time)?,
                    "thread" => write!(writer, "{}", thread)?,
                    "target" => write!(writer, "{}", target)?,
                    "level" => {
                        if use_bracket_level {
                            write!(writer, "[{}]", level)?
                        } else {
                            write!(writer, "{}", level)?
                        }
                    }
                    "file" => write!(writer, "{}", file)?,
                    "module" => write!(writer, "{}", module)?,
                    "message" => write!(writer, "{}", message)?,
                    _ => write!(writer, "{}", placeholder)?,
                }

                if is_terminal && config.enable_colors {
                    if let Some(writer) = (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>()
                    {
                        writer.reset()?;
                    }
                }

                last_end = end + 1; // Update the last_end to the character after the closing bracket
            }
        }
    }

    // Write any remaining part of the format_str after the last match
    if last_end < format_str.len() {
        write!(writer, "{}", &format_str[last_end..])?;
    }

    Ok(())
}
