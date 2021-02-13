use binread::{
    BinRead,
};

use binwrite::{
    BinWrite,
};

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
    Unk1(Unk1InfoSubsection),
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
pub struct Unk1InfoSubsection {
    pub header: InfoSubsectionHeader,
    pub section_size: u32,
    #[br(count = section_size - 0x8)]
    padding: Vec<u8>,
}