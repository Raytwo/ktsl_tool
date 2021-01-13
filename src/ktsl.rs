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

pub const KTSL_HEADER_SIZE: u32 =  0x40;

// Header for the container representing every single entry
#[derive(BinRead, BinWrite, Debug, Default, Clone)]
pub struct MusicSection {
    pub section_size: u32,
    pub link_id: u32,
    pub header_size: u32,
    #[binwrite(align_after(0x40))]
    pub ktss_size: u32,
    #[br(align_before(0x40), align_after(0x40))]
    #[binwrite(align_after(0x40))]
    pub ktss: Ktss,
}

impl MusicSection {
    pub fn new() -> Self {
        MusicSection {
            header_size: KTSL_HEADER_SIZE,
            .. Default::default()
        }
    }
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
pub struct Ktss {
    pub magic: u32,
    #[binwrite(align_after(0x20))]
    pub section_size: u32,
    #[br(align_before(0x20))]
    pub codec: u8,
    unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    codec_start_offset: u32,
    pub layer_count: u8,
    pub channel_count: u8,
    unk4: u16,
    pub sample_rate: u32,
    pub sample_count: u32,
    pub loop_start: u32,
    pub loop_length: u32,
    padding: u32,
    audio_section_addr: u32,
    audio_section_size: u32,
    unk5: u32,
    pub frame_count: u32,
    pub frame_size: u16,
    some_constant: u16,
    pub orig_sample_rate: u32,
    pub skip: u16,
    pub stream_count: u8,
    pub coupled_count: u8,
    #[br(count = channel_count, align_after(0x10), pad_after(0x10))]
    #[binwrite(align_after(0x10), pad_after(0x10))]
    pub channel_mapping: Vec<u8>,
    #[br(big, count = frame_count)]
    #[binwrite(big)]
    pub audio: Vec<LopusPacket>
}

impl Ktss {
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::read(&mut BufReader::new(File::open(path)?))
    }
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
pub struct LopusPacket {
    pub size: u32,
    pub unk: u32,
    #[br(count = size)]
    pub content: Vec<u8>,
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