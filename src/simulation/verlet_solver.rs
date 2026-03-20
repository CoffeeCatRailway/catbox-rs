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
}

const F_DESTROYED: u8 = 0;
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
	
	fn collideWithBoundary(&self, _dt: f32, physical: &PhysicalRef) {
		let mut physical = physical.write().unwrap();
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
	
	fn collide(&self, dt: f32) {
		for physical in self.physicals.iter() {
			self.collideWithBoundary(dt, physical);
		}
	}
	
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
		&self.mesh
	}
	
	fn shaderRef(&self) -> &ShaderRef {
		&self.shader
	}
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::from_scale(self.worldSize)
	}
}