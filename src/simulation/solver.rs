use std::time::Instant;
use bool_flags::Flags8;
use dear_imgui_rs::{Ui, WindowFlags};
use glam::{vec3, Mat4, Vec3};
use tracing::info;
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
	
	fn elasticity(&self) -> f32; // todo: try moving properties to separate component
	
	fn update(&mut self, dt: f32);
	
	fn accelerate(&mut self, acceleration: Vec3);
	
	fn setVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn addVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn getVelocity(&self, dt: f32) -> Vec3;
	
	fn getColor(&self) -> Vec3; // todo: move shape and collision to separate components
}

struct Edge {
	physical: PhysicalRef,
	isLeft: bool,
}

// const F_DESTROYED: u8 = 0;
const F_PAUSED: u8 = 1;
const F_FORCE_STEP: u8 = 2;

pub struct Solver {
	mesh: MeshRef,
	shader: ShaderRef,
	
	pub gravity: Vec3,
	pub worldSize: Vec3,
	
	edges: Vec<Edge>,
	physicals: Vec<PhysicalRef>,
	
	subSteps: u32,
	updatesDone: u32,
	
	flags: Flags8,
	
	sortTime: f32,
}

impl Solver {
	pub fn new(worldSize: Vec3, gl: GlRef, shader: ShaderRef) -> Result<Solver, String> {
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
			
			edges: Vec::new(),
			physicals: Vec::new(),
			
			subSteps: 8,
			updatesDone: 0,
			
			flags,
			sortTime: 0.0,
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
		self.edges.push(Edge {
			physical: physical.clone(),
			isLeft: true,
		});
		self.edges.push(Edge {
			physical: physical.clone(),
			isLeft: false,
		});
		self.physicals.push(physical);
	}
	
	fn sortEdges(&mut self) {
		let now = Instant::now();
		
		// ~0.269025ms, average from 8 substeps with 800 physicals, done until stable state
		// default rust sort
		// self.edges.sort_by(|a, b| {
		// 	let ax = {
		// 		let ap = a.physical.borrow();
		// 		if a.isLeft {
		// 			ap.transform().position.x - ap.transform().scale.x * 0.5
		// 		} else {
		// 			ap.transform().position.x + ap.transform().scale.x * 0.5
		// 		}
		// 	};
		// 	let bx = {
		// 		let bp = b.physical.borrow();
		// 		if b.isLeft {
		// 			bp.transform().position.x - bp.transform().scale.x * 0.5
		// 		} else {
		// 			bp.transform().position.x + bp.transform().scale.x * 0.5
		// 		}
		// 	};
		// 	ax.total_cmp(&bx)
		// });
		
		// ~0.14825ms, average from 8 substeps with 800 physicals, done until stable state
		// insertion sort
		for i in 1..self.edges.len() {
			for j in (0..i).rev() { // j=i-1; j >= 0; j--
				let ax = {
					let a = &self.edges[j];
					let ap = a.physical.borrow();
					if a.isLeft {
						ap.transform().position.x - ap.transform().scale.x * 0.5
					} else {
						ap.transform().position.x + ap.transform().scale.x * 0.5
					}
				};
		
				let bx = {
					let b = &self.edges[j + 1];
					let bp = b.physical.borrow();
					if b.isLeft {
						bp.transform().position.x - bp.transform().scale.x * 0.5
					} else {
						bp.transform().position.x + bp.transform().scale.x * 0.5
					}
				};
		
				if ax < bx {
					break;
				}
				self.edges.swap(j, j + 1);
			}
		}
		
		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.sortTime += end;
	}
	
	// fn sortPhysicals(&mut self) {
	// 	self.physicals.sort_by(|a, b| {
	// 		let a = a.borrow();
	// 		let b = b.borrow();
	//
	// 		let p1 = a.transform().position.x - a.transform().scale.x;
	// 		let p2 = b.transform().position.x - b.transform().scale.x;
	//
	// 		p1.total_cmp(&p2)
	// 	});
	// }
	
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
		let mut touching: Vec<PhysicalRef> = Vec::new();
		for edge in self.edges.iter() {
			if edge.isLeft {
				for other in touching.iter() {
					self.collideWithPhysical(other.clone(), edge.physical.clone());
				}
				self.collideWithBoundary(dt, edge.physical.clone());
				touching.push(edge.physical.clone());
			} else {
				if let Some(index) = touching.iter().position(|x| x.borrow().transform().position == edge.physical.borrow().transform().position) {
					touching.remove(index);
				}
			}
		}
		
		// for physical in self.physicals.iter() {
		// 	self.collideWithBoundary(dt, physical.clone());
		// }
	}
	
	fn updatePhysicals(&self, dt: f32) {
		for physical in self.physicals.iter() {
			let mut physical = physical.borrow_mut();
			physical.accelerate(self.gravity);
			physical.update(dt);
		}
	}
	
	fn subStep(&mut self, dt: f32) {
		self.sortEdges();
		// self.sortPhysicals();
		self.collide(dt);
		self.updatePhysicals(dt);
	}
	
	pub fn update(&mut self, dt: f32) {
		if !self.isPaused() || self.isForceStep() {
			let subStepDt = dt / self.subSteps as f32;
			for _ in 0..self.subSteps {
				self.subStep(subStepDt);
			}
			
			info!("Avg sort time: {}ms", self.sortTime * (1.0 / self.subSteps as f32));
			self.sortTime = 0.0;
			
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

impl Renderable for Solver {
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