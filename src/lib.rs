use std::io;

use commands::MProtocolCommand;
use serde::{Deserialize, Serialize};
use serialport::SerialPort;

pub mod commands;
pub mod web_server;

/// The header of new transmissions to the sign.
pub const TRANSMISSION_HEADER: [u8; 5] = [0x00; 5];
/// Byte to signal the start of the message heading.
pub const START_OF_HEADING: u8 = 0x01;
/// Byte to signal the start of the message text.
pub const START_OF_TEXT: u8 = 0x02;
/// Byte to signal the end of a transmission.
pub const END_OF_TRANSMISSION: u8 = 0x04;

/// A sign made by Alpha-American.
pub struct AlphaSign {
    /// The serial port that the sign is connected to.
    port: Box<dyn SignSerial>,
    /// The address of the sign.
    sign_address: [u8; 2],
    /// the type of sign to broadcast to.
    type_code: TypeCode,
}

/// Types of sign that can be broadcast to.
#[derive(Clone, Copy)]
pub enum TypeCode {
    /// Broadcast to all signs.
    AllSigns,
}

/// A command that can be sent to a sign.
pub enum SignCommand {
    /// Write some text directly to the sign.
    WriteText {
        /// The text to display (should only contain ASCII characters).
        text: String,
    },
    /// Run a script, this will block any other commands from being executed until the script exits.
    RunScript {
        script_language: SignScriptLanguage,
        /// The script to execute.
        script: String,
    },
}

/// Laguages that are supported for writing scripts for the sign.
#[derive(Debug, Serialize, Deserialize)]
pub enum SignScriptLanguage {
    /// https://rhai.rs/
    #[serde(rename = "rhai")]
    Rhai,
}

/// A trait to be implemented by types that provide access to signs.
pub trait SignSerial {
    /// Write some bytes to the sign.
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error>;
}

impl AlphaSign {
    /// Creates a new [`AlphaSign`].
    ///
    /// # Arguments
    /// * `serial_port`: Communication method for talking to the sign.
    /// * `sign_address`: Address of the sign to talk to.
    /// * `type_code`: The type of sign to talk to.
    ///
    /// # Returns
    /// A new [`AlphaSign`].
    pub fn new(
        serial_port: Box<dyn SignSerial>,
        sign_address: [u8; 2],
        type_code: TypeCode,
    ) -> Self {
        Self {
            port: serial_port,
            sign_address,
            type_code,
        }
    }

    /// Sends a command to the sign.
    ///
    /// # Arguments
    /// * `command`: The command to send.
    pub fn send_command<Command>(&mut self, command: Command)
    where
        Command: MProtocolCommand,
    {
        let command = [
            TRANSMISSION_HEADER.to_vec(),
            [START_OF_HEADING].to_vec(),
            [Into::<u8>::into(self.type_code)].to_vec(),
            self.sign_address.to_vec(),
            [START_OF_TEXT].to_vec(),
            [command.command_code()].to_vec(),
            command.data(),
            [END_OF_TRANSMISSION].to_vec(),
        ]
        .concat();
        self.port.write(&command).expect("Write failed!");
    }
}

impl From<TypeCode> for u8 {
    fn from(value: TypeCode) -> Self {
        match value {
            TypeCode::AllSigns => 0x5A,
        }
    }
}

impl<S> SignSerial for Box<S>
where
    S: SerialPort + Sized,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        S::write(self, buf)
    }
}
