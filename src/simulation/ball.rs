use std::f32::consts::TAU;
use bool_flags::Flags8;
use glam::{vec3, Vec3};
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::render_manager::Renderable;
use crate::simulation::transform::Transform;
use crate::types::{newMeshRef, GlRef, MeshRef, ShaderRef};

const F_FIXED: u8 = 0;
const F_VISIBLE: u8 = 1;

/// Instance renderable
pub struct BallRenderable {
	mesh: MeshRef,
	shader: ShaderRef,
}

impl BallRenderable {
	pub fn new(gl: GlRef, shader: ShaderRef) -> Self {
		let (vertices, indices) = Self::data();
		let mesh = Mesh::instance(gl, vertices, Some(indices));
		Self {
			mesh: newMeshRef(mesh),
			shader,
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
				position: vec3(angle.cos(), angle.sin(), 0.0),
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
	fn meshRef(&self) -> &MeshRef {
		&self.mesh
	}
	
	fn shaderRef(&self) -> &ShaderRef {
		&self.shader
	}
}

impl Drop for BallRenderable {
	fn drop(&mut self) {
		self.meshRef().write().unwrap().destroy();
	}
}

/// Physics object
pub struct Ball {
	pub transform: Transform,
	pub lastTransform: Transform,
	pub acceleration: Vec3,
	pub color: Vec3,
	pub radius: f32,
	pub elasticity: f32,
	flags: Flags8,
}

impl Ball {
	pub fn new() -> Self {
		let mut flags = Flags8::none();
		flags.set(F_FIXED);
		flags.set(F_VISIBLE);
		Self {
			transform: Default::default(),
			lastTransform: Default::default(),
			acceleration: Vec3::ZERO,
			color: Vec3::ONE,
			radius: 1.0,
			elasticity: 1.0,
			flags,
		}
	}
}
}