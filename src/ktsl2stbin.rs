use std::fs::File;
use std::path::Path;
use std::io::BufReader;

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

use binwrite::{
    BinWrite,
};

#[derive(BinRead, BinWrite, Debug)]
#[br(magic = b"KTSR")]
pub struct Ktsr {
    pub section_type: u32,
    pub flags: u16,
    pub platform_id: u16,
    pub game_id: u32,
    pub padding: u64,
    pub decomp_size: u32,
    pub comp_size: u32,
}

#[derive(BinRead, BinWrite, Debug)]
pub struct KtslEntry {
    pub section_type: u32,
    pub section_size: u32,
    pub link_id: u32,
    pub header_size: u32,
    pub ktss_size: u32,
    #[br(align_before(0x40), align_after(0x40))]
    pub ktss: Ktss,
}

#[derive(BinRead, BinWrite, Debug)]
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

#[derive(BinRead, BinWrite, Debug)]
pub struct LopusPacket {
    pub size: u32,
    pub unk: u32,
    #[br(count = size)]
    pub content: Vec<u8>,
}

#[derive(BinWrite, Debug)]
//#[binread(little)]
pub struct Ktsl2stbin {
    pub header: Ktsr,
    //#[br(seek_before = SeekFrom::Start(0x40 as _)]
    // #[binwrite(align(0x40))]
    pub entries: Vec<KtslEntry>,
}

impl Ktsl2stbin {
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::read(&mut BufReader::new(File::open(path)?))
    }

    pub fn unpack<P: AsRef<Path>>(&self, out_dir: P) {
        for ktss in &self.entries {
            let mut file_path = out_dir.as_ref().to_path_buf();
            file_path.push(format!("{:08x}.ktss", ktss.link_id));
            
            let file = std::fs::File::create(&file_path).unwrap();
            let mut writer = std::io::BufWriter::new(file);
            ktss.ktss.write(&mut writer).unwrap();
        }
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