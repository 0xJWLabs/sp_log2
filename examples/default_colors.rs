#[cfg(feature = "termcolor")]
use log::*;
#[cfg(feature = "termcolor")]
use sp_log2::*;

#[cfg(feature = "termcolor")]
fn main() {

    let mut config_builder = ConfigBuilder::new();
    config_builder.set_format(Format::LevelFlag | Format::Time | Format::FileName | Format::Thread | Format::Target | Format::Location);
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

#[cfg(not(feature = "termcolor"))]
fn main() {
    println!("this example requires the termcolor feature.");
}
