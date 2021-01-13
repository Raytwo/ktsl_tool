#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct UnknownSection {
    pub section_size: u32,
    link_id: u32,
    #[br(count = section_size - 0x8)]
    unknown_1: Vec<u8>,
}