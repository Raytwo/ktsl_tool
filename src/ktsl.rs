/// Koei Tecmo Sound Resource
/// Reverse engineered by Raytwo
/// Special thanks to HealingBrew/Yretenai, Devin, Liam and DeathChaos25. Let me know if I forgot someone!

use std::fs::File;
use std::path::Path;
use std::io::BufReader;

use binread::{
    io::{Read, Seek, SeekFrom},
    BinRead, BinResult, ReadOptions,
};

use crate::sections;
use sections::{ InfoSection, SoundSection, MusicSection, PaddingSection, UnknownSection };


#[repr(C)]
#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub struct Ktsr {
    pub magic: [u8;4],
    pub filetype: Filetype,
    pub flags: u16,
    pub platform: Platform,
    pub game: Game,
    pub padding: u64,
    pub decomp_size: u32,
    pub comp_size: u32,
    pub enc_seed_size: u8,
    #[br(count = enc_seed_size)]
    pub enc_seed: Vec<u8>
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum Platform {
    #[br(magic = 0x100u16)]
    PC,
    #[br(magic = 0x400u16)]
    Switch,
    Unknown(u32)
    // ...
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum Game {
    #[br(magic = 0xB75674CEu32)]
    ThreeHouses,
    Unknown(u32)
    // ...
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
/// The type of content in the KTSR. Names are made up.
pub enum Filetype {
    /// AS-bin
    #[br(magic = 0x1A487B77u32)]
    Asset,
    /// ST-bin
    #[br(magic = 0xFCDD9402u32)]
    Stream,
}

/// Header used by both Ktsl2asbin and Ktsl2stbin
impl Ktsr {
    pub fn new() -> Self {
        // Temp
        Ktsr {
            magic: *b"KTSR",
            filetype: Filetype::Stream,
            flags: 1,
            // TODO: Ask it in argument or serialize in a json?
            platform: Platform::Switch,
            // TODO: Ask it in argument or serialize in a json?
            game: Game::ThreeHouses,
            padding: 0,
            decomp_size: 0,
            comp_size: 0,
            enc_seed_size: 0,
            enc_seed: vec![],
        }
    }
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum Section {
    // Name subject to change. Seems heavily related to voice groups
    #[br(magic = 0x368C88BDu32)]
    Info(InfoSection),
    // Contains either a KTSS/KOVS/RIFF descriptor or a embedded GCADPCM (or whatever they use on other platforms than the Switch)
    #[br(magic = 0x70CBCCC5u32)]
    Sound(SoundSection),
    #[br(magic = 0x15F4D409u32)]
    Music(MusicSection),
    #[br(magic = 0xA8DB7261u32)]
    Padding(PaddingSection),
    #[br(magic = 0x368C88BDu32)]
    Unknown(UnknownSection),
}

// Ktsl2stbin and Ktsl2asbin are actually the exact same container with different structs inside. This structure represents their format.
#[repr(C)]
#[derive(Debug)]
pub struct Ktsl {
    pub header: Ktsr,
    pub entries: Vec<Section>,
}

impl Ktsl {
    pub fn new_asbin() -> Self {
        Ktsl {
            header: Ktsr {
                filetype: Filetype::Asset,
                .. Ktsr::new()
            },
            entries: vec![],
        }
    }

    pub fn new_stbin() -> Self {
        Ktsl {
            header: Ktsr {
                filetype: Filetype::Stream,
                .. Ktsr::new()
            },
            entries: vec![],
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::read(&mut BufReader::new(File::open(path)?))
    }
}

impl BinRead for Ktsl {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _options: &ReadOptions, _args: Self::Args) -> BinResult<Self> {
        let mut ktsl = Ktsl {
            header: Ktsr::read(reader)?,
            entries: vec![],
        };

        reader.seek(SeekFrom::Start(0x40)).unwrap();

        while ktsl.header.decomp_size != binread::io::Seek::seek(reader, SeekFrom::Current(0))? as u32 {
            let section = Section::read(reader)?;

            ktsl.entries.push(section);
        }

        Ok(ktsl)
    }
}