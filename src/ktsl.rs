/// Koei Tecmo Sound Resource
/// Reverse engineered by Raytwo
/// Special thanks to HealingBrew/Yretenai, Devin, Liam and DeathChaos25. Let me know if I forgot someone!

use std::fs::File;
use std::path::Path;
use std::io::BufReader;

use binread::BinReaderExt;
use binread::{
    io::{Cursor, Read, Seek, SeekFrom},
    BinRead, BinResult, ReadOptions,
};

use binwrite::{
    BinWrite,
    WriterOption,
};

use std::{convert::TryInto, env, fs};

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
pub struct Ktsr {
    pub magic: [u8;4],
    pub section_type: Filetype,
    pub flags: u16,
    pub platform_id: Platform,
    pub game_id: Game,
    pub padding: u64,
    pub decomp_size: u32,
    #[binwrite(align_after(0x40))]
    pub comp_size: u32,
}

#[repr(u16)]
#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum Platform {
    Switch = 0x400,
    // ...
}

#[repr(u32)]
#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum Game {
    ThreeHouses = 0xB75674CE,
    // ...
}

#[repr(u32)]
#[derive(BinRead, Debug, Clone)]
#[br(little)]
/// The type of content in the KTSR. Names are made up.
pub enum Filetype {
    /// AS-bin
    AgdpcmStorage = 0x1A487B77,
    /// ST-bin
    StreamTable = 0xFCDD9402,
}

/// Header used by both Ktsl2asbin and Ktsl2stbin
impl Ktsr {
    pub fn new() -> Self {
        // Temp
        Ktsr {
            magic: *b"KTSR",
            section_type: Filetype::StreamTable,
            flags: 1,
            // TODO: Ask it in argument or serialize in a json?
            platform_id: Platform::Switch,
            // TODO: Ask it in argument or serialize in a json?
            game_id: Game::ThreeHouses,
            .. Default::default()
        }
    }
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum Section {
    // Name subject to change. Probably grouping stuff.
    #[br(magic = 0x368C88BDu32)]
    Info1(InfoSection),
    #[br(magic = 0x70CBCCC5u32)]
    Sound(SoundSection),
    // For future Ktsl2stbin parsing
    #[br(magic = 0x15F4D409u32)]
    Music(MusicSection),
    #[br(magic = 0xA8DB7261u32)]
    Padding(PaddingSection),
    #[br(magic = 0x368C88BDu32)]
    Unknown(UnknownSection),
}

// Ktsl2stbin and Ktsl2asbin are actually the exact same container with different structs inside. This structure represents their format.
#[derive(BinWrite, Debug, Default)]
pub struct Ktsl {
    pub header: Ktsr,
    //#[br(seek_before = SeekFrom::Start(0x40 as _)]
    #[binwrite(align(0x40))]
    pub entries: Vec<Section>,
}

impl Ktsl {
    pub fn new_asbin() -> Self {
        Ktsl {
            header: Ktsr {
                section_type: Filetype::AgdpcmStorage,
                .. Ktsr::new(),
            },
            entries: vec![],
        }
    }

    pub fn new_stbin() -> Self {
        Ktsl {
            header: Ktsr {
                section_type: Filetype::StreamTable,
                .. Ktsr::new(),
            },
            entries: vec![],
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::read(&mut BufReader::new(File::open(path)?))
    }

    // TODO: Rework this to be more generic. This is gross.
    pub fn get_companion_sections(&mut self) -> Vec<&mut KtssCompanionSection> {
        self.entries.iter_mut().filter_map(|section| {
            if let Section::Adpcm(adpcm) = section {
                return Some(adpcm)
            }

            None
        }).collect()
    }

    // Replace that.
    pub fn pack(&self) {
        let file = std::fs::File::create(&"./out.ktsl").unwrap();
        let mut writer = std::io::BufWriter::new(file);
        self.write(&mut writer).unwrap();
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