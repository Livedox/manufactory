use nalgebra_glm as glm;

pub struct PhysicPipiline {

}

impl PhysicPipiline {
    pub fn step<F>(gravity: &glm::TVec3<f64>, custom_steps: Vec<F>)
        where F: FnMut()
    {

    }
}