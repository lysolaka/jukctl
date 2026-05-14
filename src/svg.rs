//! SVG to G-code conversion.

use std::fs;
use std::path::Path;

use g_code::emit::FormatOptions;
use g_code::emit::format_gcode_fmt;

use roxmltree::Document;

use svg2gcode::ConversionConfig;
use svg2gcode::ConversionOptions;
use svg2gcode::Machine;
use svg2gcode::SupportedFunctionality;
use svg2gcode::svg2program;

use svgtypes::Length;
use svgtypes::LengthUnit;

/// SVG file read into memory.
pub struct Svg {
    contents: String,
}

impl Svg {
    /// Default paper width.
    const PAPER_WIDTH: Length = Length {
        number: 297.0 - 30.0, // some margin space
        unit: LengthUnit::Mm,
    };

    /// Default paper height.
    const PAPER_HEIGHT: Length = Length {
        number: 210.0 - 30.0, // some margin space
        unit: LengthUnit::Mm,
    };

    /// Open the file at `path` and read its contents into memory.
    ///
    /// This function does not really check if the file is actually an SVG file.
    pub fn open<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        log::debug!("Opening file: {}", path.as_ref().display());
        let contents = fs::read_to_string(path)?;
        Ok(Self { contents })
    }

    /// Take the SVG contents and emit G-code.
    pub fn emit_gcode(self) -> crate::Result<String> {
        log::debug!("Parsing the SVG file");
        let doc = Document::parse(&self.contents)?;

        let config = ConversionConfig {
            tolerance: 0.1,
            feedrate: 10.0,
            dpi: 96.0,
            origin: [Some(0.0); 2],
            extra_attribute_name: None,
        };

        let options = ConversionOptions {
            dimensions: [Some(Self::PAPER_WIDTH), Some(Self::PAPER_HEIGHT)],
        };

        let functionality = SupportedFunctionality {
            circular_interpolation: true,
        };
        let machine = Machine::new(functionality, None, None, None, None);

        log::debug!("Converting to G-code");
        let program = svg2program(&doc, &config, options, machine);

        log::debug!("Emitting G-code");
        let fmt_options = FormatOptions {
            checksums: false,
            line_numbers: false,
            delimit_with_percent: false,
            newline_before_comment: true,
        };
        let mut gcode = String::with_capacity(program.len() * 8); // length assumption
        format_gcode_fmt(program, fmt_options, &mut gcode)?;
        Ok(gcode)
    }
}
