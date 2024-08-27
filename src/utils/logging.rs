use std::time::SystemTime;

use fern::colors::{Color, ColoredLevelConfig};
use log::debug;

pub fn setup_logger() {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let colors_level = colors_line.debug(Color::Magenta);

    // Define colors for each part of the log message
    let color_date = Color::Green;
    let color_level = Color::Cyan;
    let color_target = Color::Blue;
    let color_message = Color::White;

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{color_date}{date}\x1B[0m {color_level}{level}\x1B[0m {color_target}{target}\x1B[0m {color_line}] {color_message}{message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                color_date = format_args!("\x1B[{}m", color_date.to_fg_str()),
                color_level = format_args!("\x1B[{}m", color_level.to_fg_str()),
                color_target = format_args!("\x1B[{}m", color_target.to_fg_str()),
                color_message = format_args!("\x1B[{}m", color_message.to_fg_str()),
                date = humantime::format_rfc3339_seconds(SystemTime::now()),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        .level(log::LevelFilter::Debug)
        .level_for("pretty_colored", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    debug!("finished setting up logging! yay!");
}