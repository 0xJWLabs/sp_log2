use log::*;
use sp_log2::*;

fn main() {
    let mut config_builder = ConfigBuilder::new();
    config_builder.set_format(Format::LevelFlag | Format::Time | Format::Thread | Format::Target | Format::FileLocation);
    config_builder.set_formatter(Some(
        "{time:#89dceb} {level} ({thread}) {target:rgb(137, 180, 250)}: {message} [{file:#89b4fa}]\n",
    ));
    config_builder.set_time_format_custom(format_description!(
        "[day]/[month]/[year] [hour]:[minute]:[second],[subsecond digits:3]"
    ));
    let config = config_builder.build();
    TermLogger::init(
        LevelFilter::Trace,
        config,
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .unwrap();
    error!("Red error");
    warn!("Yellow warning");
    info!("Blue info");
    debug!("Cyan debug");
    trace!("White trace");
}
