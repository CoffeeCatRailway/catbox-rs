#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};
use std::time::Instant;
use glam::{vec2, Mat4, Vec2, Vec3};
use crate::graphics::{LineRenderer, Renderable, ShapeRenderer};
use crate::simulation::VerletObject;

pub struct SimpleSolver {
    pub gravity: Vec2,
    pub worldSize: Vec2,

    objects: Vec<Arc<Mutex<VerletObject>>>,

    pub subSteps: u32,
    totalSteps: u32,

    pub pause: bool,
    btnStep: bool,

    time: f32,
    // frameDt: f32, // Use crate::TIME_STEP
    updateTime: f32,
	destroyed: bool,
}

impl SimpleSolver {
    pub fn new(worldSize: Vec2, subSteps: u32) -> Self {
        SimpleSolver {
            gravity: vec2(0.0, 0.0),
            worldSize,

            objects: Vec::new(),

            subSteps,
            totalSteps: 0,

            pause: false,
            btnStep: false,

            time: 0.0,
            updateTime: 0.0,
			destroyed: false,
        }
    }

    pub fn addObject(&mut self, object: Arc<Mutex<VerletObject>>) {
        self.objects.push(object);
    }
	
	fn worldCollision(&mut self, _dt: f32, obj: &mut Arc<Mutex<VerletObject>>, worldSize: Vec2) {
		if let Ok(mut obj) = obj.lock() {
			let halfSize = worldSize * 0.5 - obj.radius;
			let velocity = obj.getVelocity(1.0) * obj.elasticity;
			if obj.position.x < -halfSize.x {
				obj.position.x = -halfSize.x;
				obj.positionLast.x = obj.position.x + velocity.x;
			} else if obj.position.x > halfSize.x {
				obj.position.x = halfSize.x;
				obj.positionLast.x = obj.position.x + velocity.x;
			}
			
			if obj.position.y < -halfSize.y {
				obj.position.y = -halfSize.y;
				obj.positionLast.y = obj.position.y + velocity.y;
			} else if obj.position.y > halfSize.y {
				obj.position.y = halfSize.y;
				obj.positionLast.y = obj.position.y + velocity.y;
			}
		}
	}
	
	fn handleCollision(&mut self, dt: f32) {
        let mut objects = self.objects.clone();
		for obj in objects.iter_mut() {
			// object-object
			// object-line
			self.worldCollision(dt, obj, self.worldSize);
		}
        self.objects = objects;
	}

    fn updateObjects(&mut self, dt: f32) {
        for obj in &self.objects {
			if let Ok(mut obj) = obj.lock() {
				obj.accelerate(self.gravity);
				obj.update(dt);
			}
        }
    }

    fn step(&mut self, dt: f32) {
        // sort
        self.handleCollision(dt);
        // constrain
        self.updateObjects(dt);
    }

    pub fn update(&mut self, dt: f32) {
        if !self.pause || self.btnStep {
            let then = Instant::now();

            self.time += dt;
            let stepDt = dt / self.subSteps as f32;
            for _ in 0..self.subSteps {
                self.step(stepDt);
            }

            let elapsed = then.elapsed().as_secs_f32();
            self.updateTime = elapsed;

            self.totalSteps += 1;
            self.btnStep = false;
        }
    }

    pub fn destroy(&mut self) {
		self.destroyed = true;
    }
	
	pub fn destroyed(&self) -> bool {
		self.destroyed
	}

    pub fn getObjectCount(&self) -> usize {
        self.objects.len()
    }

    pub fn getTotalSteps(&self) -> u32 {
        self.totalSteps
    }

	#[allow(unused)]
    pub fn getTimeElapsed(&self) -> f32 {
        self.time
    }
}

impl Renderable for SimpleSolver {
    fn render(&mut self, dt: f32, pvMatrix: &Mat4, shapeRenderer: &mut ShapeRenderer, lineRenderer: &mut LineRenderer) {
        shapeRenderer.pushBox(Vec2::ZERO, Vec3::splat(0.15), self.worldSize, 0.0, 10.0);
		
		for obj in &self.objects {
			if let Ok(mut obj) = obj.lock() {
				obj.render(dt, pvMatrix, shapeRenderer, lineRenderer);
			}
		}
	}
}
