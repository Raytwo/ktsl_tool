use binread::{
    BinRead,
};

use binwrite::{
    BinWrite,
};

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub struct InfoSection {
    // Seems related to what subsection_magic is used?
    pub section_size: u32,
    pub link_id: u32,
    channel_count: u16,
    layer_count: u16,
    padding_1: u32,
    cancel: u32,
    pub subsection: InfoSubsection,
}

#[derive(BinRead, Debug, Clone)]
#[br(little)]
pub enum InfoSubsection {
    // #[br(magic = 0x241318u32)]
    // Unk1(Unk1InfoSubsection),
    // Voice groups?
    // #[br(magic = 0x14AB5u32)]
    // Unk2(Unk1InfoSubsection),
    #[br(magic = 0xB7DB4B73u32)]
    Unk3(UnkInfo1Subsection),
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct UnkInfo1Subsection {
    pub unk1: i32,
    pub subsubsection_offset_count_idk: u32,
    // lmao
    pub offset_to_subsubsection_offset: u32,
    pub unk4: [u32;2],
    // Not sure
    pub subsubsection_offset: u32,
    // Padding until the first offset?
    pub unk5: u32,
}

#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct UnkInfo1Subsubsection {
    // 0x4820efc4
    pub section_magic: u32,
    pub section_size: u32,
    // Not sure
    pub link_id: u32,
    pub unk1: u32,
    pub entry_offset_count: u32,
    // Relative to section_magic
    pub entry_offset_section_offset: u32,
    // Honestly not sure here, number of float seems to match with second u8?
    pub some_magic: u32,
    #[br(count = 0x19)]
    pub some_section: Vec<f32>,
    #[br(count = entry_offset_count)]
    pub entry_offsets: Vec<u32>

}