//! The `jukctl` library with all the utilities to make the program work.

pub mod cli;
pub mod logger;

mod gcode;
mod svg;
mod transport;

use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use juk_cmd::cmd::Command;
use juk_cmd::cmd::Response;
use juk_cmd::config::Frame;

use cli::Args;
use gcode::to_sequence;
use logger::Logger;
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
    #[error("The plotter is not in absolute coordinates mode")]
    FrameNotAbs,
    #[error("Unexpected `svg2gcode` output")]
    UnexpectedGcode,
    #[error("Unexpected response: `{0:?}`")]
    UnexpectedResponse(Response),

    #[error("Formatting error")]
    FmtError(#[from] std::fmt::Error),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Motion error")]
    MotionError(#[from] juk_cmd::MotionError),
    #[error("G-code parsing error")]
    ParseError(#[from] juk_cmd::ParseError),
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

    log::info!("Opening the serial port, make sure the plotter is in binary mode");
    let mut interface = bail!(
        Interface::open(&args.port),
        "Failed to connect to the plotter"
    );

    log::info!("Getting the system configuration");
    let syscfg = match bail!(
        interface.transaction(&Command::ConfigGet {
            key: "".to_string(),
        }),
        "Failed to obtain the system configuration"
    ) {
        Response::Config(c) => c,
        r => return Err(Error::UnexpectedResponse(r)),
    };

    log::info!("");
    log::info!("System configuration:");
    log::info!("`accel` = {}", syscfg.accel);
    log::info!("`vel` = {}", syscfg.vel);
    match syscfg.frame {
        Frame::Absolute => log::info!("`frame` = abs"),
        Frame::Relative => log::info!("`frame` = rel"),
    }
    log::info!("`mmpsX` = {}", syscfg.mmps.0);
    log::info!("`mmpsY` = {}", syscfg.mmps.1);
    log::info!("`mmpsZ` = {}", syscfg.mmps.2);
    log::info!("");

    log::info!("Converting G-code to movement sequence");
    let seq = bail!(to_sequence(&gcode, &syscfg), "Failed to parse G-code");

    if args.homing {
        log::info!("Homing is enabled, starting the homing procedure");
        let response = interface.transaction(&Command::Home {
            x: true,
            y: true,
            z: true,
        })?;

        match response {
            Response::Ok => log::debug!("Homing complete"),
            Response::Err(e) => {
                log::error!("Homing failed: got a Response::Err");
                return Err(e.into());
            }
            r => return Err(Error::UnexpectedResponse(r)),
        }
    }

    let pb = ProgressBar::new(seq.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] [{bar:55}] ({pos}/{len})")
            .expect("the progress template should always work")
            .progress_chars("=> "),
    );

    let pb = Logger::get().install(pb);

    for cmd in seq.iter() {
        log::trace!("Executing: {:?}", cmd);
        match interface.transaction(cmd)? {
            Response::Ok => pb.inc(1),
            Response::Err(e) => {
                pb.finish();
                log::error!("Execution failed: got a Response::Err");
                return Err(e.into());
            }
            r => {
                pb.finish();
                return Err(Error::UnexpectedResponse(r));
            }
        }
    }
    pb.finish();

    Ok(())
}
