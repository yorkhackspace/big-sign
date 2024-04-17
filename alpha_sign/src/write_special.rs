use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::hex_digit0;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::combinator::value;
use nom::multi::count;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::preceded;
use time::Time;

use crate::ParseInput;
use crate::ParseResult;

#[derive(Debug, Eq, PartialEq)]
pub enum WriteSpecial {
    SetTime(SetTime),
    ToggleSpeaker(ToggleSpeaker),
    ConfigureMemory(ConfigureMemory),
    ClearMemoryAndFlash(ClearMemoryAndFlash),
    SetDayOfWeek(SetDayOfWeek),
    SetTimeFormat(SetTimeFormat),
    GenerateSpeakerTone(GenerateSpeakerTone),
    SetRunTimeTable(SetRunTimeTable),
    DisplayAtXYPosition(),
    SoftReset(SoftReset),
    SetRunSequence(SetRunSequence),
    SetDimminRegister(),
    SetDimmingTimes(),
    SetRunDayTable(SetRunDayTable),
    ClearSerialErrorStatusRegister(ClearSerialErrorStatusRegister),
}

impl WriteSpecial {
    const COMMANDCODE: u8 = 0x45;

    pub fn encode(&self) -> Vec<u8> {
        let mut res = vec![Self::COMMANDCODE];
        let mut inner = match &self {
            WriteSpecial::SetTime(set_time) => set_time.encode(),
            WriteSpecial::ToggleSpeaker(toggle_speaker) => toggle_speaker.encode(),
            WriteSpecial::ConfigureMemory(configure_memory) => configure_memory.encode(),
            WriteSpecial::ClearMemoryAndFlash(clear_memory_and_flash) => {
                clear_memory_and_flash.encode()
            }
            WriteSpecial::SetDayOfWeek(set_day_of_week) => set_day_of_week.encode(),
            WriteSpecial::SetTimeFormat(set_time_format) => set_time_format.encode(),
            WriteSpecial::GenerateSpeakerTone(generate_speaker_tone) => {
                generate_speaker_tone.encode()
            }
            WriteSpecial::SetRunTimeTable(set_run_time_table) => set_run_time_table.encode(),
            WriteSpecial::DisplayAtXYPosition() => todo!(),
            WriteSpecial::SoftReset(soft_reset) => soft_reset.encode(),
            WriteSpecial::SetRunSequence(set_run_sequence) => set_run_sequence.encode(),
            WriteSpecial::SetDimminRegister() => todo!(),
            WriteSpecial::SetDimmingTimes() => todo!(),
            WriteSpecial::SetRunDayTable(set_run_day_table) => set_run_day_table.encode(),
            WriteSpecial::ClearSerialErrorStatusRegister(clear_serial_status_register) => {
                clear_serial_status_register.encode()
            }
        };
        res.append(&mut inner);
        res
    }

    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        delimited(
            tag([0x02, Self::COMMANDCODE]),
            alt((
                map(SetTime::parse, WriteSpecial::SetTime),
                map(ToggleSpeaker::parse, WriteSpecial::ToggleSpeaker),
                map(ConfigureMemory::parse, WriteSpecial::ConfigureMemory),
                map(ClearMemoryAndFlash::parse, |x| {
                    WriteSpecial::ClearMemoryAndFlash(x)
                }),
                map(SetDayOfWeek::parse, WriteSpecial::SetDayOfWeek),
                map(SetTimeFormat::parse, WriteSpecial::SetTimeFormat),
                map(GenerateSpeakerTone::parse, |x| {
                    WriteSpecial::GenerateSpeakerTone(x)
                }),
                map(SetRunTimeTable::parse, WriteSpecial::SetRunTimeTable),
                // TODO displayatXY position
                map(SoftReset::parse, WriteSpecial::SoftReset),
                map(SetRunSequence::parse, WriteSpecial::SetRunSequence),
                // TODO setDimmingRegister
                // TODO set dimming times
                map(SetRunDayTable::parse, WriteSpecial::SetRunDayTable),
                map(ClearSerialErrorStatusRegister::parse, |x| {
                    WriteSpecial::ClearSerialErrorStatusRegister(x)
                }),
            )),
            opt(preceded(char(0x03.into()), count(hex_digit0, 4))),
        )(input)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct SetTime {
    pub time: Time,
}

impl SetTime {
    const SPECIAL_LABEL: &'static [u8] = &[0x20];

    pub fn new(time: Time) -> Self {
        Self { time }
    }

    fn encode(&self) -> Vec<u8> {
        let hours = self.time.hour();
        let minutes = self.time.minute();
        let mut time = format!("{hours:0>2}{minutes:0>2}").into_bytes();
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        res.append(&mut time);
        res
    }

    pub fn parse(input: ParseInput) -> ParseResult<Self> {
        let (remain, parse) = preceded(
            char(0x20.into()),
            pair(
                map_res(count(one_of("0123456789"), 2), |x| {
                    x.iter().collect::<String>().parse::<u8>()
                }),
                map_res(count(one_of("0123456789"), 2), |x| {
                    x.iter().collect::<String>().parse::<u8>()
                }),
            ),
        )(input)?;

        Ok((
            remain,
            SetTime::new(Time::from_hms(parse.0, parse.1, 0).unwrap()),
        ))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ToggleSpeaker {
    pub enabled: bool,
}

impl ToggleSpeaker {
    const SPECIAL_LABEL: &'static [u8] = &[0x21];

    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        if self.enabled {
            res.push(0x30);
            res.push(0x30);
        } else {
            res.push(0x46);
            res.push(0x46);
        }
        res
    }
    fn parse(input: ParseInput) -> ParseResult<Self> {
        let (remain, parse) = preceded(
            char(0x21.into()),
            alt((
                value(true, tag([0x30, 0x30])),
                value(false, tag([0x46, 0x46])),
            )),
        )(input)?;

        Ok((remain, ToggleSpeaker::new(parse)))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum ColorStatus {
    Monochrome,
    Tricolor,
    Octocolor,
}
#[derive(Debug, PartialEq, Eq)]
pub struct StartStopTime {
    time: Time,
}

impl StartStopTime {
    pub fn new(hour: u8, tens: u8) -> Result<Self, time::error::ComponentRange> {
        Ok(Self {
            time: Time::from_hms(hour, tens * 10, 0)?,
        })
    }
    pub fn time(&self) -> Time {
        self.time
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum OnPeriod {
    Always,
    Never,
    AllDay, //TODO work out what this means
    Range {
        start_time: StartStopTime,
        end_time: StartStopTime,
    },
}

impl OnPeriod {
    fn encode(&self) -> Vec<u8> {
        let res: [u8; 2] = match self {
            OnPeriod::Always => [0xFF, 0x00],
            OnPeriod::Never => [0xFE, 0x00],
            OnPeriod::AllDay => [0xFD, 0x00],
            OnPeriod::Range {
                start_time,
                end_time,
            } => [
                start_time.time.hour() * 6 + start_time.time.minute() / 10,
                end_time.time.hour() * 6 + end_time.time.minute() / 10,
            ],
        };
        format!("{start:0<2X}{end:0<2X}", start = res[0], end = res[1]).into_bytes()
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Text {
        size: u16,
        on_period: OnPeriod,
    },
    String {
        size: u16,
    },
    Dots {
        x: u8,
        y: u8,
        color_status: ColorStatus,
    },
}
#[derive(Debug, PartialEq, Eq)]
pub struct MemoryConfiguration {
    pub label: char,
    pub file_type: FileType,
    pub keyboard_accessible: bool,
}

impl MemoryConfiguration {
    pub fn new(label: char, file_type: FileType, keyboard_accessible: bool) -> Self {
        Self {
            label,
            file_type,
            keyboard_accessible,
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![self.label as u8];
        let file_type = match self.file_type {
            FileType::Text { .. } => 0x41,
            FileType::String { .. } => 0x42,
            FileType::Dots { .. } => 0x43, //TODO confirm if this is correct might be 0x44 is typo in spec
        };
        res.push(file_type);
        if self.keyboard_accessible {
            res.push(0x55);
        } else {
            res.push(0x4c)
        }
        let mut file_size = match &self.file_type {
            FileType::Text { size, .. } | FileType::String { size, .. } => {
                format!("{size:0>4}").into_bytes()
            }
            FileType::Dots { x, y, .. } => format!("{y:0>2}{x:0>2}").into_bytes(),
        };
        res.append(&mut file_size);
        let mut file_config: Vec<u8> = match &self.file_type {
            FileType::Text { ref on_period, .. } => on_period.encode(),
            FileType::String { .. } => vec![0x30, 0x30, 0x30, 0x30],
            FileType::Dots { color_status, .. } => match color_status {
                ColorStatus::Monochrome => vec![0x31, 0x30, 0x30, 0x30],
                ColorStatus::Tricolor => vec![0x32, 0x30, 0x30, 0x30],
                ColorStatus::Octocolor => vec![0x38, 0x30, 0x30, 0x30],
            },
        };
        res.append(&mut file_config);
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}

pub struct SignOutOfMemory {}

#[derive(Debug, PartialEq, Eq)]
pub struct ConfigureMemory {
    //TODO check only the last file can have a size of 0
    configurations: Vec<MemoryConfiguration>,
}

impl ConfigureMemory {
    const SPECIAL_LABEL: &'static [u8] = &[0x24];

    pub fn new(configurations: Vec<MemoryConfiguration>) -> Result<Self, SignOutOfMemory> {
        for configuration in configurations.iter().rev().skip(1) {
            //TODO ignore for last element
            match configuration.file_type {
                FileType::Text { size, .. } | FileType::String { size } => {
                    if size == 0 {
                        return Err(SignOutOfMemory {});
                    }
                }
                _ => (),
            }
        }
        Ok(Self { configurations })
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        for configuration in &self.configurations {
            res.append(&mut configuration.encode())
        }
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ClearMemoryAndFlash {}

impl Default for ClearMemoryAndFlash {
    fn default() -> Self {
        Self::new()
    }
}

impl ClearMemoryAndFlash {
    const SPECIAL_LABEL: &'static [u8] = &[0x24, 0x24, 0x24, 0x24];

    pub fn new() -> Self {
        Self {}
    }

    fn encode(&self) -> Vec<u8> {
        Self::SPECIAL_LABEL.into()
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct SetDayOfWeek {
    pub day: time::Weekday,
}

impl SetDayOfWeek {
    const SPECIAL_LABEL: &'static [u8] = &[0x26];

    pub fn new(day: time::Weekday) -> Self {
        Self { day }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        let day = match self.day {
            time::Weekday::Sunday => 0x31,
            time::Weekday::Monday => 0x32,
            time::Weekday::Tuesday => 0x33,
            time::Weekday::Wednesday => 0x34,
            time::Weekday::Thursday => 0x35,
            time::Weekday::Friday => 0x36,
            time::Weekday::Saturday => 0x37,
        };
        res.push(day);
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct SetTimeFormat {
    pub twenty_four_hour: bool,
}

impl SetTimeFormat {
    const SPECIAL_LABEL: &'static [u8] = &[0x27];

    pub fn new(twenty_four_hour: bool) -> Self {
        Self { twenty_four_hour }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        if self.twenty_four_hour {
            res.push(0x4D)
        } else {
            res.push(0x53)
        }

        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ToneError {
    DurationOutOfRange,
    RepeatsOutOfRange,
    FrequencyOutOfRange,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ProgrammmableTone {
    frequency: u8,
    duration: u8,
    repeats: u8,
}

impl ProgrammmableTone {
    pub fn new(frequency: u8, duration: u8, repeats: u8) -> Result<Self, ToneError> {
        if frequency > 0xFE {
            Err(ToneError::FrequencyOutOfRange)
        } else if duration > 0xF {
            Err(ToneError::DurationOutOfRange)
        } else if repeats > 0xF {
            Err(ToneError::RepeatsOutOfRange)
        } else {
            Ok(Self {
                frequency,
                duration,
                repeats,
            })
        }
    }

    pub fn frequency(&self) -> u8 {
        self.frequency
    }

    pub fn duration(&self) -> u8 {
        self.duration
    }

    pub fn repeats(&self) -> u8 {
        self.repeats
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![0x32];
        res.append(
            &mut format!(
                "{frequency:0<2X}{duration:X}{repeats:X}",
                frequency = self.frequency,
                duration = self.duration,
                repeats = self.repeats
            )
            .into_bytes(),
        );
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum ToneType {
    SpeakerOn,
    SpeakerOff,
    Continuous2Seconds,
    ShortBeep2Seconds,
    ProgrammmableTone {
        programmable_tone: ProgrammmableTone,
    },
    StoreProgrammableSound,
    TriggerProgrammableSound,
}
#[derive(Debug, PartialEq, Eq)]
pub struct GenerateSpeakerTone {
    pub tone_type: ToneType,
}

impl GenerateSpeakerTone {
    const SPECIAL_LABEL: &'static [u8] = &[0x28];

    pub fn new(tone_type: ToneType) -> Self {
        Self { tone_type }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        match &self.tone_type {
            ToneType::SpeakerOn => res.push(0x41),
            ToneType::SpeakerOff => res.push(0x42),
            ToneType::Continuous2Seconds => res.push(0x30),
            ToneType::ShortBeep2Seconds => res.push(0x31),
            ToneType::ProgrammmableTone { programmable_tone } => {
                res.append(&mut programmable_tone.encode())
            }
            ToneType::StoreProgrammableSound => todo!(),
            ToneType::TriggerProgrammableSound => todo!(),
        }
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunTimeTable {
    label: char,
    on_period: OnPeriod,
}

impl RunTimeTable {
    pub fn new(label: char, on_period: OnPeriod) -> Self {
        Self { label, on_period }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![self.label as u8];
        res.append(&mut self.on_period.encode());
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetRunTimeTable {
    pub run_time_tables: Vec<RunTimeTable>,
}

impl SetRunTimeTable {
    const SPECIAL_LABEL: &'static [u8] = &[0x29];

    pub fn new(run_time_tables: Vec<RunTimeTable>) -> Self {
        Self { run_time_tables }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        for run_time_table in &self.run_time_tables {
            res.append(&mut run_time_table.encode())
        }
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SoftReset {}

impl Default for SoftReset {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftReset {
    const SPECIAL_LABEL: &'static [u8] = &[0x2c];

    pub fn new() -> Self {
        Self {}
    }

    fn encode(&self) -> Vec<u8> {
        let res: Vec<u8> = Self::SPECIAL_LABEL.into();
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
pub struct TooManyTextFiles {}

#[derive(Debug, PartialEq, Eq)]
pub enum RunSequenceType {
    FollowFileTimes,
    IgnoreFileTimes,
    DeleteAtOffTime,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetRunSequence {
    pub run_seqeunce_type: RunSequenceType,

    pub keyboard_accessible: bool,
    text_files: Vec<char>,
}

impl SetRunSequence {
    const SPECIAL_LABEL: &'static [u8] = &[0x2e];

    pub fn new(
        run_seqeunce_type: RunSequenceType,
        keyboard_accessible: bool,
        text_files: Vec<char>,
    ) -> Result<Self, TooManyTextFiles> {
        if text_files.len() > 128 {
            return Err(TooManyTextFiles {});
        }
        Ok(Self {
            run_seqeunce_type,
            keyboard_accessible,
            text_files,
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        if self.keyboard_accessible {
            res.push(0x55)
        } else {
            res.push(0x4C)
        }
        for label in &self.text_files {
            res.push(*label as u8)
        }
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum RunDays {
    Daily,
    WeekDays,
    Weekends,
    Always,
    Never,
    Range {
        start_day: time::Weekday,
        stop_day: time::Weekday,
    },
}

impl RunDays {
    fn encode(&self) -> Vec<u8> {
        match &self {
            RunDays::Daily => vec![0x30, 0x30],
            RunDays::WeekDays => vec![0x38, 0x30],
            RunDays::Weekends => vec![0x39, 0x30],
            RunDays::Always => vec![0x41, 0x30],
            RunDays::Never => vec![0x42, 0x30],
            RunDays::Range {
                start_day,
                stop_day,
            } => {
                let start = match start_day {
                    time::Weekday::Sunday => 0x31,
                    time::Weekday::Monday => 0x32,
                    time::Weekday::Tuesday => 0x33,
                    time::Weekday::Wednesday => 0x34,
                    time::Weekday::Thursday => 0x35,
                    time::Weekday::Friday => 0x36,
                    time::Weekday::Saturday => 0x37,
                };
                let stop = match stop_day {
                    time::Weekday::Sunday => 0x31,
                    time::Weekday::Monday => 0x32,
                    time::Weekday::Tuesday => 0x33,
                    time::Weekday::Wednesday => 0x34,
                    time::Weekday::Thursday => 0x35,
                    time::Weekday::Friday => 0x36,
                    time::Weekday::Saturday => 0x37,
                };
                vec![start, stop]
            }
        }
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct SetRunDayTable {
    pub label: char,
    pub run_days: RunDays,
}

impl SetRunDayTable {
    const SPECIAL_LABEL: &'static [u8] = &[0x32];

    pub fn new(label: char, run_days: RunDays) -> Self {
        Self { label, run_days }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Self::SPECIAL_LABEL.into();
        res.push(self.label as u8);
        res.append(&mut self.run_days.encode());
        res
    }
    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ClearSerialErrorStatusRegister {
    //TODO confirm whether this is correct, the
    //documentation sucks
}

impl Default for ClearSerialErrorStatusRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl ClearSerialErrorStatusRegister {
    const SPECIAL_LABEL: &'static [u8] = &[0x34];

    pub fn new() -> Self {
        Self {}
    }

    fn encode(&self) -> Vec<u8> {
        let res: Vec<u8> = Self::SPECIAL_LABEL.into();
        res
    }

    fn parse(_input: ParseInput) -> ParseResult<Self> {
        todo!()
    }
}
