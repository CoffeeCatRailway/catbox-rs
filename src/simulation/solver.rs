use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use bool_flags::Flags8;
use dear_imgui_rs::{TreeNodeFlags, Ui, WindowFlags};
use glam::Vec3;
use crate::simulation::region::AABB;
use crate::simulation::Transform;
use crate::types::PhysicalRef;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn newId() -> usize {
	ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub trait Physical: Debug + Send + Sync {
	fn id(&self) -> usize;
	
	fn transform(&self) -> &Transform;
	fn transformMut(&mut self) -> &mut Transform;
	
	fn transformPrev(&self) -> &Transform;
	fn transformPrevMut(&mut self) -> &mut Transform;
	
	fn update(&mut self, dt: f32);
	
	fn applyForce(&mut self, force: Vec3);
	
	fn getVelocity(&self, dt: f32) -> Vec3;
	fn setVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn color(&self) -> Vec3;
	
	fn bounds(&self) -> AABB;
}

const F_DESTROYED: u8 = 0;
const F_PAUSED: u8 = 1;
const F_FORCE_STEP: u8 = 2;

pub struct Solver {
	pub gravity: Vec3,
	flags: Flags8,
	
	physicals: HashMap<usize, PhysicalRef>,
	
	subSteps: u32,
	updatesDone: usize,
	
	subStepTimeAccum: f32,
	subStepTime: f32,
	
	stepTime: f32,
}

impl Solver {
	pub fn new() -> Result<Solver, String> {
		let mut flags = Flags8::none();
		flags.set(F_PAUSED);
		
		Ok(Self {
			gravity: Vec3::ZERO,
			flags,
			
			physicals: HashMap::new(),
			
			subSteps: 8,
			updatesDone: 0,
			
			subStepTimeAccum: 0.0,
			subStepTime: 0.0,
			
			stepTime: 0.0,
		})
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
	
	pub fn forceStep(&mut self) {
		self.flags.set(F_FORCE_STEP);
	}
	
	pub fn addPhysical(&mut self, physical: PhysicalRef) {
		let id = {
			let borrow = physical.read().unwrap();
			borrow.id()
		};
		self.physicals.insert(id, physical);
	}
	
	pub fn getPhysicals(&self) -> &HashMap<usize, PhysicalRef> {
		&self.physicals
	}
	
	fn subStep(&mut self, dt: f32) {
		let now = Instant::now();
		
		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.subStepTimeAccum += end;
	}
	
	pub fn update(&mut self, dt: f32) {
		if self.flags.get(F_DESTROYED) {
			return;
		}
		
		let forceStep = self.flags.get(F_FORCE_STEP);
		if !self.isPaused() || forceStep {
			let now = Instant::now();
			
			let subSteps = self.subSteps;
			let subStepDt = dt / subSteps as f32;
			
			for _ in 0..subSteps {
				self.subStep(subStepDt);
			}
			
			self.subStepTime = self.subStepTimeAccum / self.subSteps as f32;
			self.subStepTimeAccum = 0.0;
			
			let end = now.elapsed().as_secs_f32() * 1000.0;
			self.stepTime = end;
			
			self.updatesDone += 1;
			if forceStep {
				self.flags.clear(F_FORCE_STEP);
			}
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
			  
			  ui.text(format!("Sub steps: {}", self.subSteps));
			  ui.text(format!("Updates: {}", self.updatesDone));
			  ui.text(format!("Update dt: {}", dt));
			  ui.text(format!("Sub step dt: {}", dt / self.subSteps as f32));
			  
			  let mut pause = self.isPaused();
			  ui.checkbox("Pause", &mut pause);
			  self.pause(pause);
			  if pause {
				  ui.same_line();
				  if ui.small_button("Step") {
					  self.forceStep();
				  }
			  }
			  ui.separator();
			  
			  if ui.collapsing_header("Times", TreeNodeFlags::COLLAPSING_HEADER) {
				  ui.text(format!("Sub step time*: {}ms", self.subStepTime));
				  ui.text(format!("Step time: {}ms", self.stepTime));
			  }
		  });
	}
	
	pub fn destroy(&mut self) {
		self.flags.set(F_DESTROYED);
		// self.threadPool.stopAll();
	}
}
