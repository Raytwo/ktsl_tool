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