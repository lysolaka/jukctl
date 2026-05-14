//! Logger implementation.

use anstyle::AnsiColor;
use anstyle::Style;

use indicatif::MultiProgress;
use indicatif::ProgressBar;

use log::Level;
use log::LevelFilter;
use log::Log;
use log::Metadata;
use log::Record;

use once_cell::sync::OnceCell;

/// The global logger instance
static LOGGER: OnceCell<Logger> = OnceCell::new();

/// The logger definition
#[derive(Debug)]
pub struct Logger {
    level: LevelFilter,
    bar: MultiProgress,
}

impl Logger {
    /// Initialize the logger with log level set to `level`.
    ///
    /// # Panics
    ///
    /// 1. if called more than once,
    /// 2. if registering the logger with the [`log`] crate fails.
    pub fn init(level: LevelFilter) {
        let logger = Logger {
            level,
            bar: MultiProgress::new(),
        };

        LOGGER
            .set(logger)
            .expect("called `Logger::init()` more than once");

        let logger = LOGGER.get().expect("the logger should be set");

        log::set_logger(logger).expect("failed to initialize the logger");
        log::set_max_level(logger.level);
    }

    /// Get a reference to the logger instance.
    ///
    /// # Panics
    ///
    /// If the function is called before [`Logger::init()`]
    pub fn get() -> &'static Logger {
        LOGGER
            .get()
            .expect("called `Logger::get()`, but the logger was not initialized")
    }

    /// Get a clone of the progress bar controller ([`MultiProgress`]).
    ///
    /// Using it to install progress bars avoids log messages messing up the bars.
    pub fn multi(&self) -> MultiProgress {
        self.bar.clone()
    }

    /// Install a [`ProgressBar`] for use in a way which does not mess it up.
    pub fn install(&self, pb: ProgressBar) -> ProgressBar {
        self.bar.add(pb)
    }

    /// Return the indicator colour for a message with the given `level`.
    const fn color(level: Level) -> Style {
        match level {
            Level::Error => AnsiColor::Red.on_default().bold(),
            Level::Warn => AnsiColor::Yellow.on_default().bold(),
            Level::Info => AnsiColor::Green.on_default().bold(),
            Level::Debug => AnsiColor::Blue.on_default().bold(),
            Level::Trace => AnsiColor::White.on_default().bold(),
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        self.bar.suspend(|| {
            if record.level() <= self.level {
                eprintln!(
                    "{0}*{0:#} {1}",
                    Logger::color(record.level()),
                    record.args()
                );
            }
        })
    }

    fn flush(&self) {}
}
