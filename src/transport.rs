//! Serial transport and [`Command`] - [`Response`] transactions.

use std::io::BufReader;
use std::io::prelude::*;
use std::time::Duration;

use juk_cmd::cmd::Command;
use juk_cmd::cmd::Response;

use serialport::ClearBuffer;
use serialport::DataBits;
use serialport::Parity;
use serialport::SerialPort;
use serialport::StopBits;

/// Wrapper around a serial port, providing a transaction interface for [`Command`] - [`Response`]
/// communication.
pub struct Interface {
    serial: BufReader<Box<dyn SerialPort>>,
}

impl Interface {
    /// Open the [`Interface`] at `port`.
    pub fn open<S: AsRef<str>>(port: S) -> crate::Result<Self> {
        log::debug!("Opening serial port at: {}", port.as_ref());
        let serial = serialport::new(port.as_ref(), 115200)
            .timeout(Duration::from_mins(5))
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .open()?;

        serial.clear(ClearBuffer::All)?;

        if let Some(name) = serial.name() {
            log::info!("Connected to: {}", name);
        } else {
            log::info!("Connected to: {}", port.as_ref());
        }

        Ok(Self {
            serial: BufReader::with_capacity(32, serial),
        })
    }

    /// Perform a transaction.
    ///
    /// # Warning
    ///
    /// This function may block indefinitely until an end-of-frame was received.
    pub fn transaction(&mut self, cmd: &Command) -> crate::Result<Response> {
        let output = postcard::to_stdvec_cobs(cmd)?;
        self.serial.get_mut().write_all(&output)?;

        let mut input = Vec::with_capacity(32);
        let n = self.serial.read_until(0, &mut input)?;
        let resp = postcard::from_bytes_cobs(&mut input[..n])?;

        Ok(resp)
    }
}
