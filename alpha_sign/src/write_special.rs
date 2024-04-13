use time::Time;

pub enum WriteSpecial {
    SetTime(SetTime),
    ToggleSpeaker(ToggleSpeaker),
    ConfigureMemory(ConfigureMemory),
}

impl WriteSpecial {
    const COMMANDCODE: u8 = 0x45;

    pub fn encode(&self) -> Vec<u8> {
        let mut res = vec![Self::COMMANDCODE];
        let mut inner = match &self {
            WriteSpecial::SetTime(set_time) => set_time.encode(),
            WriteSpecial::ToggleSpeaker(toggle_speaker) => toggle_speaker.encode(),
            WriteSpecial::ConfigureMemory(configure_memory) => configure_memory.encode(),
        };
        res.append(&mut inner);
        res
    }
}

pub struct SetTime {
    time: Time
}

impl SetTime {
    const SPECIALLABEL: &'static [u8] = &[0x20];

    pub fn new(time:Time) -> Self{
        Self{
            time
        }
    }

    fn encode(&self) -> Vec<u8> {
        let hours = self.time.hour();
        let minutes = self.time.minute();
        let mut time = format!("{hours:0>2}{minutes:0>2}").into_bytes();
        let mut res: Vec<u8>= Self::SPECIALLABEL.into();
        res.append(&mut time);
        res
    }
}

pub struct ToggleSpeaker {
    enabled: bool
}

impl ToggleSpeaker {
    const SPECIAL_LABEL: &'static [u8] = &[0x21];

    pub fn new(enabled:bool) -> Self {
        Self {
            enabled
        }
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
}

pub enum ColorStatus {
    Monochrome,
    Tricolor,
    Octocolor,
}

pub struct StartStopTime {
    time: Time
}

impl StartStopTime {
    pub fn new (hour:u8, tens:u8) -> Result<Self, time::error::ComponentRange>{
        Ok(Self{
            time:Time::from_hms(hour, tens*10, 0)?
        })
    }
} 

pub enum OnPeriod{
    Always,
    Never,
    AllDay,//TODO work out what this means
    Range {start_time: StartStopTime, end_time: StartStopTime}
}

impl OnPeriod{

    fn encode(&self) -> Vec<u8>{
        let res: [u8; 2]= match self{
            OnPeriod::Always => [0xFF,0x00],
            OnPeriod::Never => [0xFE,0x00],
            OnPeriod::AllDay => [0xFD,0x00],
            OnPeriod::Range { start_time, end_time } => {
                [start_time.time.hour()*6+start_time.time.minute()/10,
                    end_time.time.hour()*6 + end_time.time.minute()/10]
            }
        };
        format!("{start:0<2X}{end:0<2X}",start=res[0],end=res[1]).into_bytes()

    }
}

pub enum FileType {
    Text {on_period: OnPeriod},
    String,
    Dots {color_status: ColorStatus}, 
}

pub struct MemoryConfiguration {
    label: char,
    file_type: FileType,
    keyboard_accessible: bool,
    size: u16
}

impl MemoryConfiguration {
    pub fn new(label:char, file_type: FileType, keyboard_accessible: bool, size: u16) -> Self {
        Self{
            label,
            file_type,
            keyboard_accessible,
            size
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![self.label as u8];
        let file_type = match self.file_type {
            FileType::Text {..} => 0x41,
            FileType::String => 0x42,
            FileType::Dots {..} => 0x43//TODO confirm if this is correct might be 0x44 is typo in spec
        };
        res.push(file_type);
        if self.keyboard_accessible{
            res.push(0x55);
        } else {
            res.push(0x4c)
        }
        res.append(&mut format!("{size:0>4}",size=self.size).into_bytes());
        let mut file_config: Vec<u8> = match &self.file_type {
            FileType::Text {ref on_period } => on_period.encode(),
            FileType::String => vec![0x30, 0x30, 0x30, 0x30],
            FileType::Dots { color_status } => {
                match color_status {
                    ColorStatus::Monochrome => vec![0x31, 0x30, 0x30, 0x30],
                    ColorStatus::Tricolor => vec![0x32, 0x30, 0x30, 0x30],
                    ColorStatus::Octocolor => vec![0x38, 0x30, 0x30, 0x30],
                }
            },
        };
        res.append(&mut file_config);
        res
    }
}

pub struct ConfigureMemory {
    pub configurations: Vec<MemoryConfiguration>
}

impl ConfigureMemory {
    const SPECIAL_LABEL: &'static [u8] = &[0x21];

    pub fn new() -> Self {
        Self{
            configurations:Vec::new()
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut res:Vec<u8> = Self::SPECIAL_LABEL.into();
        for configuration in &self.configurations {
            res.append(&mut configuration.encode())
        }
        res
    }
}
