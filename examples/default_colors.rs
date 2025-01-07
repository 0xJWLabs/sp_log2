use log::*;
use sp_log2::*;

fn main() {
    let mut config_builder = ConfigBuilder::new();
    config_builder.set_format(
        Format::LevelFlag | Format::Time | Format::Thread | Format::Target | Format::FileLocation,
    );
    // config_builder.set_formatter(Some(
    //     "{time:#89dceb} {level} ({thread}) {target:rgb(137, 180, 250):bold}: {message} [{file:#eba0ac}]\n",
    // ));
    config_builder.set_formatter(Some(
    "[time:#89dceb] [level] ([thread]) target: [target:rgb(137 180 250):bold]: [message] [[file:#6c7086]]\n",
));
    config_builder.set_time_format_custom("%d/%m/%Y %H:%M:%S,%3f");
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
