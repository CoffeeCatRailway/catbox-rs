#![allow(non_snake_case)]

use glam::{Mat4, Quat, Vec3};

#[allow(unused)]
#[derive(Copy, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3, // probably won't use
    pub front: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            front: Vec3::NEG_Z,
            right: Vec3::X,
            up: Vec3::Y,
        }
    }
}

#[allow(unused)]
impl Transform {
    pub fn getPositionMatrix(&self) -> Mat4 {
        Mat4::from_translation(-self.position)
    }

    pub fn getRotationMatrix(&self) -> Mat4 {
        Mat4::from_quat(self.rotation)
    }

    pub fn getScaleMatrix(&self) -> Mat4 {
        Mat4::from_scale(self.scale)
    }

    pub fn getWorldMatrix(&self) -> Mat4 {
        self.getPositionMatrix() * self.getRotationMatrix()
    }

    pub fn getViewMatrix(&self) -> Mat4 {
        self.getRotationMatrix() * self.getPositionMatrix()
    }

    pub fn calcFront(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::NEG_Z)
    }

    pub fn calcRight(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::X)
    }

    pub fn calcUp(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::Y)
    }

    pub fn updateLocalVectors(&mut self) {
        self.front = self.calcFront();
        self.right = self.calcRight();
        self.up = self.calcUp();
    }

    pub fn moveForward(&mut self, dt: f32) {
        self.position += self.front * dt;
    }

    pub fn moveBackward(&mut self, dt: f32) {
        self.position -= self.front * dt;
    }

    pub fn moveLeft(&mut self, dt: f32) {
        self.position -= self.right * dt;
    }

    pub fn moveRight(&mut self, dt: f32) {
        self.position += self.right * dt;
    }

    pub fn moveUp(&mut self, dt: f32) {
        self.position += self.up * dt;
    }

    pub fn moveDown(&mut self, dt: f32) {
        self.position -= self.up * dt;
    }
}
