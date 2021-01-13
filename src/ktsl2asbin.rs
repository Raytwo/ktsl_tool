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

use crate::ktsl2stbin::{
    Ktsr,
    Ktss,
    // Will replace KtssSection at some point
    //KtslEntry
};

/// Is actually the exact same format as Ktsl2stbin. The implementation should probably be merged.
#[derive(BinWrite, Debug, Default, Clone)]
pub struct Ktsl2asbin {
    pub header: Ktsr,
    #[binwrite(with(write_sections))]
    pub entries: Vec<Section>,
}

impl Ktsl2asbin {
    pub fn new() -> Self {
        Ktsl2asbin {
            header: Ktsr::new(),
            entries: vec![],
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::read(&mut BufReader::new(File::open(path)?))
    }

    pub fn get_companion_sections(&mut self) -> Vec<&mut KtssCompanionSection> {
        self.entries.iter_mut().filter_map(|section| {
            if let Section::Adpcm(adpcm) = section {
                return Some(adpcm)
            }

            None
        }).collect()
    }

    pub fn pack(&self) {
        let file = std::fs::File::create(&"./out.ktsl2asbin").unwrap();
        let mut writer = std::io::BufWriter::new(file);
        self.write(&mut writer).unwrap();
    }
}

impl BinRead for Ktsl2asbin {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _options: &ReadOptions, _args: Self::Args) -> BinResult<Self> {
        let mut ktsl2asbin = Ktsl2asbin {
            header: Ktsr::read(reader)?,
            entries: vec![],
        };

        reader.seek(SeekFrom::Start(0x40)).unwrap();

        while ktsl2asbin.header.decomp_size != binread::io::Seek::seek(reader, SeekFrom::Current(0))? as u32 {
            let section = Section::read(reader)?;

            ktsl2asbin.entries.push(section);
        }

        Ok(ktsl2asbin)
    }
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
pub struct KtssSection {
    pub section_size: u32,
    pub link_id: u32,
    pub header_size: u32,
    #[binwrite(align_after(0x40))]
    pub ktss_size: u32,
    #[br(align_before(0x40), align_after(0x40))]
    #[binwrite(align_after(0x40))]
    pub ktss: Ktss,
}

// impl BinWrite for KtssSection {
//     fn write_options<W: std::io::Write>(&self, writer: &mut W, options: &WriterOption) -> std::io::Result<()> {
//         (0x70CBCCC5, self).write_options(writer, options)
//     }
// }

// Most of it is absolutely incorrect
#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct InfoSection {
    // Seems related to what subsection_magic is used?
    pub section_size: u32,
    link_id: u32,
    channel_count: u16,
    layer_count: u16,
    padding_1: u32,
    cancel: u32,
    // Not sure but seems to match?
    subsection_magic: u32,
    #[br(count = section_size - 0x1C)]
    unk: Vec<u8>,
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum InfoSubsection {
    #[br(magic = 0x241318u32)]
    Unk1(Unk1InfoSubbection),
    #[br(magic = 0x14AB5u32)]
    Unk2(InfoSection),
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct InfoSubsectionHeader {
    pub unk1: u32,
    pub unk2: u32,
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct Unk1InfoSubbection {
    pub header: InfoSubsectionHeader,
    pub section_size: u32,
    #[br(count = section_size - 0x8)]
    padding: Vec<u8>,
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct PaddingSection {
    pub section_size: u32,
    #[br(count = section_size - 0x8)]
    padding: Vec<u8>,
}

// TODO: Rework this to use a subsection
#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct KtssCompanionSection {
    pub section_size: u32,
    pub link_id: u32,
    #[br(count = 0x10)]
    unknown_1: Vec<u8>,
    pub header_size: u32,
    // This one actually is important and determines what follows, magic for the 0x60 "KTSS companion" subsection is 0x7D43D038
    subsection_magic: u32,
    section_size_2: u32,
    unknown_2: u32,
    pub channel_count: u32,
    transition_related: u32,
    unknown_3: u32,
    pub sample_rate: u32,
    pub sample_count: u32,
    unknown_4: u32,
    pub loop_start: u32,
    #[br(count = 0xC)]
    unknown_5: Vec<u8>,
    pub ktss_offset: u32,
    pub ktss_size: u32,
    unknown_6: u32,
    #[br(count = section_size - 0x60)]
    unk: Vec<u8>,
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct UnknownSection {
    pub section_size: u32,
    link_id: u32,
    #[br(count = section_size - 0x8)]
    unknown_1: Vec<u8>,
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum Section {
    #[br(magic = 0x368C88BDu32)]
    Info1(InfoSection),
    #[br(magic = 0x70CBCCC5u32)]
    Adpcm(KtssCompanionSection),
    // For future Ktsl2stbin parsing
    #[br(magic = 0x15F4D409u32)]
    Ktss(KtssSection),
    #[br(magic = 0xA8DB7261u32)]
    Padding(PaddingSection),
    #[br(magic = 0x368C88BDu32)]
    Unknown(UnknownSection),
}

fn write_sections<W: std::io::Write>(vec: &Vec<Section>, writer: &mut W, options: &WriterOption) -> std::io::Result<()> {
    let mut rooster = Ok(());

    for section in vec {
        rooster = match section {
            Section::Info1(info) => (0x368C88BDu32, info).write_options(writer, options),
            Section::Adpcm(adpcm) => (0x70CBCCC5u32, adpcm).write_options(writer, options),
            Section::Ktss(ktss) => (0x15F4D409u32, ktss).write_options(writer, options),
            Section::Padding(padding) => (0xA8DB7261u32, padding).write_options(writer, options),
            Section::Unknown(unk) => (0x368C88BDu32, unk).write_options(writer, options),
        };
    }

    rooster
}

fn ftell_read<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _: ()) -> BinResult<u32> {
    let current_pos = reader.seek(SeekFrom::Current(0))?;
    Ok(current_pos.try_into().unwrap())
}