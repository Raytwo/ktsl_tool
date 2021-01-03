use std::fs::File;
use std::ffi::OsStr;
use std::path::Path;
use std::io::BufReader;

extern crate stopwatch;
use stopwatch::Stopwatch;

use binread::{
    io::{
        Read,
        Seek,
        SeekFrom
    },
    BinRead,
    ReadOptions,
    BinResult,
};

use std::io::Write;

use binwrite::{
    BinWrite,
    WriterOption,
};

use jwalk::WalkDir;

use rayon::prelude::*;

use crate::ktsl2asbin::Ktsl2asbin;

pub const KTSL_HEADER_SIZE: u32 =  0x40;

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
pub struct Ktsr {
    pub magic: [u8;4],
    pub section_type: u32,
    pub flags: u16,
    pub platform_id: u16,
    pub game_id: u32,
    pub padding: u64,
    pub decomp_size: u32,
    #[binwrite(align_after(0x40))]
    pub comp_size: u32,
}

impl Ktsr {
    pub fn new() -> Self {
        // Temp
        Ktsr {
            magic: *b"KTSR",
            section_type: 0xFCDD9402,
            flags: 1,
            platform_id: 0x400,
            game_id: 0xB75674CE,
            .. Default::default()
        }
    }
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
pub struct KtslEntry {
    pub section_type: u32,
    pub section_size: u32,
    pub link_id: u32,
    pub header_size: u32,
    #[binwrite(align_after(0x40))]
    pub ktss_size: u32,
    #[br(align_before(0x40), align_after(0x40))]
    #[binwrite(align_after(0x40))]
    pub ktss: Ktss,
}

impl KtslEntry {
    pub fn new() -> Self {
        KtslEntry {
            section_type: 0,
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

#[derive(BinWrite, Debug, Default)]
pub struct Ktsl2stbin {
    pub header: Ktsr,
    //#[br(seek_before = SeekFrom::Start(0x40 as _)]
    #[binwrite(align(0x40))]
    pub entries: Vec<KtslEntry>,
}

impl Ktsl2stbin {
    pub fn new() -> Self {
        Ktsl2stbin {
            header: Ktsr::new(),
            entries: vec![],
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::read(&mut BufReader::new(File::open(path)?))
    }

    /// **Warning**: gross
    pub fn pack<P: AsRef<Path>>(&mut self, dir: P, mut asbin: Option<Box<Ktsl2asbin>>) {
        println!("Starting to pack...");
        
        let sw = Stopwatch::start_new();

        let mut ktsl2asbin = match asbin {
            Some(pingas) => {
                pingas
            },
            None => Box::new(Ktsl2asbin::new()),
        };

        let mut sections = ktsl2asbin.get_companion_sections();

        // Ignore the KTSR header
        let mut ktsl_offset = 0x40;

        //for entry in WalkDir::new(&dir)
        sections.iter_mut().for_each(|section| {
            let ktss = match Ktss::open(format!("{}/{:08x}.ktss", dir.as_ref().display(), section.link_id)) {
                Ok(ktss) => ktss,
                // TODO: Make this better
                Err(err) => {
                    panic!(err);
                },
            };

            // Some align required, should probably be made into a preprocessor?
            let section_size = if (ktss.section_size + KTSL_HEADER_SIZE) % 0x40 != 0 {
                ktss.section_size + KTSL_HEADER_SIZE + ( 0x40 - ((ktss.section_size + KTSL_HEADER_SIZE) % 0x40))
            } else {
                ktss.section_size + KTSL_HEADER_SIZE
            };

            ktsl_offset += KTSL_HEADER_SIZE;

            let ktsl = KtslEntry {
                // Less gross
                //link_id: u32::from_str_radix(entry.path().file_stem().and_then(|s: &OsStr| s.to_str()).map_or("0", |name| name), 16).unwrap(),
                link_id: section.link_id,
                // TODO: Turn this into a const or enum
                section_type: 0x15F4D409,
                section_size,
                ktss_size: ktss.section_size,
                ktss,
                .. KtslEntry::new()
            };

            let ktss_companion = Some(section);

            if let Some(companion) = ktss_companion {
                companion.ktss_size = ktsl.ktss.section_size;
                companion.loop_start = if ktsl.ktss.loop_start == 0 { 0xFFFFFFFF } else { ktsl.ktss.loop_start };
                companion.sample_count = ktsl.ktss.sample_count;
                companion.sample_rate = ktsl.ktss.sample_rate;
                companion.ktss_offset = ktsl_offset;
            }

            ktsl_offset += section_size - KTSL_HEADER_SIZE;

            self.entries.push(ktsl);
        });

        ktsl2asbin.pack();

        println!("Packing took {} secs", sw.elapsed().as_secs());

        self.header.decomp_size = ktsl_offset;
        self.header.comp_size = ktsl_offset;

        // Test
        let file = std::fs::File::create(&"./out.ktsl2stbin").unwrap();
        let mut writer = std::io::BufWriter::new(file);
        self.write(&mut writer).unwrap();
    }

    pub fn unpack(&self, out_dir: &Path) {
        &self.entries.par_iter().for_each(|ktss| {
            let mut file_path = out_dir.to_path_buf();
            file_path.push(format!("{:08X}.ktss", ktss.link_id));
            
            let file = std::fs::File::create(&file_path).unwrap();
            let mut writer = std::io::BufWriter::new(file);
            ktss.ktss.write(&mut writer).unwrap();
        });
    }
}

impl BinRead for Ktsl2stbin {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _options: &ReadOptions, _args: Self::Args) -> BinResult<Self> {
        let mut ktsl2stbin = Ktsl2stbin {
            header: Ktsr::read(reader)?,
            entries: vec![],
        };

        reader.seek(SeekFrom::Start(0x40)).unwrap();

        while ktsl2stbin.header.decomp_size != binread::io::Seek::seek(reader, SeekFrom::Current(0))? as u32 {
            ktsl2stbin.entries.push(KtslEntry::read(reader).unwrap());
        }

        Ok(ktsl2stbin)
    }
}