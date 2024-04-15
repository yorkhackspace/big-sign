use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::character::complete::anychar;
use nom::character::complete::char;
use nom::character::complete::hex_digit0;
use nom::character::complete::one_of;
use nom::combinator::map_opt;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::multi::count;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::str;

use crate::ParseInput;
use crate::ParseResult;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
pub enum TextPosition {
    MiddleLine = 0x20,
    TopLine = 0x22,
    BottomLine = 0x26,
    Fill = 0x30,
    Left = 0x31,
    Right = 0x32,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TransitionMode {
    Rotate,
    Hold,
    Flash,
    RollUp,
    RollDown,
    RollLeft,
    RollRight,
    WipeUp,
    WipeDown,
    WipeLeft,
    WipeRight,
    Scroll,
    AutoMode,
    RollIn,
    RollOut,
    WipeIn,
    WipeOut,
    CompressedRotate,
    Explode,
    Clock,
    //next ones are special and dont work on all signs
    Twinkle,
    Sparkle,
    Snow,
    Interlock,
    Switch,
    Slide,
    Spray,
    Starburst,
    Welcome,
    SlotMachine,
    NewsFlash,
    TrumpetAnimation,
    CycleColors,
}
impl Into<Vec<u8>> for TransitionMode {
    fn into(self) -> Vec<u8> {
        match self {
            TransitionMode::Rotate => vec![0x61],
            TransitionMode::Hold => vec![0x62],
            TransitionMode::Flash => vec![0x63],
            TransitionMode::RollUp => vec![0x65],
            TransitionMode::RollDown => vec![0x66],
            TransitionMode::RollLeft => vec![0x67],
            TransitionMode::RollRight => vec![0x68],
            TransitionMode::WipeUp => vec![0x69],
            TransitionMode::WipeDown => vec![0x6A],
            TransitionMode::WipeLeft => vec![0x6B],
            TransitionMode::WipeRight => vec![0x6C],
            TransitionMode::Scroll => vec![0x6D],
            TransitionMode::AutoMode => vec![0x6F],
            TransitionMode::RollIn => vec![0x70],
            TransitionMode::RollOut => vec![0x71],
            TransitionMode::WipeIn => vec![0x72],
            TransitionMode::WipeOut => vec![0x73],
            TransitionMode::CompressedRotate => vec![0x74],
            TransitionMode::Explode => vec![0x75],
            TransitionMode::Clock => vec![0x76],
            TransitionMode::Twinkle => vec![0x6E, 0x30],
            TransitionMode::Sparkle => vec![0x6E, 0x31],
            TransitionMode::Snow => vec![0x6E, 0x32],
            TransitionMode::Interlock => vec![0x6E, 0x33],
            TransitionMode::Switch => vec![0x6E, 0x34],
            TransitionMode::Slide => vec![0x6E, 0x35],
            TransitionMode::Spray => vec![0x6E, 0x36],
            TransitionMode::Starburst => vec![0x6E, 0x37],
            TransitionMode::Welcome => vec![0x6E, 0x38],
            TransitionMode::SlotMachine => vec![0x6E, 0x39],
            TransitionMode::NewsFlash => vec![0x6E, 0x3a],
            TransitionMode::TrumpetAnimation => vec![0x6E, 0x3b],
            TransitionMode::CycleColors => vec![0x6E, 0x43],
        }
    }
}

impl From<Vec<u8>> for TransitionMode {
    fn from(input: Vec<u8>) -> Self {
        let modes = [
            TransitionMode::Rotate,
            TransitionMode::Hold,
            TransitionMode::Flash,
            TransitionMode::RollUp,
            TransitionMode::RollDown,
            TransitionMode::RollLeft,
            TransitionMode::RollRight,
            TransitionMode::WipeUp,
            TransitionMode::WipeDown,
            TransitionMode::WipeLeft,
            TransitionMode::WipeRight,
            TransitionMode::Scroll,
            TransitionMode::AutoMode,
            TransitionMode::RollIn,
            TransitionMode::RollOut,
            TransitionMode::WipeIn,
            TransitionMode::WipeOut,
            TransitionMode::CompressedRotate,
            TransitionMode::Explode,
            TransitionMode::Clock,
            TransitionMode::Twinkle,
            TransitionMode::Sparkle,
            TransitionMode::Snow,
            TransitionMode::Interlock,
            TransitionMode::Switch,
            TransitionMode::Slide,
            TransitionMode::Spray,
            TransitionMode::Starburst,
            TransitionMode::Welcome,
            TransitionMode::SlotMachine,
            TransitionMode::NewsFlash,
            TransitionMode::TrumpetAnimation,
            TransitionMode::CycleColors,
        ];

        for m in modes {
            let val: Vec<u8> = m.into();
            if input.as_slice() == val.as_slice() {
                return m;
            }
        }
        TransitionMode::AutoMode
    }
}

impl TextPosition {
    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        map_opt(one_of([0x20, 0x22, 0x26, 0x30, 0x31, 0x32]), |x| {
            TextPosition::from_u8(x as u8)
        })(input)
    }
}
impl TransitionMode {
    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        let (remain, parse) = pair(anychar, opt(anychar))(input)?;

        let mut code: Vec<u8> = vec![parse.0 as u8];
        if let Some(second) = parse.1 {
            code.push(second as u8)
        }
        Ok((remain, TransitionMode::from(code)))
    }
}

// parses any number of ASCII printable characters
#[derive(Debug, PartialEq, Eq)]
pub struct WriteText {
    pub label: char,
    pub message: String,
    pub position: TextPosition,
    pub mode: TransitionMode,
}
impl WriteText {
    pub const PRIORITY_LABEL: char = '0';
    const COMMANDCODE: u8 = 0x41;

    pub fn new(label: char, message: String) -> Self {
        //TODO check lable is valid
        //TODO make a message type
        Self {
            label,
            message,
            position: TextPosition::MiddleLine,
            mode: TransitionMode::AutoMode,
        }
    }

    pub fn position(mut self, position: TextPosition) -> Self {
        self.position = position;
        self
    }

    pub fn mode(mut self, mode: TransitionMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut res = vec![Self::COMMANDCODE, self.label as u8];

        if self.position != TextPosition::MiddleLine || self.mode != TransitionMode::AutoMode {
            res.push(0x1b);
            res.push(self.position as u8);
            res.append(&mut self.mode.into());
        }
        res.extend_from_slice(self.message.as_bytes().into());
        res
    }

    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        let (remain, parse) = delimited(
            tag([0x02, Self::COMMANDCODE]), // command code
            tuple((
                anychar, // label, TODO label parser
                opt(preceded(
                    char(0x1b.into()),
                    pair(TextPosition::parse, TransitionMode::parse),
                )), // text position and transition mode
                map_res(take_while(|x| x >= 0x20), str::from_utf8), // message body
            )),
            opt(preceded(char(0x03.into()), count(hex_digit0, 4))),
        )(input)?;

        let mut w = WriteText::new(parse.0, parse.2.to_string());

        if let Some((position, mode)) = parse.1 {
            w.position = position;
            w.mode = mode;
        }

        Ok((remain, w))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ReadText {
    pub label: char,
}

impl ReadText {
    const COMMANDCODE: u8 = 0x42;
    pub fn new(label: char) -> Self {
        Self { label }
    }

    pub fn encode(&self) -> Vec<u8> {
        vec![Self::COMMANDCODE, self.label as u8]
    }

    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        let (remain, parse) = delimited(
            tag([0x02, Self::COMMANDCODE]),
            anychar,                                                // label
            opt(preceded(char(0x03.into()), count(hex_digit0, 4))), // optional checksum
        )(input)?;

        Ok((remain, ReadText::new(parse)))
    }
}
