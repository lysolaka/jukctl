//! G-code to [`Command`] conversion.

use std::str::FromStr;

use juk_cmd::cmd::ArcDir;
use juk_cmd::cmd::Axis;
use juk_cmd::cmd::Command;
use juk_cmd::cmd::Displacement;
use juk_cmd::config::Frame;
use juk_cmd::config::SystemConfig;

use crate::Error;

/// Custom acceleration for rapid movement (G0).
const RAPID_ACCEL: f32 = 25_000.0;
/// Custom velocity for rapid movement (G0).
const RAPID_VEL: f32 = 18_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Code {
    MoveRapid,
    Move,
    ArcPos,
    ArcNeg,
}

peg::parser! {
    grammar parser() for str {
        rule code() -> Code = c:$("G" ['0'..='3']) {
            match c {
                "G0" => Code::MoveRapid,
                "G1" => Code::Move,
                "G2" => Code::ArcNeg,
                "G3" => Code::ArcPos,
                _ => unreachable!(),
            }
        }

        rule param_key() -> char = k:['X' | 'Y' | 'R' | 'F'] {
            k
        }

        rule param_val() -> f32 = v:$("-"*<,1> ['0'..='9']+ ( "." ['0'..='9']+ )?) {?
            // parse to f64 since it's the data type in G-code
            let v = f64::from_str(v).or(Err("f64"))?;
            Ok(v as f32)
        }

        rule param() -> (char, f32) = k:param_key() v:param_val() {
            (k, v)
        }

        /// Very simple G-code parser with support for G0, G1, G2, G3 only. Everything else is an
        /// error, even comments.
        pub rule parse() -> (Code, Vec<(char, f32)>) = code:code() " " params:(param() **<1,6> " ") {
            (code, params)
        }
    }
}

/// Extract movement parameters from `params` and apply them to `code` producing a ready [`Command`] for
/// use with the plotter.
fn to_command(
    code: Code,
    params: Vec<(char, f32)>,
    syscfg: &SystemConfig,
    pos: &mut (f32, f32),
) -> crate::Result<Command> {
    match code {
        Code::MoveRapid => {
            let mut x = 0.0;
            let mut y = 0.0;

            for (k, v) in params {
                match k {
                    'X' => x = v,
                    'Y' => y = v,
                    _ => (),
                }
            }

            let dx = Displacement::from_mm(x - pos.0, Axis::X, syscfg)?;
            let dy = Displacement::from_mm(y - pos.1, Axis::Y, syscfg)?;

            pos.0 = x;
            pos.1 = y;

            Ok(Command::Move {
                x: dx,
                y: dy,
                z: Displacement::Relative(0),
                a: RAPID_ACCEL,
                v: RAPID_VEL,
            })
        }
        Code::Move => {
            let mut x = 0.0;
            let mut y = 0.0;

            for (k, v) in params {
                match k {
                    'X' => x = v,
                    'Y' => y = v,
                    _ => (),
                }
            }

            let dx = Displacement::from_mm(x - pos.0, Axis::X, syscfg)?;
            let dy = Displacement::from_mm(y - pos.1, Axis::Y, syscfg)?;

            pos.0 = x;
            pos.1 = y;

            Ok(Command::Move {
                x: dx,
                y: dy,
                z: Displacement::Relative(0),
                a: syscfg.accel,
                v: syscfg.vel,
            })
        }
        Code::ArcPos => {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut r = 0;

            for (k, v) in params {
                match k {
                    'X' => x = v,
                    'Y' => y = v,
                    'R' => r = ((2.0 * v) / (syscfg.mmps.0 + syscfg.mmps.1)).round() as u32,
                    _ => (),
                }
            }

            let dx = Displacement::from_mm(x - pos.0, Axis::X, syscfg)?;
            let dy = Displacement::from_mm(y - pos.1, Axis::Y, syscfg)?;

            pos.0 = x;
            pos.1 = y;

            Ok(Command::Arc {
                x: dx,
                y: dy,
                z: Displacement::Relative(0),
                r,
                dir: ArcDir::Pos,
                a: syscfg.accel,
                v: syscfg.vel,
            })
        }
        Code::ArcNeg => {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut r = 0;

            for (k, v) in params {
                match k {
                    'X' => x = v,
                    'Y' => y = v,
                    'R' => r = ((2.0 * v) / (syscfg.mmps.0 + syscfg.mmps.1)).round() as u32,
                    _ => (),
                }
            }

            let dx = Displacement::from_mm(x - pos.0, Axis::X, syscfg)?;
            let dy = Displacement::from_mm(y - pos.1, Axis::Y, syscfg)?;

            pos.0 = x;
            pos.1 = y;

            Ok(Command::Arc {
                x: dx,
                y: dy,
                z: Displacement::Relative(0),
                r,
                dir: ArcDir::Neg,
                a: syscfg.accel,
                v: syscfg.vel,
            })
        }
    }
}

/// Parse G-code into a sequence of movement [`Command`]s.
pub fn to_sequence(gcode_doc: &str, syscfg: &SystemConfig) -> crate::Result<Vec<Command>> {
    log::debug!("Parsing G-code");
    // make our copy of syscfg to ensure the relative frame
    let syscfg = SystemConfig {
        frame: Frame::Relative,
        ..*syscfg
    };

    let s = gcode_doc.lines().count();
    log::debug!("Reserving space for {} commands", s);
    let mut sequence = Vec::with_capacity(s);

    let mut pos = (0.0, 0.0);

    for line in gcode_doc.lines().filter(|l| line_filter(*l)) {
        let (code, params) = match parser::parse(line) {
            Ok(o) => (o.0, o.1),
            Err(e) => {
                log::error!("Failed to parse: `{}`", line);
                log::error!("{}", e);
                return Err(Error::UnexpectedGcode);
            }
        };

        // before G0 raise the pen
        if code == Code::MoveRapid {
            sequence.push(Command::Move {
                x: Displacement::Relative(0),
                y: Displacement::Relative(0),
                z: Displacement::Relative(3000),
                a: syscfg.accel,
                v: syscfg.vel,
            });
        }

        let cmd = to_command(code, params, &syscfg, &mut pos)?;
        sequence.push(cmd);

        // lower
        if code == Code::MoveRapid {
            sequence.push(Command::Move {
                x: Displacement::Relative(0),
                y: Displacement::Relative(0),
                z: Displacement::Relative(-3000),
                a: syscfg.accel,
                v: syscfg.vel,
            });
        }
    }

    sequence.push(Command::Move {
        x: Displacement::Relative(0),
        y: Displacement::Relative(0),
        z: Displacement::Relative(5000),
        a: syscfg.accel,
        v: syscfg.vel,
    });

    Ok(sequence)
}

/// Filter only those lines, which will not cause an error in the parser.
fn line_filter(line: &str) -> bool {
    line.starts_with("G0 ")
        || line.starts_with("G1 ")
        || line.starts_with("G2 ")
        || line.starts_with("G3 ")
}
