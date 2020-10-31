use simplelog::{TermLogger, LevelFilter, Config, TerminalMode, ConfigBuilder};

pub(crate) fn logging() {
    let logger_config = ConfigBuilder::new()
        .set_time_format_str("%H:%M:%S:%6f")
        .build();
    TermLogger::init(LevelFilter::Info, logger_config, TerminalMode::Mixed);
}
