#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TextPosition {
    MiddleLine = 0x20,
    TopLine = 0x22,
    BottomLine = 0x26,
    Fill = 0x30,
    Left = 0x31,
    Right = 0x32,
}

#[derive(PartialEq, Eq, Clone, Copy)]
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

pub struct WriteText {
    label: char,
    message: String,
    position: TextPosition,
    mode: TransitionMode,
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
            res.push(self.position as u8);
            res.append(&mut self.mode.into());
        }
        res.extend_from_slice(self.message.as_bytes().into());
        res
    }
}

pub struct ReadText {
    label: char,
}

impl ReadText {
    const COMMANDCODE: u8 = 0x41;
    pub fn new(label: char) -> Self {
        Self { label }
    }

    pub fn encode(&self) -> Vec<u8> {
        vec![Self::COMMANDCODE, self.label as u8]
    }
}
