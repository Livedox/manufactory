extern crate nalgebra_glm as glm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraDirection {
    Default,
    Behind,
    Front,
}

impl CameraDirection {
    pub fn next(&mut self) {
        match self {
            CameraDirection::Default => *self = Self::Behind,
            CameraDirection::Behind => *self = Self::Front,
            CameraDirection::Front => *self = Self::Default,
        };
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub(super) direction: CameraDirection,
    pub(super) fov: f32,
    pub(super) near: f32,
    pub(super) far: f32,
    pub(super) position: glm::Vec3,
    pub(super) front: glm::Vec3,
    pub(super) up: glm::Vec3,
    pub(super) right: glm::Vec3,
    pub(super) rotation: glm::Mat4
}

const FRONT: glm::Vec4 = glm::Vec4::new(0.0, 0.0, -1.0, 1.0);
const RIGHT: glm::Vec4 = glm::Vec4::new(1.0, 0.0, 0.0, 1.0);
const UP: glm::Vec4 = glm::Vec4::new(0.0, 1.0, 0.0, 1.0);
impl Camera {
    pub fn new(position: glm::Vec3, fov: f32, near: f32, far: f32) -> Camera {
        let identity = glm::Mat4::identity();
        let front = glm::vec4_to_vec3( &(identity*FRONT) );
        let right = glm::vec4_to_vec3( &(identity*RIGHT) );
        let up = glm::vec4_to_vec3( &(identity*UP ) );

        Camera {
            fov,
            position,
            rotation: identity,
            front,
            right,
            up,
            near,
            far,
            direction: CameraDirection::Default
        }
    }


    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = glm::rotate_z(&self.rotation, z);
        self.rotation = glm::rotate_y(&self.rotation, y);
        self.rotation = glm::rotate_x(&self.rotation, x);

        self.front = glm::vec4_to_vec3(&(self.rotation * FRONT));
        self.right = glm::vec4_to_vec3(&(self.rotation * RIGHT));
        self.up = glm::vec4_to_vec3(&(self.rotation * UP));
    }


    pub fn projection(&self, width: f32, height: f32) -> glm::Mat4 {
        let aspect = width/height.max(1.0);
        glm::perspective(aspect, self.fov, self.near, self.far)
    }


    pub fn view(&self) -> glm::Mat4 {
        if self.direction == CameraDirection::Behind {
            return glm::look_at(&(self.position-self.front*1.5), &(self.position), &self.up);
        }
        if self.direction == CameraDirection::Front {
            return glm::look_at(&(self.position+self.front*1.5), &(self.position), &self.up);
        }
        glm::look_at(&self.position, &(self.position+self.front), &self.up)
    }


    pub fn proj_view(&self, width: f32, height: f32) -> glm::Mat4 {
        self.projection(width, height)*self.view()
    }

    pub fn position(&self) -> &glm::Vec3 { &self.position }
    pub fn front(&self) -> &glm::Vec3 { &self.front }
    pub fn up(&self) -> &glm::Vec3 { &self.up }
    pub fn right(&self) -> &glm::Vec3 { &self.right }
    pub fn position_array(&self) -> [f32; 3] {
        [self.position.x, self.position.y, self.position.z]
    }
    pub fn position_tuple(&self) -> (f32, f32, f32) {
        (self.position.x, self.position.y, self.position.z)
    }
    pub fn front_array(&self) -> [f32; 3] {
        [self.front.x, self.front.y, self.front.z]
    }

    pub fn next_direction(&mut self) {
        self.direction.next();
    }
}