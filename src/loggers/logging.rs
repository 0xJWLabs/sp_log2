use crate::config::{Format, TargetPadding, TimeFormat};
use crate::{Config, LevelPadding, ThreadLogMode, ThreadPadding};
use chrono::DateTime;
use log::Record;
use std::any::Any;
use std::io::{Error, Write};
use std::str::FromStr;
use std::thread;
use termcolor2::{BufferedStandardStream, Color, ColorSpec, WriteColor};

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
    use chrono::Local;

    let dt: DateTime<Local> = Local::now();

    let formatted_time = match config.time_format.clone() {
        TimeFormat::Rfc2822 => dt.to_rfc2822(),
        TimeFormat::Rfc3339 => dt.to_rfc3339(),
        TimeFormat::Custom(format) => dt.format(format).to_string(),
    };

    Ok(formatted_time)
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
fn apply_style(style: &str) -> Option<(Color, bool)> {
    let is_bg = style.starts_with("bg");
    let new_style = match is_bg {
        true => &style[2..],
        false => style,
    };

    if let Ok(color) = Color::from_str(new_style) {
        return Some((color, !is_bg));
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
    let mut chars = format_str.chars().enumerate().peekable(); // To look ahead for brackets

    while let Some((i, c)) = chars.next() {
        if c == '[' {
            // Check for double brackets `[[`
            if let Some((_, next_c)) = chars.peek() {
                if *next_c == '[' {
                    chars.next(); // Consume the second `[`

                    // Find the closing double brackets `]]`
                    if let Some(end) = format_str[i + 2..].find("]]") {
                        let end = i + 2 + end;

                        // Write the part before the placeholder
                        if last_end < i {
                            write!(writer, "{}", &format_str[last_end..i])?;
                        }

                        // Include the brackets in the output by simply writing them
                        write!(writer, "[")?;
                        let placeholder = &format_str[i + 2..end];
                        process_placeholder(
                            writer,
                            placeholder,
                            level_color.clone(),
                            config,
                            level,
                            time,
                            thread,
                            target,
                            file,
                            module,
                            message,
                            is_terminal,
                        )?;
                        write!(writer, "]")?;

                        last_end = end + 2; // Update last_end to the character after `]]`
                        continue;
                    }
                }
            }

            // Handle single brackets `[`
            if let Some(end) = format_str[i + 1..].find(']') {
                let end = i + 1 + end;

                // Write the part before the placeholder
                if last_end < i {
                    write!(writer, "{}", &format_str[last_end..i])?;
                }

                let placeholder = &format_str[i + 1..end];
                process_placeholder(
                    writer,
                    placeholder,
                    level_color.clone(),
                    config,
                    level,
                    time,
                    thread,
                    target,
                    file,
                    module,
                    message,
                    is_terminal,
                )?;

                last_end = end + 1; // Update last_end to the character after `]`
            }
        }
    }

    // Write any remaining part of the format_str after the last match
    if last_end < format_str.len() {
        write!(writer, "{}", &format_str[last_end..])?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_placeholder<W>(
    writer: &mut W,
    placeholder: &str,
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
    let parts: Vec<&str> = placeholder.split(':').collect();
    let key = parts[0];

    let mut use_bracket_level = true;

    if is_terminal {
        let styles = if parts.len() > 1 {
            parts[1..].to_vec()
        } else {
            vec![]
        };

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
            fg_color = fg_color.or(level_color.clone());
        }

        if config.enable_colors {
            if let Some(writer) = (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>()
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
        if let Some(writer) = (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>() {
            writer.reset()?;
        }
    }

    Ok(())
}

// #[allow(clippy::too_many_arguments)]
// fn parse_and_format_log_internal<W>(
//     writer: &mut W,
//     level_color: Option<Color>,
//     config: &Config,
//     level: &str,
//     time: &str,
//     thread: &str,
//     target: &str,
//     file: &str,
//     module: &str,
//     message: &str,
//     is_terminal: bool,
// ) -> Result<(), Error>
// where
//     W: Write + Sized + Any,
// {
//     let format_str = config.formatter.clone().unwrap();
//     let mut last_end = 0;
//
//     for (i, c) in format_str.char_indices() {
//     if c == '[' {
//         let mut closing_bracket = ']';
//         let mut start_idx = i + 1;
//
//         // Detect double-brackets for literal brackets
//         if format_str[start_idx..].starts_with('[') {
//             closing_bracket = ']'; // Double brackets use a single closing bracket
//             start_idx += 1;       // Adjust start index
//         }
//
//         // Find the closing bracket
//         if let Some(end_idx) = format_str[start_idx..].find(closing_bracket) {
//             let end_idx = start_idx + end_idx;
//
//             // Write the part before the placeholder
//             if last_end < i {
//                 write!(writer, "{}", &format_str[last_end..i])?;
//             }
//
//             // Extract the placeholder content
//             let placeholder = &format_str[start_idx..end_idx];
//             let parts: Vec<&str> = placeholder.split(':').collect();
//             let key = parts[0];
//
//             // Extract styles (if any)
//             let style = parts.get(1).cloned();
//
//             // Apply styles if terminal supports it
//             if is_terminal && config.enable_colors {
//                 if let Some(writer) = (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>()
//                 {
//                     apply_style(writer, style)?;
//                 }
//             }
//
//             // Write the resolved placeholder value
//             let value = match key {
//                 "time" => time,
//                 "thread" => thread,
//                 "target" => target,
//                 "level" => level,
//                 "file" => file,
//                 "message" => message,
//                 _ => key, // Unknown placeholders are treated as literal keys
//             };
//
//             if closing_bracket == ']' && placeholder.starts_with('[') {
//                 // Double brackets -> wrap output in brackets
//                 write!(writer, "[{}]", value)?;
//             } else {
//                 // Single brackets -> raw output
//                 write!(writer, "{}", value)?;
//             }
//
//             if is_terminal && config.enable_colors {
//                 if let Some(writer) = (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>()
//                 {
//                     writer.reset()?; // Reset styles
//                 }
//             }
//
//             last_end = end_idx + 1; // Update last_end
//         }
//     }
// }
//
// // Write remaining text after the last placeholder
// if last_end < format_str.len() {
//     write!(writer, "{}", &format_str[last_end..])?;
// }
//
//
//     Ok(())
// }
