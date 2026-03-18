use glam::Mat4;
use crate::simulation::camera::Camera;
use crate::types::{MeshRef, RenderableRef, ShaderRef};

#[allow(unused)]
pub trait Renderable {
	fn meshRef(&self) -> &MeshRef;
	
	fn shaderRef(&self) -> &ShaderRef;
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::IDENTITY
	}
	
	fn visible(&self) -> bool {
		true
	}
}

pub struct RenderManager {
	pub renderables: Vec<RenderableRef>,
}

impl RenderManager {
	pub fn new() -> Self {
		Self {
			renderables: Vec::new(),
		}
	}
	
	pub fn addRenderable(&mut self, renderable: RenderableRef) {
		self.renderables.push(renderable);
	}
	
	pub fn draw(&mut self, projViewMat: &Mat4, _camera: &Camera) {
		for renderable in &self.renderables {
			if !renderable.visible() {
				continue;
			}
			let mesh = renderable.meshRef().read().unwrap();
			let shader = renderable.shaderRef().read().unwrap();
			
			shader.bind();
			let pvm = projViewMat * renderable.modelMatrix();
			shader.setMatrix4f("u_pvm", &pvm);
			
			mesh.draw();
		}
	}
}