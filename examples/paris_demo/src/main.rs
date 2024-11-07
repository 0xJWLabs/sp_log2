fn main() {
    sp_log::TermLogger::init(sp_log::LevelFilter::Debug,
                                sp_log::Config::default(),
                                sp_log::TerminalMode::Mixed,
                                sp_log::ColorChoice::Auto).expect("Failed to start sp_log");

    sp_log::info!("I can write <b>bold</b> text or use tags to <red>color it</>");
}
