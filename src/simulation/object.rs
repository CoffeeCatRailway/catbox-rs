#![allow(non_snake_case)]

use glam::{Mat4, Vec2, Vec3};
use crate::graphics::{LineRenderer, Renderable, ShapeRenderer};

pub struct VerletObject {
    pub position: Vec2,
    pub positionLast: Vec2,
    pub acceleration: Vec2,
    pub color: Vec3,
    pub radius: f32,
    #[allow(unused)]
    pub friction: f32,
    pub elasticity: f32,
    pub fixed: bool,
    pub visible: bool,
}

impl Default for VerletObject {
    fn default() -> Self {
        VerletObject {
            position: Vec2::ZERO,
            positionLast: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            color: Vec3::ONE,
            radius: 10.0,
            friction: 0.0,
            elasticity: 1.0,
            fixed: false,
            visible: true,
        }
    }
}

impl VerletObject {
    pub fn update(&mut self, dt: f32) {
        if self.fixed {
            return;
        }
        let displacement = self.position - self.positionLast;
        self.positionLast = self.position;
        self.position += displacement + self.acceleration * dt * dt;
        self.acceleration = Vec2::ZERO;
    }

    pub fn accelerate(&mut self, acceleration: Vec2) {
        if self.fixed {
            return;
        }
        self.acceleration += acceleration;
    }

    #[allow(unused)]
    pub fn setVelocity(&mut self, velocity: Vec2, dt: f32) {
        if self.fixed {
            return;
        }
        self.positionLast = self.position - velocity * dt;
    }

    #[allow(unused)]
    pub fn addVelocity(&mut self, velocity: Vec2, dt: f32) {
        if self.fixed {
            return;
        }
        self.positionLast -= velocity * dt;
    }

    pub fn getVelocity(&self, dt: f32) -> Vec2 {
        (self.position - self.positionLast) / dt
    }
}

impl Renderable for VerletObject {
    fn render(&self, dt: f32, _pvMatrix: &Mat4, shapeRenderer: &mut ShapeRenderer, lineRenderer: &mut LineRenderer) {
        if self.visible {
            shapeRenderer.pushCircle(self.position, self.color, self.radius, 0.0);

            let velocity = self.getVelocity(dt);
            let color = 1.0 - self.color;
            lineRenderer.pushLine2(self.position, color, self.position + velocity, color);
        }
    }
}
