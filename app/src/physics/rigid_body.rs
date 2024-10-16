use nalgebra_glm as glm;

pub struct RigidBody {
    pos: glm::TVec3<f64>,
    rotation: glm::Qua<f64>,
}