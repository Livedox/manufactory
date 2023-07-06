use crate::{input_event::{input_service::{InputService, Key}, KeypressState}, my_time::Time};
use cgmath::num_traits::Float;
use nalgebra_glm as glm;

use super::camera::Camera;

pub struct CameraController {
    yaw: f32,
    pitch: f32,
    camera: Camera,
}


impl CameraController {
    const SPEED: f32 = 14.0;
    const SENSETIV: f32 = 0.3;
    pub fn new(position: glm::Vec3, fov: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            camera: Camera::new(position, fov)
        }
    }

    pub fn update(&mut self, input: &InputService, window_height: f32, time: &Time, is_cursor: bool) {
        let mut delta = time.delta();

        if input.is_keys(&[(Key::W, KeypressState::AnyStayPress)]) {
            self.camera.position +=  self.camera.front * Self::SPEED * delta;
        }
        if input.is_keys(&[(Key::S, KeypressState::AnyStayPress)]) {
            self.camera.position -=  self.camera.front * Self::SPEED * delta;
        }
        if input.is_keys(&[(Key::A, KeypressState::AnyStayPress)]) {
            self.camera.position -=  self.camera.right * Self::SPEED * delta;
        }
        if input.is_keys(&[(Key::D, KeypressState::AnyStayPress)]) {
            self.camera.position +=  self.camera.right * Self::SPEED * delta;
        }

        //I don't know how to fix it
        //When the amount of fps increases the camera slows down,
        //and without delta time the camera slows down at low fps
        //So I put a limiter here
        if delta < 0.0033 { delta = 0.002 };
        if delta < 0.0042 { delta = 0.003 };
        if delta < 0.0083 { delta = 0.0083 };
        if delta > 0.033 { delta = 0.033 };
        if !is_cursor {
            self.camera.rotation = glm::Mat4::identity();
            self.yaw -= input.delta().0*Self::SENSETIV*delta;
            self.pitch -= input.delta().1*Self::SENSETIV*delta;
            if self.pitch > 1.569_051 {self.pitch = 1.569_051}
            if self.pitch < -1.569_051 {self.pitch = -1.569_051}
            self.camera.rotate(self.pitch, self.yaw, 0.); 
        }
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
    pub fn position(&self) -> &glm::Vec3 {&self.camera.position()}
    pub fn front(&self) -> &glm::Vec3 {&self.camera.front()}
}