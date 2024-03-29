use crate::input_event::{input_service::{InputService, Key}, KeypressState};
use nalgebra_glm as glm;

use super::{camera::Camera, frustum::Frustum};

#[derive(Debug, Clone)]
pub struct CameraController {
    yaw: f32,
    pitch: f32,
    camera: Camera
}


impl CameraController {
    const SPEED: f32 = 14.0; //14.0 default
    const SENSETIV: f32 = 0.3;
    pub fn new(position: glm::Vec3, fov: f32, near: f32, far: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            camera: Camera::new(position, fov, near, far)
        }
    }

    pub fn set_angle(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch;
    }

    pub fn update_rotation(&mut self, mouse_delta_x: f32, mouse_delta_y: f32, delta_time: f32) {
        self.camera.rotation = glm::Mat4::identity();
        self.yaw -= mouse_delta_x*Self::SENSETIV*delta_time;
        self.pitch -= mouse_delta_y*Self::SENSETIV*delta_time;
        if self.pitch > 1.569_051 {self.pitch = 1.569_051}
        if self.pitch < -1.569_051 {self.pitch = -1.569_051}
        self.camera.rotate(self.pitch, self.yaw, 0.0);
    }

    pub fn update(&mut self, input: &InputService, delta_time: f32, is_cursor: bool) {
        if !is_cursor {
            self.camera.rotation = glm::Mat4::identity();
            self.yaw -= input.delta().0*Self::SENSETIV*delta_time;
            self.pitch -= input.delta().1*Self::SENSETIV*delta_time;
            if self.pitch > 1.569_051 {self.pitch = 1.569_051}
            if self.pitch < -1.569_051 {self.pitch = -1.569_051}
            self.camera.rotate(self.pitch, self.yaw, 0.0); 
        }

        if input.is_key(&Key::KeyW, KeypressState::AnyStayPress) {
            self.camera.position +=  self.camera.front * Self::SPEED * delta_time;
        }
        if input.is_key(&Key::KeyS, KeypressState::AnyStayPress) {
            self.camera.position -=  self.camera.front * Self::SPEED * delta_time;
        }
        if input.is_key(&Key::KeyA, KeypressState::AnyStayPress) {
            self.camera.position -=  self.camera.right * Self::SPEED * delta_time;
        }
        if input.is_key(&Key::KeyD, KeypressState::AnyStayPress) {
            self.camera.position +=  self.camera.right * Self::SPEED * delta_time;
        }
    }

    pub fn set_position(&mut self, position: glm::Vec3) {
        self.camera.position = position;
    }

    pub fn projection(&self, width: f32, height: f32) -> glm::Mat4 {
        self.camera.projection(width, height)
    }
    pub fn view(&self) -> glm::Mat4 {
        self.camera.view()
    }
    pub fn proj_view(&self, width: f32, height: f32) -> glm::Mat4 {
        self.camera.proj_view(width, height)
    }
    pub fn position(&self) -> &glm::Vec3 {self.camera.position()}
    pub fn front(&self) -> &glm::Vec3 {self.camera.front()}
    pub fn position_array(&self) -> [f32; 3] {self.camera.position_array()}
    pub fn position_tuple(&self) -> (f32, f32, f32) {self.camera.position_tuple()}
    pub fn front_array(&self) -> [f32; 3] {self.camera.front_array()}
    pub fn up(&self) -> &glm::Vec3 {self.camera.up()}
    pub fn right(&self) -> &glm::Vec3 {self.camera.right()}
    pub fn near(&self) -> f32 {self.camera.near}
    pub fn far(&self) -> f32 {self.camera.far}
    pub fn fov(&self) -> f32 {self.camera.fov}
    pub fn yaw(&self) -> f32 {self.yaw}
    pub fn pitch(&self) -> f32 {self.pitch}

    pub fn new_frustum(&self, aspect: f32) -> Frustum {
        Frustum::new(self, aspect)
    }
}