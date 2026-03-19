use glam::Mat4;
use crate::types::{MeshRef, RenderableRef, ShaderRef};

#[allow(unused)]
pub trait Renderable {
	fn meshRef(&self) -> &MeshRef;
	
	fn shaderRef(&self) -> &ShaderRef;
	
	fn render(&self, projViewMat: &Mat4, dt: f32) -> Result<(), String> {
		let mesh = self.meshRef().read().unwrap();
		let shader = self.shaderRef().read().unwrap();
		
		shader.bind();
		let pvm = projViewMat * self.modelMatrix();
		shader.setMatrix4f("u_pvm", &pvm);
		
		mesh.draw();
		Ok(())
	}
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::IDENTITY
	}
	
	fn visible(&self) -> bool {
		true
	}
}

pub struct RenderManager {
	renderables: Vec<RenderableRef>
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
	
	pub fn draw(&mut self, projViewMat: &Mat4, dt: f32) -> Result<(), String> {
		for renderable in self.renderables.iter() {
			if renderable.visible() {
				renderable.render(projViewMat, dt)?;
			}
		}
		Ok(())
	}
}