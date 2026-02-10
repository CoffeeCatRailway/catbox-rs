#![allow(non_snake_case)]

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use dear_imgui_rs::{TreeNodeFlags, Ui, WindowFlags};
use glam::{vec2, Mat4, Vec2, Vec3};
use crate::graphics::{LineRenderer, Renderable, ShapeRenderer};
use crate::simulation::VerletObject;
use crate::STEP_DT;

pub struct SimpleSolver {
    pub gravity: Vec2,
    pub worldSize: Vec2,

    objects: Vec<Arc<Mutex<VerletObject>>>,

    subSteps: u32,
    totalSteps: u32,

    pub pause: bool,
    pub btnStep: bool,

    totalTimeElapsed: f32,
    fullUpdateTime: f32,
	stepTime: f32,
	sortTime: f32,
	collisionTime: f32,
	objUpdateTime: f32,
	
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

            pause: true,
            btnStep: false,

            totalTimeElapsed: 0.0,
            fullUpdateTime: 0.0,
			stepTime: 0.0,
			sortTime: 0.0,
			collisionTime: 0.0,
			objUpdateTime: 0.0,
			
			destroyed: false,
        }
    }

    pub fn addObject(&mut self, object: Arc<Mutex<VerletObject>>) {
        self.objects.push(object);
    }
	
	fn sortObjects(&mut self) {
		self.objects.sort_by(|o1, o2| {
			let o1 = o1.lock().unwrap();
			let o2 = o2.lock().unwrap();
			let o1 = o1.position.x - o1.radius;
			let o2 = o2.position.x - o2.radius;
			o1.total_cmp(&o2)
		});
	}
	
	fn solveObjectObjectCollision(&self, obj1: &mut MutexGuard<VerletObject>, obj2: &mut MutexGuard<VerletObject>) {
		let dir = obj1.position - obj2.position;
		let dist = dir.length();
		let minDist = obj1.radius + obj2.radius;
		if dist < minDist {
			let mut dir = dir.normalize();
			if dist <= f32::EPSILON {
				dir = Vec2::X;
			}
			
			let massRatio1 = obj1.radius / minDist;
			let massRatio2 = obj2.radius / minDist;
			let force = 0.5 * ((obj1.elasticity + obj2.elasticity) * 0.5) * (dist - minDist);
			
			if !obj1.fixed {
				obj1.position -= dir * massRatio2 * force;
			}
			if !obj2.fixed {
				obj2.position += dir * massRatio1 * force;
			}
		}
	}
	
	fn worldCollision(&self, _dt: f32, obj: &mut MutexGuard<VerletObject>) {
		let halfSize = self.worldSize * 0.5 - obj.radius;
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
	
	fn handleCollision(&self, dt: f32) {
		let mut i: usize = 0;
		for obj1 in self.objects.iter() {
			i += 1;
			let mut obj1 = obj1.lock().unwrap();
			for obj2 in self.objects.iter().skip(i) {
				let mut obj2 = obj2.lock().unwrap();
				if (obj2.position.x - obj2.radius) > (obj1.position.x + obj1.radius) {
					break;
				}
				self.solveObjectObjectCollision(&mut obj1, &mut obj2);
			}
			
			self.worldCollision(dt, &mut obj1);
		}
	}

    fn updateObjects(&self, dt: f32) {
        for obj in self.objects.iter() {
			let mut obj = obj.lock().unwrap();
			obj.accelerate(self.gravity);
			obj.update(dt);
        }
    }

    fn step(&mut self, dt: f32) {
		let mut start = Instant::now();
        self.sortObjects();
		self.sortTime += start.elapsed().as_secs_f32();
		
		start = Instant::now();
        self.handleCollision(dt);
		self.collisionTime += start.elapsed().as_secs_f32();
		
        // constrain
		
		start = Instant::now();
        self.updateObjects(dt);
		self.objUpdateTime += start.elapsed().as_secs_f32();
    }

    pub fn update(&mut self, dt: f32) {
        if !self.pause || self.btnStep {
			self.stepTime = 0.0;
			self.sortTime = 0.0;
			self.collisionTime = 0.0;
			self.objUpdateTime = 0.0;
			
            let start = Instant::now();

            self.totalTimeElapsed += dt;
            let stepDt = dt / self.subSteps as f32;
            for _ in 0..self.subSteps {
				let start = Instant::now();
                self.step(stepDt);
				self.stepTime += start.elapsed().as_secs_f32();
            }
			
			let elapsed = start.elapsed().as_secs_f32();
			self.fullUpdateTime = elapsed;
			
			let recip = 1.0 / (self.subSteps as f32);
			self.stepTime *= recip;
			self.sortTime *= recip;
			self.collisionTime *= recip;
			self.objUpdateTime *= recip;

            self.totalSteps += 1;
            self.btnStep = false;
        }
    }
	
	pub fn gui(&mut self, ui: &mut Ui) {
		ui.window("Solver")
		  .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
		  .build(|| {
			  ui.input_float2("Gravity", self.gravity.as_mut()).build();
			  ui.separator();
			  
			  ui.text(format!("Objects: {}", self.getObjectCount()));
			  ui.separator();
			  
			  ui.text(format!("Sub steps: {}\tTotal steps: {}", self.subSteps, self.totalSteps));
			  ui.text(format!("Update dt: {}", STEP_DT));
			  ui.text(format!("Step dt: {}", STEP_DT / self.subSteps as f32));
			  ui.checkbox("Pause", &mut self.pause);
			  if self.pause {
				  ui.same_line();
				  if ui.small_button("Step") {
					  self.btnStep = true;
				  }
			  }
			  ui.separator();
			  
			  if ui.collapsing_header("Times", TreeNodeFlags::COLLAPSING_HEADER) {
				  ui.text(format!("Total time elapsed: {}s", self.totalTimeElapsed));
				  ui.text(format!("Full update time: {}ms", self.fullUpdateTime * 1000.0));
				  
				  ui.text(format!("Step time (avg): {}ms", self.stepTime * 1000.0));
				  let mut hovered = ui.is_item_hovered();
				  
				  ui.text(format!("Sort time (avg): {}ms", self.sortTime * 1000.0));
				  hovered |= ui.is_item_hovered();
				  
				  ui.text(format!("Collision time (avg): {}ms", self.collisionTime * 1000.0));
				  hovered |= ui.is_item_hovered();
				  
				  ui.text(format!("Obj update time (avg): {}ms", self.objUpdateTime * 1000.0));
				  hovered |= ui.is_item_hovered();
				  
				  if hovered {
					  let combined = (self.stepTime + self.sortTime + self.collisionTime + self.objUpdateTime) * 1000.0;
					  ui.tooltip_text(format!("Averaged out over sub steps\nCombined averages: {}ms", combined));
				  }
			  }
		  });
	}

    pub fn destroy(&mut self) {
		self.destroyed = true;
    }

    pub fn getObjectCount(&self) -> usize {
        self.objects.len()
    }

    pub fn getTotalSteps(&self) -> u32 {
        self.totalSteps
    }
}

impl Renderable for SimpleSolver {
    fn render(&self, dt: f32, pvMatrix: &Mat4, shapeRenderer: &mut ShapeRenderer, lineRenderer: &mut LineRenderer) {
        shapeRenderer.pushBox(Vec2::ZERO, Vec3::splat(0.15), self.worldSize, 0.0, 10.0);
		
		for obj in self.objects.iter() {
			obj.lock().unwrap().render(dt, pvMatrix, shapeRenderer, lineRenderer);
		}
	}
}
