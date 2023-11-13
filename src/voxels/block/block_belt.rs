use crate::world::xyz::XYZ;

use super::{interaction::BlockInteraction, block_type::BlockType, light_permeability::LightPermeability};

pub struct BlockBelt {
    pub id: u32,
    pub emission: [u8; 3],
    pub light_permeability: LightPermeability,
    pub block_type: BlockType,
    pub is_additional_data: bool,
}

impl BlockInteraction for BlockBelt {
    fn id(&self) -> u32 {self.id}
    fn emission(&self) -> &[u8; 3] {&self.emission}
    fn light_permeability(&self) -> LightPermeability {self.light_permeability}
    fn block_type(&self) -> &BlockType {&self.block_type}
    fn is_additional_data(&self) -> bool {self.is_additional_data}

    fn min_point(&self) -> &XYZ {
        &XYZ(0.0, 0.0, 0.0)
    }
    fn max_point(&self) -> &XYZ {
        &XYZ(1.0, 0.25, 1.0)
    }
    fn is_voxel_size(&self) -> bool {true}
}