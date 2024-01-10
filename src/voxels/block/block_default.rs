use super::{interaction::BlockInteraction, block_type::BlockType, light_permeability::LightPermeability};


pub struct BlockDefault {
    pub id: u32,
    pub emission: [u8; 3],
    pub is_light_passing: bool,
    pub block_type: BlockType,
    pub is_additional_data: bool,
}


impl BlockInteraction for BlockDefault {
    fn id(&self) -> u32 {self.id}
    fn emission(&self) -> &[u8; 3] {&self.emission}
    #[inline]
    fn is_light_passing(&self) -> bool {self.is_light_passing}
    fn block_type(&self) -> &BlockType {&self.block_type}
    fn is_additional_data(&self) -> bool {self.is_additional_data}
}