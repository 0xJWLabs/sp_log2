use crate::config::{Format, TargetPadding, TimeFormat};
use crate::{Config, LevelPadding, ThreadLogMode, ThreadPadding};
use log::Record;
use regex::Regex;
use std::any::Any;
use std::io::{Error, Write};
use std::thread;
use termcolor::{BufferedStandardStream, Color, ColorSpec, WriteColor};

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

#[inline(always)]
pub fn write_target(record: &Record<'_>, config: &Config) -> Result<String, Error> {
    let target = match config.target_padding {
        TargetPadding::Left(pad) => format!("{:>pad$}", record.target(), pad = pad),
        TargetPadding::Right(pad) => format!("{:<pad$}", record.target(), pad = pad),
        TargetPadding::Off => record.target().to_string(),
    };
    Ok(target)
}

#[inline(always)]
pub fn write_location(record: &Record<'_>) -> Result<String, Error> {
    let file = record.file().unwrap_or("<unknown>");
    let location = if let Some(line) = record.line() {
        format!("{}:{}", file, line)
    } else {
        format!("{}:<unknown>", file)
    };
    Ok(location)
}

#[inline(always)]
pub fn write_module(record: &Record<'_>) -> Result<String, Error> {
    let module = record.module_path().unwrap_or("<unknown>");

    Ok(module.to_string())
}

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

#[inline(always)]
#[allow(unused_variables)]
pub fn write_args(record: &Record<'_>, line_ending: &str) -> Result<String, Error> {
    Ok(format!("{}{}", record.args(), line_ending))
}

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

fn apply_style(style: &str) -> Option<Color> {
    if style.starts_with("rgb(") {
        let rgb_re = Regex::new(r"rgb\((\d+),\s*(\d+),\s*(\d+)\)").unwrap();
        if let Some(caps) = rgb_re.captures(style) {
            let r: u8 = caps[1].parse().unwrap_or(255);
            let g: u8 = caps[2].parse().unwrap_or(255);
            let b: u8 = caps[3].parse().unwrap_or(255);

            return Some(Color::Rgb(r, g, b));
        }
    }

    None
}

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
    let re = Regex::new(r"\{(\w+)(?::([^:}]+))?(?::([^}]+))?\}").unwrap();
    let mut last_end = 0; // Tracks the position of the last match's end

    // Iterate over each match found by the regex
    for caps in re.captures_iter(format_str.as_str()) {
        // Write the part of the format string before the match
        if last_end < caps.get(0).unwrap().start() {
            write!(
                writer,
                "{}",
                &format_str[last_end..caps.get(0).unwrap().start()]
            )?;
        }

        let key = &caps[1];
        let style = caps.get(2).map_or("", |m| m.as_str());
        let level_part = caps.get(3).map_or("", |m| m.as_str());

        let use_bracket_level = if key == "level" {
            match (style, level_part) {
                // Case 1: level:nb
                ("nb", "") | ("", "nb") => false, // No brackets around the level
                // Case 2: level:color:nb
                (_, "nb") => false, // No brackets around the level
                // Case 3: level (default, with brackets)
                _ => true, // Default case, brackets around the level
            }
        } else {
            true // For all non-level keys, we assume brackets are used
        };

        let color = if key == "level" {
            match (style, level_part) {
                ("nb", "") | ("", "nb") => level_color,
                (_, "nb") => apply_style(style).or(level_color),
                _ => apply_style(style).or(level_color),
            }
        } else {
            apply_style(style)
        };

        if is_terminal && !config.write_log_enable_colors {
            if let Some(writer) = (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>()
            {
                writer.set_color(ColorSpec::new().set_fg(color))?; // This works only for WriteColor types
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
            _ => write!(writer, "{}", &caps[0])?,
        }

        if is_terminal && !config.write_log_enable_colors {
            if let Some(writer) = (writer as &mut dyn Any).downcast_mut::<BufferedStandardStream>()
            {
                writer.reset()?;
            }
        }

        // Update the last_end to the end position of the current match
        last_end = caps.get(0).unwrap().end();
    }

    // Write any remaining part of the format_str after the last match
    if last_end < format_str.len() {
        write!(writer, "{}", &format_str[last_end..])?;
    }

    Ok(())
}
