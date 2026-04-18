use bool_flags::Flags8;
use glam::Mat4;
use crate::graphics::line_renderer::LineRenderer;
use crate::types::{GlRef, MeshRef, RenderableRef, ShaderRef};

#[allow(unused)]
pub trait Renderable {
	fn meshRef(&self) -> Option<&MeshRef>;
	
	fn shaderRef(&self) -> Option<&ShaderRef>;
	
	fn render(&self, projViewMat: &Mat4, dt: f32, _lineRenderer: &mut LineRenderer) -> Result<(), String> {
		if let Some(mesh) = self.meshRef() && let Some(shader) = self.shaderRef() {
			let mesh = mesh.borrow();
			let shader = shader.read().unwrap();
			
			shader.bind();
			let pvm = projViewMat * self.modelMatrix();
			shader.setMatrix4f("u_pvm", &pvm);
			
			mesh.draw();
		}
		self.renderPost(projViewMat, dt, _lineRenderer)?;
		Ok(())
	}
	
	fn renderPost(&self, _projViewMat: &Mat4, _dt: f32, _lineRenderer: &mut LineRenderer) -> Result<(), String> {
		Ok(())
	}
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::IDENTITY
	}
	
	fn visible(&self) -> bool {
		true
	}
	
	fn destroy(&mut self) {
		if let Some(mesh) = self.meshRef() {
			mesh.borrow_mut().destroy();
		}
	}
}

const F_DESTROYED: u8 = 0;

pub struct RenderManager {
	flags: Flags8,
	renderables: Vec<RenderableRef>,
	lineRenderer: LineRenderer,
}

impl RenderManager {
	pub fn new(gl: GlRef) -> Result<Self, String> {
		let mut lineRenderer = LineRenderer::new(gl, 1024)?;
		lineRenderer.enable(false);
		lineRenderer.setLineWidth(1.5);
		Ok(Self {
			flags: Flags8::none(),
			renderables: Vec::new(),
			lineRenderer,
		})
	}
	
	pub fn addRenderable(&mut self, renderable: RenderableRef) {
		self.renderables.push(renderable);
	}
	
	pub fn draw(&mut self, projViewMat: &Mat4, dt: f32) -> Result<(), String> {
		for renderable in self.renderables.iter() {
			let renderable = renderable.borrow();
			if renderable.visible() {
				renderable.render(projViewMat, dt, &mut self.lineRenderer)?;
			}
		}
		self.lineRenderer.drawFlush(&projViewMat);
		Ok(())
	}
	
	pub fn destroy(&mut self) {
		self.flags.set(F_DESTROYED);
		for renderable in self.renderables.iter() {
			renderable.borrow_mut().destroy();
		}
		self.lineRenderer.destroy();
	}
	
	pub fn lineRendererMut(&mut self) -> &mut LineRenderer {
		&mut self.lineRenderer
	}
}