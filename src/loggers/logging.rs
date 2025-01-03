use crate::config::{Format, TargetPadding, TimeFormat};
use crate::{Config, LevelPadding, ThreadLogMode, ThreadPadding};
use log::Record;
use std::io::{Error, Write};
use std::thread;
#[cfg(all(feature = "termcolor", feature = "ansi_term"))]
use termcolor::Color;

#[cfg(all(feature = "termcolor", feature = "ansi_term"))]
pub fn termcolor_to_ansiterm(color: &Color) -> Option<ansi_term::Color> {
    match color {
        Color::Black => Some(ansi_term::Color::Black),
        Color::Red => Some(ansi_term::Color::Red),
        Color::Green => Some(ansi_term::Color::Green),
        Color::Yellow => Some(ansi_term::Color::Yellow),
        Color::Blue => Some(ansi_term::Color::Blue),
        Color::Magenta => Some(ansi_term::Color::Purple),
        Color::Cyan => Some(ansi_term::Color::Cyan),
        Color::White => Some(ansi_term::Color::White),
        _ => None,
    }
}

#[inline(always)]
pub fn try_log<W>(config: &Config, record: &Record<'_>, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
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

    if config.format & Format::Location != 0 {
        location = write_location(record)?;
    }

    if config.format & Format::Module != 0 {
        module = write_module(record)?;
    }

    let args = write_args(record, &config.line_ending)?;

    let mut r = String::with_capacity(
        level.len()
            + time.len()
            + thread.len()
            + target.len()
            + location.len()
            + module.len()
            + args.len()
            + 4,
    );
    if !time.is_empty() {
        r.push_str(&time);
    }

    if !level.is_empty() {
        r.push(' ');
        r.push_str(&level);
    }

    if !thread.is_empty() {
        r.push(' ');
        r.push_str(&thread);
    }

    if !target.is_empty() {
        r.push(' ');
        r.push_str(&target);
    }

    if !module.is_empty() {
        r.push(' ');

        r.push_str(&module);
    }

    r.push(' ');

    r.push_str(&args);

    if !location.is_empty() {
        r.push(' ');
        r.push_str(&location);
    }

    write!(write, "{}", r)?;

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
    #[cfg(all(feature = "termcolor", feature = "ansi_term"))]
    let color = match &config.level_color[record.level() as usize] {
        Some(termcolor) => {
            if config.write_log_enable_colors {
                termcolor_to_ansiterm(termcolor)
            } else {
                None
            }
        }
        None => None,
    };

    let level = match config.level_padding {
        LevelPadding::Left => format!("[{: >5}]", record.level()),
        LevelPadding::Right => format!("[{: <5}]", record.level()),
        LevelPadding::Off => format!("[{}]", record.level()),
    };

    #[cfg(all(feature = "termcolor", feature = "ansi_term"))]
    let formatted_level = match color {
        Some(c) => format!("{}", c.paint(level)),
        None => level.to_string(),
    };

    #[cfg(not(feature = "ansi_term"))]
    let formatted_level = level.to_string();

    Ok(formatted_level)
}

#[inline(always)]
pub fn write_target(record: &Record<'_>, config: &Config) -> Result<String, Error> {
    let target = match config.target_padding {
        TargetPadding::Left(pad) => format!("{:>pad$}:", record.target(), pad = pad),
        TargetPadding::Right(pad) => format!("{:<pad$}:", record.target(), pad = pad),
        TargetPadding::Off => format!("{}:", record.target()),
    };
    Ok(target)
}

#[inline(always)]
pub fn write_location(record: &Record<'_>) -> Result<String, Error> {
    let file = record.file().unwrap_or("<unknown>");
    let location = if let Some(line) = record.line() {
        format!("[{}:{}]", file, line)
    } else {
        format!("[{}:<unknown>]", file)
    };
    Ok(location)
}

#[inline(always)]
pub fn write_module(record: &Record<'_>) -> Result<String, Error> {
    let module = record.module_path().unwrap_or("<unknown>");
    Ok(format!("[{}]", module))
}

pub fn write_thread_name(config: &Config) -> Result<String, Error> {
    if let Some(name) = thread::current().name() {
        let thread_name = match config.thread_padding {
            ThreadPadding::Left { 0: qty } => {
                format!("({:>width$})", name, width = qty)
            }
            ThreadPadding::Right { 0: qty } => {
                format!("({:<width$})", name, width = qty)
            }
            ThreadPadding::Off => format!("({})", name),
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
            format!("({:>width$})", id, width = qty)
        }
        ThreadPadding::Right { 0: qty } => {
            format!("({:<width$})", id, width = qty)
        }
        ThreadPadding::Off => format!("({})", id),
    };
    Ok(thread_id)
}

#[inline(always)]
#[cfg(feature = "paris")]
pub fn write_args(
    record: &Record<'_>,
    with_colors: bool,
    line_ending: &str,
) -> Result<String, Error> {
    let formatted_args = crate::__private::paris::formatter::format_string(
        format!("{}", record.args()),
        with_colors,
    );
    Ok(format!("{}{}", formatted_args, line_ending))
}

#[inline(always)]
#[cfg(not(feature = "paris"))]
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
