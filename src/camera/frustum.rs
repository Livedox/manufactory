use super::camera_controller::CameraController;
extern crate nalgebra_glm as glm;
pub struct Plane {
    normal: glm::Vec3,
    distance: f32,            
}

impl Plane {
    fn new(normal: &glm::Vec3, point: &glm::Vec3) -> Self {
        let normal = glm::normalize(normal);
        Self {
            normal,
            distance: glm::dot(&normal, point),
        }
    }

    pub fn signed_distance_to_plane(&self, point: &glm::Vec3) -> f32 {
        glm::dot(&self.normal, point) - self.distance
    }
}

pub struct Frustum {
    near_face: Plane,
    far_face: Plane,
    right_face: Plane,
    left_face: Plane,
    top_face: Plane,
    bottom_face: Plane,
}

impl Frustum {
    pub fn new(camera: &CameraController, aspect: f32) -> Self {
        // See
        // https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling
        let front = camera.front();
        let right = camera.right();
        let up = camera.up();
        let position = camera.position();

        let far = camera.far();
        let half_vside = far * (camera.fov()*0.5).tan();
        let half_hside = half_vside * aspect;
        let front_mult_far = far * camera.front();

        Self {
            near_face: Plane::new(front, &(position + camera.near()*front)),
            far_face: Plane::new(&(-front), &(position+front_mult_far)),
            right_face: Plane::new(&glm::cross(&(front_mult_far - right*half_hside), up), position),
            left_face: Plane::new(&glm::cross(up, &(front_mult_far + right*half_hside)), position),
            top_face: Plane::new(&glm::cross(right, &(front_mult_far - up*half_vside)), position),
            bottom_face: Plane::new(&glm::cross(&(front_mult_far + up*half_vside), right), position),
        }
    }


    pub fn is_cube_in(&self, center: &glm::Vec3, half_size: f32) -> bool {
        self.into_iter().all(|p: &Plane| {
            let r = half_size * (p.normal.x.abs() + p.normal.y.abs() + p.normal.z.abs());
            -r <= p.signed_distance_to_plane(center)
        })
    }
}


impl<'a> IntoIterator for &'a Frustum {
    type Item = &'a Plane;
    type IntoIter = FrustumIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FrustumIterator {
            frustum: self,
            index: 0,
        }
    }
}

pub struct FrustumIterator<'a> {
    frustum: &'a Frustum,
    index: usize,
}

impl<'a> Iterator for FrustumIterator<'a> {
    type Item = &'a Plane;
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => &self.frustum.right_face,
            1 => &self.frustum.left_face,
            2 => &self.frustum.bottom_face,
            3 => &self.frustum.top_face,
            4 => &self.frustum.near_face,
            5 => &self.frustum.far_face,
            _ => return None
        };
        self.index += 1;
        Some(result)
    }
}