//! The `jukctl` library with all the utilities to make the program work.

pub mod cli;

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

/// Run the program with the given `args`.
pub fn run(args: cli::Args) -> crate::Result<()> {
    todo!()
}
