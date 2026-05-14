//! The `jukctl` library with all the utilities to make the program work.

pub mod cli;
pub mod logger;

mod gcode;
mod svg;
mod transport;

use cli::Args;
use svg::Svg;

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

    Err(Error::Infallible)
}
