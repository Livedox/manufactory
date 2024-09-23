use nalgebra_glm as glm;

#[derive(Debug, Clone)]
pub struct Joint {
    pub index: usize,
    pub name: String,
    pub children: Vec<Joint>,

    inverse_bind_transform: glm::Mat4,
}

impl Joint {
    pub fn new(index: usize, name: String, inverse_bind_transform: glm::Mat4) -> Self {
        Self {
            index,
            name,
            children: vec![],

            inverse_bind_transform,
        }
    }

    pub fn add_child(&mut self, joint: Joint) {
        self.children.push(joint);
    }

    pub fn inverse_bind_transform(&self) -> &glm::Mat4 {
        &self.inverse_bind_transform
    }
}