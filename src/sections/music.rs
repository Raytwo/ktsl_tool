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