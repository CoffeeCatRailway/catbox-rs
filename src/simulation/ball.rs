use std::f32::consts::TAU;
use std::rc::Rc;
use glam::{vec3, Mat4, Vec2, Vec3};
use crate::graphics::instance_mesh::InstanceMesh;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::render_manager::Renderable;
use crate::graphics::shader::Shader;
use crate::simulation::transform::Transform;
use crate::types::{GlRef, ShaderRef};

pub struct Ball {
	pub mesh: InstanceMesh,
	pub shader: ShaderRef,
	pub transform: Transform,
	pub positionLast: Vec2,
	pub acceleration: Vec2,
	pub color: Vec3
}

impl Renderable for Ball {
	fn mesh(&self) -> &dyn Mesh {
		&self.mesh
	}
	
	fn meshMut(&mut self) -> &mut dyn Mesh {
		&mut self.mesh
	}
	
	fn shader(&self) -> &Shader {
		&self.shader
	}
	
	fn shaderMut(&mut self) -> &mut Shader {
		Rc::get_mut(&mut self.shader).unwrap()
	}
	
	fn modelMatrix(&self) -> Mat4 {
		self.transform.getModelMatrix()
	}
}

impl Ball {
	pub fn new(gl: GlRef, shader: ShaderRef) -> Self {
		let (vertices, indices) = Self::data();
		let mesh = InstanceMesh::withIndices(gl, vertices, indices);
		Self {
			mesh,
			shader,
			transform: Default::default(),
			positionLast: Vec2::ZERO,
			acceleration: Vec2::ZERO,
			color: Vec3::ONE,
		}
	}
	
	fn data() -> (Vec<Vertex>, Vec<u32>) {
		let segments = 20;
		let mut vertices = Vec::with_capacity(segments + 1);
		let mut indices = Vec::with_capacity(8 * 3);
		
		vertices.push(Vertex {
			color: vec3(0.5, 0.5, 0.0),
			..Default::default()
		});
		for i in 0..segments {
			let angle = i as f32 * TAU / segments as f32;
			vertices.push(Vertex {
				position: vec3(angle.cos(), angle.sin(), 0.0),
				color: vec3(angle.cos() * 0.5 + 0.5, angle.sin() * 0.5 + 0.5, 0.0),
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
