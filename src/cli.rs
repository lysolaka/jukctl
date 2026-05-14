//! Command line arguments parsing

use std::path::PathBuf;

use anstyle::AnsiColor;
use anstyle::Style;

use clap::ArgAction;
use clap::builder::styling::Styles;

use log::LevelFilter;

const HEADER: Style = AnsiColor::Green.on_default().bold();
const USAGE: Style = AnsiColor::Green.on_default().bold();
const LITERAL: Style = AnsiColor::White.on_default().bold();
const PLACEHOLDER: Style = AnsiColor::Magenta.on_default();
const ERROR: Style = AnsiColor::BrightRed.on_default().bold();
const VALID: Style = AnsiColor::White.on_default().bold();
const INVALID: Style = AnsiColor::Yellow.on_default().bold();

const CLI_STYLE: Styles = Styles::styled()
    .header(HEADER)
    .usage(USAGE)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(ERROR)
    .valid(VALID)
    .invalid(INVALID);

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
#[command(styles = CLI_STYLE)]
pub struct Args {
    /// Port the plotter is connected to.
    ///
    /// This will usually be `/dev/ttyUSB0` on Linux, or `COM5` on Windows.
    #[arg(short, long, value_name = "PORT", required = true)]
    pub port: String,
    /// Perform homing before plotting.
    ///
    /// In case this is not set, the program expects that the pen is touching the paper already.
    #[arg(short = 'H', long)]
    pub homing: bool,
    /// SVG file to plot from.
    ///
    /// The image size will be scaled automatically to A4 size by the program.
    #[arg(value_name = "FILE", required = true)]
    pub file: PathBuf,
    #[command(flatten)]
    pub verbosity: Verbosity,
}

#[derive(Debug, clap::Args)]
pub struct Verbosity {
    /// Increase logging verbosity.
    #[arg(
        short,
        long,
        action = ArgAction::Count,
        display_order = 1
    )]
    verbose: u8,
    /// Decrease logging verbosity.
    #[arg(
        short,
        long,
        action = ArgAction::Count,
        display_order = 1
    )]
    quiet: u8,
}

impl Verbosity {
    /// Get the filter that should be applied to the logger.
    pub fn filter(&self) -> LevelFilter {
        let mut filter = LevelFilter::Info;
        for _ in 0..self.verbose {
            filter = filter.increment_severity();
        }
        for _ in 0..self.quiet {
            filter = filter.decrement_severity();
        }
        filter
    }
}
