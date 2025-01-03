use log::*;
use sp_log2::*;

fn main() {
    let mut config_builder = ConfigBuilder::new();
    config_builder.set_format(Format::LevelFlag | Format::Time | Format::Thread | Format::Target | Format::FileLocation);
    config_builder.set_formatter(Some("{time:rgb(147, 153, 178)} {level} ({thread}) {target:#eee:bold}: {message} [{file}]\n"));
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
