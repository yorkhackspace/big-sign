
pub mod write_special;
pub mod text;

pub const BROADCAST: u8 = 0xFF;

pub struct AlphaSign {
    pub sign_type: SignType,
    pub address: u8, //TODO add support for multiple types and addresses
    pub checksum: bool,
}

impl AlphaSign {
    pub fn encode(&self, commands: Vec<Command>) -> Vec<u8>{
        let mut res: Vec<u8>= vec![0x00,0x00,0x00,0x00,0x00,0x01]; //start of transmission
        res.push(self.sign_type as u8);
        res.push(self.address);
        for command in commands {
            res.push(0x02); //start of command
            res.append(&mut command.encode());
        }
        res.push(0x04); //end of transmission
        res
        
    }

}


impl Default for AlphaSign {
    fn default() -> Self {
        AlphaSign {
            sign_type: SignType::All,
            address: BROADCAST,
            checksum: true,
        }
    }
}



pub enum Command {
    WriteText(text::WriteText),
    ReadText(text::ReadText),
    WriteSpecial(write_special::WriteSpecial),
}

impl Command {
    pub fn encode(&self) -> Vec<u8>{
        match self {
            Command::WriteText(write_text) => write_text.encode(),
            Command::ReadText(read_text) => read_text.encode(),
            Command::WriteSpecial(write_special) => write_special.encode(),
        }
    } 
}



#[repr(u8)]
#[derive(Copy, Clone)]
pub enum SignType {
    //TODO visual verification
    SerialClock = 0x22,
    AlphaVision = 0x23,
    FullMatrixAlphaVision = 0x24,
    CharacterMatrixAlphaVision = 0x25,
    LineMatrixAlphaVision = 0x26,
    OneLineSign = 0x31,
    TwoLineSign = 0x32,
    AllSigns = 0x3f,
    Sign430i = 0x43,
    Sign440i = 0x44,
    Sign460i = 0x45,
    AlphaEclipse3600DisplayDriverBoard = 0x46,
    AlphaEclipse3600TurboAdapterBoard = 0x47,
    LightSensorProbe = 0x4c,
    Sign790i = 0x55,
    AlphaEclipse3600Series = 0x56,
    AlphaEclipseTimeTemp = 0x57,
    AlphaPremiere4000And9000Series = 0x58,
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
    Series300 = 0x6b,
    Series7000 = 0x6c,
    MatrixSolar96x16 = 0x6d,
    MatrixSolar128x16 = 0x6e,
    MatrixSolar160x16 = 0x6f,
    MatrixSolar192x16 = 0x70,
    PPD = 0x71,
    Director = 0x72,
    DigitController1005 = 0x73,
    Sign4080C = 0x74,
    Sign210CAnd220C = 0x75,
    AlphaEclipse3500 = 0x76,
    AlphaEclipse1500TimeAndTemp = 0x77,
    AlphaPremiere9000 = 0x78,
    TemperatureProbe = 0x79,
    //TODO all signs with memory configured for 26 file = 0x7a
}

#[cfg(test)]
mod tests {}
