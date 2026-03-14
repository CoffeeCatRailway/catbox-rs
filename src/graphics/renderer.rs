use glam::Mat4;
use crate::graphics::mesh::Mesh;
use crate::graphics::shader::Shader;
use crate::simulation::camera::Camera;
use crate::types::{GlRef, RenderableRef};

pub trait Renderable {
	fn mesh(&self) -> &dyn Mesh;
	fn meshMut(&mut self) -> &mut dyn Mesh;
	
	fn shader(&self) -> &Shader;
	fn shaderMut(&mut self) -> &mut Shader;
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::IDENTITY
	}
}

pub struct Renderer {
	gl: GlRef,
	pub renderables: Vec<RenderableRef>,
}

impl Renderer {
	pub fn new(gl: GlRef) -> Self {
		Self {
			gl,
			renderables: Vec::new(),
		}
	}
	
	pub fn addRenderable(&mut self, renderable: RenderableRef) {
		self.renderables.push(renderable);
	}
	
	// update
	
	pub fn draw(&mut self, projViewMat: &Mat4, _camera: &Camera) {
		for renderable in &self.renderables {
			let obj = renderable.borrow();
			let mesh = obj.mesh();
			let shader = obj.shader();
			
			shader.bind();
			let pvm = projViewMat * obj.modelMatrix();
			shader.setMatrix4f("u_pvm", &pvm);
			
			mesh.draw();
		}
	}
}
