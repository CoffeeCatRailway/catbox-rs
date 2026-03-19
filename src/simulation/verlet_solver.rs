use bool_flags::Flags8;
use dear_imgui_rs::{Ui, WindowFlags};
use glam::Vec3;
use crate::graphics::render_manager::Renderable;
use crate::simulation::transform::Transform;
use crate::types::{MeshRef, PhysicalRef, ShaderRef};

pub trait Physical {
	fn transform(&self) -> &Transform;
	
	fn transformMut(&mut self) -> &mut Transform;
	
	fn lastTransform(&self) -> &Transform;
	
	fn lastTransformMut(&mut self) -> &mut Transform;
	
	fn fixed(&self) -> bool;
	
	fn elasticity(&self) -> f32;
	
	fn update(&mut self, dt: f32);
	
	fn accelerate(&mut self, acceleration: Vec3);
	
	fn setVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn addVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn getVelocity(&self, dt: f32) -> Vec3;
}

const F_DESTROYED: u8 = 0;
const F_PAUSED: u8 = 1;
const F_FORCE_STEP: u8 = 2;

pub struct VerletSolver {
	pub gravity: Vec3,
	pub worldSize: Vec3,
	
	physicals: Vec<PhysicalRef>,
	
	subSteps: u32,
	updatesDone: u32,
	
	flags: Flags8,
}

impl VerletSolver {
	pub fn new(worldSize: Vec3) -> VerletSolver {
		let mut flags = Flags8::none();
		flags.set(F_PAUSED);
		Self {
			gravity: Vec3::ZERO,
			worldSize,
			
			physicals: vec![],
			
			subSteps: 8,
			updatesDone: 0,
			
			flags,
		}
	}
	
	pub fn isDestroyed(&self) -> bool {
		self.flags.get(F_DESTROYED)
	}
	
	pub fn isPaused(&self) -> bool {
		self.flags.get(F_PAUSED)
	}
	
	pub fn pause(&mut self, paused: bool) {
		if paused {
			self.flags.set(F_PAUSED);
		} else {
			self.flags.clear(F_PAUSED);
		}
	}
	
	fn isForceStep(&self) -> bool {
		self.flags.get(F_FORCE_STEP)
	}
	
	pub fn forceStep(&mut self, force: bool) {
		if force {
			self.flags.set(F_FORCE_STEP);
		} else {
			self.flags.clear(F_FORCE_STEP);
		}
	}
	
	pub fn addPhysical(&mut self, physical: PhysicalRef) {
		self.physicals.push(physical);
	}
	
	fn collide(&self, dt: f32) {}
	
	fn updatePhysicals(&self, dt: f32) {
		for physical in self.physicals.iter() {
			let mut physical = physical.write().unwrap();
			physical.accelerate(self.gravity);
			physical.update(dt);
		}
	}
	
	fn step(&mut self, dt: f32) {
		self.collide(dt);
		self.updatePhysicals(dt);
	}
	
	pub fn update(&mut self, dt: f32) {
		if !self.isPaused() || self.isForceStep() {
			let stepDt = dt / self.subSteps as f32;
			for _ in 0..self.subSteps {
				self.step(stepDt);
			}
			
			self.updatesDone += 1;
			self.forceStep(false);
		}
	}
	
	pub fn gui(&mut self, ui: &mut Ui, dt: f32) {
		ui.window("Verlet Solver")
			.flags(WindowFlags::ALWAYS_AUTO_RESIZE)
			.build(|| {
				ui.input_float3("Gravity", self.gravity.as_mut()).build();
				ui.separator();
				
				ui.text(format!("Physicals: {}", self.physicals.len()));
				ui.separator();
				
				ui.text(format!("Sub steps: {}\tUpdates done: {}", self.subSteps, self.updatesDone));
				ui.text(format!("Update dt: {}", dt));
				ui.text(format!("Step dt: {}", dt / self.subSteps as f32));
				
				let mut pause = self.isPaused();
				ui.checkbox("Pause", &mut pause);
				self.pause(pause);
				if pause {
					ui.same_line();
					if ui.small_button("Step") {
						self.forceStep(true);
					}
				}
				ui.separator();
			});
	}
	
	pub fn getPhysicals(&self) -> &Vec<PhysicalRef> {
		&self.physicals
	}
}

impl Renderable for VerletSolver {
	fn meshRef(&self) -> &MeshRef {
		todo!()
	}
	
	fn shaderRef(&self) -> &ShaderRef {
		todo!()
	}
}