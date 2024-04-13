pub mod text;

/// An M-Protocol command.
pub trait MProtocolCommand {
    /// The "opcode" of the command.
    fn command_code(&self) -> u8;
    /// The data to be written as part of the command body.
    fn data(&self) -> Vec<u8>;
}

impl<T> MProtocolCommand for Box<T>
where
    T: MProtocolCommand,
{
    fn command_code(&self) -> u8 {
        T::command_code(self)
    }

    fn data(&self) -> Vec<u8> {
        T::data(self)
    }
}
