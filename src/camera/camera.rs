extern crate nalgebra_glm as glm;

#[derive(Debug)]
pub struct Camera {
    base_front: glm::Vec4,
    base_up: glm::Vec4,
    base_right: glm::Vec4,

    fov: f32,
    pub(super) position: glm::Vec3,
    pub(super) front: glm::Vec3,
    pub(super) up: glm::Vec3,
    pub(super) right: glm::Vec3,
    pub(super) rotation: glm::Mat4
}

impl Camera {
    pub fn new(position: glm::Vec3, fov: f32) -> Camera {
        let identity = glm::Mat4::identity();
        let base_front = glm::vec4(0.0, 0.0, -1.0, 1.0);
        let base_right = glm::vec4(1.0, 0.0, 0.0, 1.0);
        let base_up = glm::vec4(0.0, 1.0, 0.0, 1.0);

        let front = glm::vec4_to_vec3( &(identity*base_front) );
        let right = glm::vec4_to_vec3( &(identity*base_right) );
        let up = glm::vec4_to_vec3( &(identity*base_up ) );

        Camera {
            fov,
            position,
            rotation: identity,
            front,
            right,
            up,

            base_front,
            base_right,
            base_up,
        }
    }


    pub fn rotate(&mut self, x: f32, y: f32, z:f32) {
        self.rotation = glm::rotate_z(&self.rotation, z);
        self.rotation = glm::rotate_y(&self.rotation, y);
        self.rotation = glm::rotate_x(&self.rotation, x);

        self.front = glm::vec4_to_vec3(&(self.rotation * self.base_front));
        self.right = glm::vec4_to_vec3(&(self.rotation * self.base_right));
        self.up = glm::vec4_to_vec3(&(self.rotation * self.base_up));
    }


    pub fn projection(&self, width: f32, height: f32) -> glm::Mat4 {
        glm::perspective(width/height, self.fov, 0.1, 1000.)
    }


    pub fn view(&self) -> glm::Mat4 {
        glm::look_at(&self.position, &(self.position+self.front), &self.up)
    }


    pub fn proj_view(&self, width: f32, height: f32) -> glm::Mat4 {
        self.projection(width, height)*self.view()
    }

    pub fn position(&self) -> &glm::Vec3 { &self.position }
    pub fn front(&self) -> &glm::Vec3 { &self.front }
    pub fn position_array(&self) -> [f32; 3] {
        [self.position.x, self.position.y, self.position.z]
    }
    pub fn front_array(&self) -> [f32; 3] {
        [self.front.x, self.front.y, self.front.z]
    }
}