use super::{interaction::{BlockInteraction, BlockItem}, block_type::BlockType, light_permeability::LightPermeability};

pub struct BlockPlayer {
    pub item_id: u32,
    pub id: u32,
    pub emission: [u8; 3],
    pub is_light_passing: bool,
    pub block_type: BlockType,
    pub is_additional_data: bool,
    pub is_glass: bool,
}

impl BlockInteraction for BlockPlayer {
    fn id(&self) -> u32 {self.id}
    fn emission(&self) -> &[u8; 3] {&self.emission}
    #[inline]
    fn is_light_passing(&self) -> bool {self.is_light_passing}
    fn block_type(&self) -> &BlockType {&self.block_type}
    fn is_additional_data(&self) -> bool {self.is_additional_data}

    fn is_glass(&self) -> bool {
        self.is_glass
    }
}

impl BlockItem for BlockPlayer {
    fn item_id(&self) -> u32 {self.item_id}
}