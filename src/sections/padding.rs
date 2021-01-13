#[derive(BinRead, BinWrite, Debug, Default, Clone)]
#[br(little)]
pub struct PaddingSection {
    pub section_size: u32,
    #[br(count = section_size - 0x8)]
    padding: Vec<u8>,
}