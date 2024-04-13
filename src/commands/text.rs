use super::MProtocolCommand;

/// Command to write text.
pub struct WriteText {
    /// File to write to.
    file_name: char,
    /// Text to write.
    text: String,
}

impl WriteText {
    /// Creates a new command to write text to a sign.
    ///
    /// # Arguments
    /// * `file_name`: File to write to.
    /// * `text`: Text to write.
    ///
    /// # Returns
    /// A [`WriteText`] command.
    pub fn new(file_name: char, text: String) -> Self {
        Self { file_name, text }
    }
}

impl MProtocolCommand for WriteText {
    fn command_code(&self) -> u8 {
        0x41
    }

    fn data(&self) -> Vec<u8> {
        [self.file_name.to_string().as_bytes(), self.text.as_bytes()].concat()
    }
}
