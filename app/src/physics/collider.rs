use nalgebra_glm as glm;

pub struct AABBCollider {
    min: glm::TVec3<f64>,
    max: glm::TVec3<f64>,
}

impl AABBCollider {
    pub fn new(min: glm::TVec3<f64>, max: glm::TVec3<f64>) -> Self {
        Self { min, max }
    }

    pub fn is_point_inside(&self, point: &glm::TVec3<f64>) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersect(&self, other: &Self) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

