use bool_flags::Flags8;
use dear_imgui_rs::{Ui, WindowFlags};
use glam::{vec3, Mat4, Vec3};
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::render_manager::Renderable;
use crate::simulation::transform::Transform;
use crate::types::{newMeshRef, GlRef, MeshRef, PhysicalRef, ShaderRef};

#[allow(unused)]
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
	
	fn getColor(&self) -> Vec3; // todo: move shape and collision to separate components
}

// const F_DESTROYED: u8 = 0;
const F_PAUSED: u8 = 1;
const F_FORCE_STEP: u8 = 2;

pub struct VerletSolver {
	mesh: MeshRef,
	shader: ShaderRef,
	
	pub gravity: Vec3,
	pub worldSize: Vec3,
	
	physicals: Vec<PhysicalRef>,
	
	subSteps: u32,
	updatesDone: u32,
	
	flags: Flags8,
}

impl VerletSolver {
	pub fn new(worldSize: Vec3, gl: GlRef, shader: ShaderRef) -> Result<VerletSolver, String> {
		let mesh = {
			let vertices = vec![
				Vertex {
					position: vec3(-0.5, 0.5, 0.0),
					color: Vec3::splat(0.15),
				},
				Vertex {
					position: vec3(0.5, 0.5, 0.0),
					color: Vec3::splat(0.15),
				},
				Vertex {
					position: vec3(0.5, -0.5, 0.0),
					color: Vec3::splat(0.15),
				},
				Vertex {
					position: vec3(-0.5, -0.5, 0.0),
					color: Vec3::splat(0.15),
				}
			];
			let indices = vec![0, 1, 2, 2, 3, 0];
			let mut mesh = Mesh::simple(gl, vertices, Some(indices));
			mesh.upload(shader.clone())?;
			newMeshRef(mesh)
		};
		
		let mut flags = Flags8::none();
		flags.set(F_PAUSED);
		Ok(Self {
			mesh,
			shader,
			
			gravity: Vec3::ZERO,
			worldSize,
			
			physicals: vec![],
			
			subSteps: 8,
			updatesDone: 0,
			
			flags,
		})
	}
	
	// pub fn isDestroyed(&self) -> bool {
	// 	self.flags.get(F_DESTROYED)
	// }
	
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
	
	fn sortPhysicals(&mut self) {
		self.physicals.sort_by(|a, b| {
			let a = a.borrow();
			let b = b.borrow();
			
			let p1 = a.transform().position.x - a.transform().scale.x;
			let p2 = b.transform().position.x - b.transform().scale.x;
			
			p1.total_cmp(&p2)
		});
	}
	
	fn collideWithPhysical(&self, physical1: PhysicalRef, physical2: PhysicalRef) {
		if let Ok(mut physical1) = physical1.try_borrow_mut() {
			if let Ok(mut physical2) = physical2.try_borrow_mut() {
				let r1 = physical1.transform().scale.x * 0.5;
				let r2 = physical2.transform().scale.x * 0.5;
				
				let dir = physical1.transform().position - physical2.transform().position;
				let dist = dir.length();
				let minDist = r1 + r2;
				if dist < minDist {
					let mut dir = dir.normalize();
					if dist <= f32::EPSILON {
						dir = Vec3::X;
					}
					
					let massRatio1 = r1 / minDist;
					let massRatio2 = r2 / minDist;
					let force = 0.5 * ((physical1.elasticity() + physical2.elasticity()) * 0.5) * (dist - minDist);
					
					if !physical1.fixed() {
						physical1.transformMut().position -= dir * massRatio2 * force;
					}
					if !physical2.fixed() {
						physical2.transformMut().position += dir * massRatio1 * force;
					}
				}
			}
		}
	}
	
	fn collideWithBoundary(&self, _dt: f32, physical: PhysicalRef) {
		if let Ok(mut physical) = physical.try_borrow_mut() {
			let halfSize = (self.worldSize - physical.transform().scale.x) * 0.5;
			let velocity = physical.getVelocity(1.0) * physical.elasticity();
			
			if physical.transform().position.x < -halfSize.x {
				physical.transformMut().position.x = -halfSize.x;
				physical.lastTransformMut().position.x = -halfSize.x + velocity.x;
			} else if physical.transform().position.x > halfSize.x {
				physical.transformMut().position.x = halfSize.x;
				physical.lastTransformMut().position.x = halfSize.x + velocity.x;
			}
			
			if physical.transform().position.y < -halfSize.y {
				physical.transformMut().position.y = -halfSize.y;
				physical.lastTransformMut().position.y = -halfSize.y + velocity.y;
			} else if physical.transform().position.y > halfSize.y {
				physical.transformMut().position.y = halfSize.y;
				physical.lastTransformMut().position.y = halfSize.y + velocity.y;
			}
		}
	}
	
	fn collide(&self, dt: f32) {
		for i in 0..self.physicals.len() {
			let physical1 = self.physicals[i].clone();
			for j in (i + 1)..self.physicals.len() {
				let physical2 = self.physicals[j].clone();
				let skip = {
					let p1 = physical1.borrow();
					let p2 = physical2.borrow();
					(p2.transform().position.x - p2.transform().scale.x * 0.5) >
						(p1.transform().position.x + p1.transform().scale.x * 0.5)
				};
				if skip {
					break;
				}
				self.collideWithPhysical(physical1.clone(), physical2.clone());
			}
			
			self.collideWithBoundary(dt, physical1);
		}
	}
	
	fn updatePhysicals(&self, dt: f32) {
		for physical in self.physicals.iter() {
			let mut physical = physical.borrow_mut();
			physical.accelerate(self.gravity);
			physical.update(dt);
		}
	}
	
	fn subStep(&mut self, dt: f32) {
		self.sortPhysicals();
		self.collide(dt);
		self.updatePhysicals(dt);
	}
	
	pub fn update(&mut self, dt: f32) {
		if !self.isPaused() || self.isForceStep() {
			let subStepDt = dt / self.subSteps as f32;
			for _ in 0..self.subSteps {
				self.subStep(subStepDt);
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
				ui.text(format!("Sub step dt: {}", dt / self.subSteps as f32));
				
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
		&self.mesh
	}
	
	fn shaderRef(&self) -> &ShaderRef {
		&self.shader
	}
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::from_scale(self.worldSize)
	}
}