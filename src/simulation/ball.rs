use std::f32::consts::TAU;
use bool_flags::Flags8;
use glam::{vec3, Mat4, Vec3};
use crate::graphics::LineRenderer;
use crate::graphics::mesh::{InstanceMeshData, Mesh, Vertex};
use crate::graphics::Renderable;
use crate::simulation::region::AABB;
use crate::simulation::{solver, Transform};
use crate::simulation::solver::Physical;
use crate::types::{newMeshRef, GlRef, MeshRef, ShaderRef, SolverRef};

const F_FIXED: u8 = 0;
const F_VISIBLE: u8 = 1;

// todo: use shape/collision component
/// Instance renderable
pub struct BallRenderable {
	mesh: MeshRef,
	shader: ShaderRef,
	verletSolver: SolverRef,
}

impl BallRenderable {
	pub fn new(gl: GlRef, shader: ShaderRef, verletSolver: SolverRef) -> Self {
		let (vertices, indices) = Self::data();
		let mesh = Mesh::instance(gl, vertices, Some(indices));
		Self {
			mesh: newMeshRef(mesh),
			shader,
			verletSolver,
		}
	}
	
	fn data() -> (Vec<Vertex>, Vec<u32>) {
		let segments = 20;
		let mut vertices = Vec::with_capacity(segments + 1);
		let mut indices = Vec::with_capacity(8 * 3);
		
		vertices.push(Default::default());
		for i in 0..segments {
			let angle = i as f32 * TAU / segments as f32;
			vertices.push(Vertex {
				position: vec3(angle.cos(), angle.sin(), 0.0) / 2.0,
				..Default::default()
			});
			
			indices.push(0);
			indices.push(i as u32 + 1);
			let i2 = i as u32 + 2;
			if i2 <= segments as u32 {
				indices.push(i2);
			} else {
				indices.push(i2 - segments as u32);
			}
		}
		
		(vertices, indices)
	}
}

impl Renderable for BallRenderable {
	fn meshRef(&self) -> Option<&MeshRef> {
		Option::from(&self.mesh)
	}
	
	fn shaderRef(&self) -> Option<&ShaderRef> {
		Option::from(&self.shader)
	}
	
	fn render(&self, projViewMat: &Mat4, _dt: f32, _lineRenderer: &mut LineRenderer) -> Result<(), String> {
		if let Some(mesh) = self.meshRef() && let Some(shader) = self.shaderRef() {
			let mut mesh = mesh.borrow_mut();
			let shader = shader.read().unwrap();
			
			shader.bind();
			let pvm = projViewMat * self.modelMatrix();
			shader.setMatrix4f("u_pvm", &pvm);
			
			let data: Vec<InstanceMeshData> = self.verletSolver.borrow()
												  .getPhysicals().iter()
												  .map(|(_, physical)| {
													  let physical = physical.borrow();
													  InstanceMeshData {
														  matrix: physical.transform().getModelMatrix(),
														  color: physical.color().to_homogeneous(),
													  }
												  }).collect();
			mesh.updateInstanceData(&data)?;
			
			mesh.draw();
		}
		Ok(())
	}
}

impl Drop for BallRenderable {
	fn drop(&mut self) {
		self.destroy();
	}
}

/// Physics object
#[derive(Debug)]
pub struct Ball {
	id: usize,
	pub transform: Transform,
	pub lastTransform: Transform,
	pub acceleration: Vec3,
	pub elasticity: f32,
	pub color: Vec3,
	flags: Flags8,
	aabb: AABB,
}

impl Ball {
	pub fn new(pos: Vec3, size: Vec3) -> Self {
		let mut flags = Flags8::none();
		flags.set(F_VISIBLE);
		Self {
			id: solver::newId(),
			transform: Transform {
				position: pos,
				scale: size,
				..Default::default()
			},
			lastTransform: Transform {
				position: pos,
				scale: size,
				..Default::default()
			},
			acceleration: Vec3::ZERO,
			elasticity: 1.0,
			color: Vec3::ONE,
			flags,
			aabb: AABB {
				position: pos - size / 2.0,
				size,
			}
		}
	}
}

impl Physical for Ball {
	fn id(&self) -> usize {
		self.id
	}
	
	fn transform(&self) -> &Transform {
		&self.transform
	}
	
	fn transformMut(&mut self) -> &mut Transform {
		&mut self.transform
	}
	
	fn lastTransform(&self) -> &Transform {
		&self.lastTransform
	}
	
	fn lastTransformMut(&mut self) -> &mut Transform {
		&mut self.lastTransform
	}
	
	fn fixed(&self) -> bool {
		self.flags.get(F_FIXED)
	}
	
	fn update(&mut self, dt: f32) {
		if self.fixed() {
			return;
		}
		let delta = self.transform.position - self.lastTransform.position;
		self.lastTransform = self.transform;
		self.transform.position += delta + self.acceleration * dt * dt;
		self.acceleration = Vec3::ZERO;
		
		self.aabb.position = self.transform.position - self.transform.scale / 2.0;
		self.aabb.size = self.transform.scale;
	}
	
	fn accelerate(&mut self, acceleration: Vec3) {
		if self.fixed() {
			return;
		}
		self.acceleration += acceleration;
	}
	
	fn setVelocity(&mut self, velocity: Vec3, dt: f32) {
		if self.fixed() {
			return;
		}
		self.lastTransform.position = self.transform.position - velocity * dt;
	}
	
	fn addVelocity(&mut self, velocity: Vec3, dt: f32) {
		if self.fixed() {
			return;
		}
		self.lastTransform.position -= velocity * dt;
	}
	
	fn getVelocity(&self, dt: f32) -> Vec3 {
		(self.transform.position - self.lastTransform.position) / dt
	}
	
	fn elasticity(&self) -> f32 {
		self.elasticity
	}
	
	fn color(&self) -> Vec3 {
		self.color
	}
	
	fn bounds(&self) -> AABB {
		self.aabb
	}
}
