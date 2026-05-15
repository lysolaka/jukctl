//! The `jukctl` library with all the utilities to make the program work.

pub mod cli;
pub mod logger;

mod gcode;
mod svg;
mod transport;

use juk_cmd::cmd::Command;

use cli::Args;
use svg::Svg;
use transport::Interface;

/// Result type, equivalent to [`std::result::Result<T, Error>`].
///
/// Do not import this type - using [`jukctl::Result<T>`] or [`crate::Result<T>`] is much more
/// cleaner.
pub type Result<T> = std::result::Result<T, Error>;

/// Common error type for all the kinds of errors in this program
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Formatting error")]
    FmtError(#[from] std::fmt::Error),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Data serialization / deserialization error")]
    SerdeError(#[from] postcard::Error),
    #[error("Serial communication error")]
    SerialError(#[from] serialport::Error),
    #[error("SVG parsing error")]
    SvgError(#[from] roxmltree::Error),
    #[error("This error should not happen")]
    Infallible,
}

/// A macro used to return on error with a custom message printed by the [`log`] crate.
macro_rules! bail {
    ($e:expr, $s:literal) => {
        match $e {
            Ok(o) => o,
            Err(e) => {
                log::error!($s);
                return Err(e);
            }
        }
    };
}

/// Run the program with the given `args`.
pub fn run(args: Args) -> crate::Result<()> {
    log::debug!("Running with: {:?}", args);

    let svg = bail!(Svg::open(&args.file), "Can't open the SVG file");
    let gcode = bail!(svg.emit_gcode(), "Failed to parse or emit the G-code");

    println!("{}", gcode);

    log::info!("Opening the serial port, make sure the plotter is in binary mode");
    let mut interface = bail!(
        Interface::open(&args.port),
        "Failed to connect to the plotter"
    );

    let commands = [
        Command::ConfigGet {
            key: "".to_string(),
        },
        Command::ConfigSet {
            kv: vec![("k".to_string(), "v".to_string())],
        },
        Command::Cancel,
        Command::Home {
            x: true,
            y: true,
            z: false,
        },
    ];

    for cmd in commands {
        log::info!("Sent: {:?}", cmd);
        let resp = interface.transaction(&cmd)?;
        log::info!("Got : {:?}", resp);
    }

    Err(Error::Infallible)
}
