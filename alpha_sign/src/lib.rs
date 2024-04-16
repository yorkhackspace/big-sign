use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::{complete::char, is_hex_digit},
    combinator::{map, map_opt, map_res, opt},
    multi::{many0, many1, many_m_n},
    number::complete::u8,
    sequence::{pair, preceded, terminated, tuple},
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::str;

pub mod text;
pub mod write_special;

pub type ParseInput<'a> = &'a [u8];
pub type ParseResult<'a, O> =
    nom::IResult<ParseInput<'a>, O, nom::error::VerboseError<ParseInput<'a>>>;

/// the sign address used for broadcasting to all serial addresses
pub const BROADCAST: u8 = 0x00;

/// a sign selection with a given sign type and a given serial address.
///
/// both sign type and address have options that will allow selecting more than one sign.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SignSelector {
    pub sign_type: SignType,
    pub address: u8,
}

impl Default for SignSelector {
    /// a selector that will select every sign
    fn default() -> SignSelector {
        SignSelector {
            sign_type: SignType::All,
            address: 0,
        }
    }
}

impl SignSelector {
    /// create a new sign selector.
    ///
    /// if you want to select every sign see [`SignSelector::default`]
    pub fn new(sign_type: SignType, address: u8) -> Self {
        SignSelector { sign_type, address }
    }

    /// parse the raw bytes of a sign and turn it into a sign selector
    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        let (remain, res) = pair(
            map_opt(u8, SignType::from_u8),
            map_res(take_while(is_hex_digit), |x| {
                u8::from_str_radix(str::from_utf8(x).unwrap(), 16)
            }),
        )(input)?;

        Ok((
            remain,
            SignSelector {
                sign_type: res.0,
                address: res.1,
            },
        ))
    }
}

/// a packet containg a collection of one or more commands and sign selectors
///
/// the packet will always use checksums if a command fails check the serial status register using
/// //TODO add command
/// to see if there was an error
///
/// to run the packet on the sign run [`Packet::encode`] and send it over serial to the sign
#[derive(Debug, Eq, PartialEq)]
pub struct Packet {
    /// a vec of selectors to pick which signs the packet should operate on
    pub selectors: Vec<SignSelector>,
    /// the commands that are to be run on the sign.
    ///
    /// Note that only one read command can be used per packet and it MUST be last
    ///
    /// also if used [`write_special::GenerateSpeakerTone`] must be last and the sign will not
    /// respond on serial while controlling the speaker. see [`write_special::GenerateSpeakerTone`]
    /// for more information
    pub commands: Vec<Command>,
}

impl Packet {
    /// create a new packet.
    pub fn new(selectors: Vec<SignSelector>, commands: Vec<Command>) -> Self {
        //TODO maybe make this validate that read cant be not last
        Self {
            selectors,
            commands,
        }
    }

    /// encode a packet returning the raw bytes to be sent to the sign
    pub fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x01]; //start of transmission
        for selector in &self.selectors {
            res.push(selector.sign_type as u8);
            res.append(&mut format!("{address:0>2X}", address = selector.address).into_bytes());
            res.push(0x2c);
        }
        res.pop(); // remove trailing comma
        for command in &self.commands {
            let mut command_section: Vec<u8> = vec![0x02]; //start of command
            command_section.append(&mut command.encode());
            command_section.push(0x03); //end of command
            let mut sum: u16 = 0;
            for byte in command_section.clone() {
                sum += byte as u16;
            }
            command_section.append(&mut format!("{sum:0>4X}").into_bytes());
            res.append(&mut command_section);
        }
        res.push(0x04); //end of transmission
        res
    }

    /// parse a response from a sign returing a packet.
    pub fn parse(packet: ParseInput) -> ParseResult<Self> {
        let (remaining, result) = tuple((
            preceded(
                pair(
                    many_m_n(5, 100, char(0x00.into())),         // starting nulls
                    nom::character::complete::char(0x01.into()), // start of transmission
                ),
                many1(terminated(SignSelector::parse, opt(char(',')))),
            ),
            terminated(
                many0(Command::parse),
                nom::character::complete::char(0x04.into()), // commands
            ),
        ))(packet)?;

        Ok((
            remaining,
            Packet {
                selectors: result.0,
                commands: result.1,
            },
        ))
    }
}

/// a command to be run on the sign
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    WriteText(text::WriteText),
    ReadText(text::ReadText),
    WriteSpecial(write_special::WriteSpecial),
}

impl Command {
    /// encode a command.
    ///
    ///<div class="warning">you probably dont want to use this
    ///
    ///the sign needs to be sent a [`Packet::encode`] not a command.
    ///dont use this method if you want to send a command to the sign.
    ///
    ///</div>
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Command::WriteText(write_text) => write_text.encode(),
            Command::ReadText(read_text) => read_text.encode(),
            Command::WriteSpecial(write_special) => write_special.encode(),
        }
    }

    /// returns true if the command is a read command or false if it is a write command.
    pub fn is_read(&self) -> bool {
        match self {
            Command::WriteText(_) => false,
            Command::ReadText(_) => true,
            Command::WriteSpecial(_) => false,
        }
    }

    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        Ok(alt((
            map(text::WriteText::parse, |x| Command::WriteText(x)),
            map(text::ReadText::parse, |x| Command::ReadText(x)),
            map(write_special::WriteSpecial::parse, |x| {
                Command::WriteSpecial(x)
            }),
        ))(input)?)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq, Eq)]
/// a type of sign to operate on, some types such as [`SignType::All`] refer to more than one sign
/// model
pub enum SignType {
    /// all signs that support visual verififcation. this causes a sign to display the transmission
    /// ok message when a transmission packet is recieved without error. otherwise transmission
    /// error will appear
    SignWithVisualVerification = 0x21,
    SerialClock = 0x22,
    /// both the full matrix and character matrix alpha vision signs
    AlphaVision = 0x23,
    FullMatrixAlphaVision = 0x24,
    CharacterMatrixAlphaVision = 0x25,
    LineMatrixAlphaVision = 0x26,
    ResponsePacket = 0x30,
    /// any sign that has one line of text
    OneLineSign = 0x31,
    /// any sign that has two lines of text
    TwoLineSign = 0x32,
    /// all signs. functionally equivelent to [`SignType::All`] as far as we can tell
    AllSigns = 0x3f,
    Sign430i = 0x43,
    Sign440i = 0x44,
    Sign460i = 0x45,
    AlphaEclipse3600DisplayDriverBoard = 0x46,
    AlphaEclipse3600TurboAdapterBoard = 0x47,
    LightSensorProbe = 0x4c,
    Sign790i = 0x55,
    AlphaEclipse3600Series = 0x56,
    /// an alphaeclipse 1500 Time and temp sign. note that this sign only supports displaying time
    /// updates and cannot display messages
    AlphaEclipseTimeTemp = 0x57,
    /// both the alpha premier 4000 and 9000 series
    AlphaPremiere4000And9000Series = 0x58,
    /// all signs. functionally equivelent to [`SignType::AllSigns`] as far as we can tell
    All = 0x5a,
    Betabrite = 0x5e,
    Sign4120C = 0x61,
    Sign4160C = 0x62,
    Sign4200C = 0x63,
    Sign4240C = 0x64,
    Sign215R = 0x65,
    Sign215C = 0x66,
    Sign4120R = 0x67,
    Sign4160R = 0x68,
    Sign4200R = 0x69,
    Sign4240R = 0x6a,
    /// all 300 series signs (320C and 330C)
    Series300 = 0x6b,
    /// all 7000 series signs (7080C, 7120C, 7160C, 7200C)
    Series7000 = 0x6c,
    MatrixSolar96x16 = 0x6d,
    MatrixSolar128x16 = 0x6e,
    MatrixSolar160x16 = 0x6f,
    MatrixSolar192x16 = 0x70,
    /// personal priority display
    PPD = 0x71,
    Director = 0x72,
    DigitController1005 = 0x73,
    Sign4080C = 0x74,
    Sign210CAnd220C = 0x75,
    AlphaEclipse3500 = 0x76,
    /// an alphaeclipse 1500 Time and temp sign. note that this sign only supports displaying time
    /// updates and cannot display messages
    AlphaEclipse1500TimeAndTemp = 0x77,
    AlphaPremiere9000 = 0x78,
    TemperatureProbe = 0x79,
    /// all signs that have their memory configured for 26 files ("A" - "Z")
    AllSignsWithMemoryConfiguredFor26Files = 0x7a,
}
