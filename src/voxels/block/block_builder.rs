use crate::graphic::complex_object::ComplexObject;

use super::{block_type::BlockType, interaction::BlockInteraction, block_default::BlockDefault, block_player::BlockPlayer, light_permeability::LightPermeability};

pub enum BlockTraitType {
    Default,
    Player,
}


pub struct BlockBuilder {
    pub trait_type: BlockTraitType,
    pub id: u32,
    pub emission: Option<[u8; 3]>,
    pub light_permeability: Option<LightPermeability>,
    pub block_type: Option<BlockType>,
    pub item_id: Option<u32>,
    pub is_additional_data: Option<bool>,
}


impl BlockBuilder {
    pub fn new(id: u32) -> Self {
        BlockBuilder {
            trait_type: BlockTraitType::Default,
            id,
            emission: None,
            light_permeability: None,
            block_type: None,
            item_id: None,
            is_additional_data: None,
        }
    }
    pub fn emission(mut self, emission: [u8; 3]) -> Self {self.emission = Some(emission); self}
    pub fn light_permeability(mut self, light_permeability: LightPermeability) -> Self {
        self.light_permeability = Some(light_permeability);
        self
    }

    pub fn faces(mut self, faces: &[u32]) -> Self {
        let mut new_faces: [u32; 6] = [0; 6];
        faces.iter().cycle().take(6).enumerate().for_each(|(i, f)| {
            new_faces[i] = *f;
        });
        self.block_type = Some(BlockType::Block { faces: new_faces });
        self
    }

    pub fn model_name(mut self, name: String) -> Self {
        self.block_type = Some(BlockType::Model { name });
        self
    }

    pub fn animated_model_name(mut self, name: String) -> Self {
        self.block_type = Some(BlockType::AnimatedModel { name });
        self
    }

    pub fn set_player_trait(mut self) -> Self {
        self.trait_type = BlockTraitType::Player;
        self
    }

    pub fn set_complex_object(mut self, cp: ComplexObject) -> Self {
        self.block_type = Some(BlockType::ComplexObject { cp });
        self
    }

    pub fn set_lp_none(mut self) -> Self {
        self.light_permeability = Some(LightPermeability::NONE);
        self
    }

    pub fn set_additional_data_true(mut self) -> Self {
        self.is_additional_data = Some(true);
        self
    }

    pub fn build(self) -> Box<dyn BlockInteraction + Sync + Send> {
        let id = self.id;
        let emission = self.emission.unwrap_or([0, 0, 0]);
        let light_permeability = self.light_permeability.unwrap_or(LightPermeability::default());
        let block_type = self.block_type.unwrap_or(BlockType::None);
        let item_id = self.item_id.unwrap_or(0);
        let is_additional_data = self.is_additional_data.unwrap_or(false);

        match self.trait_type {
            BlockTraitType::Default => Box::new(BlockDefault {id, emission, light_permeability, block_type, is_additional_data}),
            BlockTraitType::Player => Box::new(BlockPlayer {id, item_id, emission, light_permeability, block_type, is_additional_data}),
        }
    }
}