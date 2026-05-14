//! The `jukctl` library with all the utilities to make the program work.

pub mod cli;
pub mod logger;

use logger::Logger;

/// Result type, equivalent to [`std::result::Result<T, Error>`].
///
/// Do not import this type - using [`jukctl::Result<T>`] or [`crate::Result<T>`] is much more
/// cleaner.
pub type Result<T> = std::result::Result<T, Error>;

/// Common error type for all the kinds of errors in this program
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("This is just a placeholder")]
    Infallible,
}

use indicatif::ProgressBar;
use indicatif::ProgressStyle;

/// Run the program with the given `args`.
pub fn run(args: cli::Args) -> crate::Result<()> {
    log::trace!("Trace test");
    log::debug!("Debug test");
    log::info!("Got args: {:?}", args);
    log::warn!("Warning test");
    log::error!("Error test");

    let pb = ProgressBar::new(4000);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] [{bar:55}] ({pos}/{len})")
            .expect("the progress template should always work")
            .progress_chars("=> "),
    );

    let pb = Logger::get().install(pb);

    for i in 0..4000 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        if i % 1000 == 0 {
            log::info!("div by 1k");
        }
        pb.inc(1);
    }
    pb.finish();

    std::thread::sleep(std::time::Duration::from_secs(5));
    todo!()
}
