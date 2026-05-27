use bool_flags::Flags8;
use glam::{Mat4, Vec3};
use crate::graphics::light::{Light, LightProperties};
use crate::graphics::LineRenderer;
use crate::LogError;
use crate::simulation::Transform;
use crate::types::{GlRef, MeshRef, RenderableRef, ShaderRef};
use crate::window::camera::Camera;

#[allow(unused)]
pub struct SimpleRenderable {
	pub transform: Transform,
	pub mesh: MeshRef,
	pub shader: ShaderRef,
}

impl Renderable for SimpleRenderable {
	fn meshRef(&self) -> Option<&MeshRef> {
		Some(&self.mesh)
	}
	
	fn shaderRef(&self) -> Option<&ShaderRef> {
		Some(&self.shader)
	}
	
	fn modelMatrix(&self) -> Mat4 {
		self.transform.getModelMatrix()
	}
}

#[allow(unused)]
pub trait Renderable {
	fn meshRef(&self) -> Option<&MeshRef>;
	
	fn shaderRef(&self) -> Option<&ShaderRef>;
	
	fn render(&self, projViewMat: &Mat4, dt: f32, _lineRenderer: &mut LineRenderer, sunLight: &Light, camera: &Camera) -> Result<(), String> {
		if let Some(mesh) = self.meshRef() && let Some(shader) = self.shaderRef() {
			let mesh = mesh.borrow();
			let shader = shader.read().unwrap();
			
			shader.bind();
			shader.setMatrix4f("u_projViewMatrix", projViewMat);
			shader.setMatrix4f("u_modelMatrix", &self.modelMatrix());
			
			shader.setUniform3fv("u_viewPos", &camera.transform.position);
			let sunProperties = sunLight.properties();
			shader.setUniform1ui("u_sunLight.type", sunLight.toU32());
			shader.setUniform3fv("u_sunLight.position", &sunProperties.position);
			shader.setUniform3fv("u_sunLight.ambient", &sunProperties.ambient);
			shader.setUniform1f("u_sunLight.ambientStrength", sunProperties.ambientStrength);
			shader.setUniform1f("u_sunLight.diffuseStrength", sunProperties.diffuseStrength);
			shader.setUniform1f("u_sunLight.specularStrength", sunProperties.specularStrength);
			
			mesh.draw();
		}
		Ok(())
	}
	
	fn renderPost(&self, _projViewMat: &Mat4, _dt: f32, _lineRenderer: &mut LineRenderer, _sunLight: &Light, _camera: &Camera) -> Result<(), String> {
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
	gl: GlRef,
	renderables: Vec<RenderableRef>,
	lineRenderer: LineRenderer,
	
	sunLight: Light,
}

impl RenderManager {
	pub fn new(gl: GlRef) -> Result<Self, String> {
		let mut lineRenderer = LineRenderer::new(gl.clone(), 1024).logErr()?;
		lineRenderer.enable(false);
		lineRenderer.setLineWidth(1.5);
		Ok(Self {
			flags: Flags8::none(),
			gl,
			renderables: Vec::new(),
			lineRenderer,
			
			sunLight: Light::Directional(LightProperties {
				position: Vec3::NEG_ONE.normalize(),
				ambientStrength: 0.1,
				specularStrength: 0.5,
				..Default::default()
			}),
		})
	}
	
	pub fn addRenderable(&mut self, renderable: RenderableRef) {
		self.renderables.push(renderable);
	}
	
	pub fn draw(&mut self, projViewMat: &Mat4, dt: f32, camera: &Camera) -> Result<(), String> {
		if self.flags.get(F_DESTROYED) {
			return Err("Tried drawing render manager after it was destroyed!".into());
		}
		
		for renderable in self.renderables.iter() {
			let renderable = renderable.borrow();
			if renderable.visible() {
				renderable.render(&self.gl, projViewMat, dt, &mut self.lineRenderer, &self.sunLight, camera).logErr()?;
				renderable.renderPost(&self.gl, projViewMat, dt, &mut self.lineRenderer, &self.sunLight, camera).logErr()?;
			}
		}
		self.lineRenderer.drawFlush(&projViewMat);
		Ok(())
	}
	
	pub fn destroy(&mut self) {
		if self.flags.get(F_DESTROYED) {
			return;
		}
		
		warn!("Destroying render manager and renderables");
		self.flags.set(F_DESTROYED);
		for renderable in self.renderables.iter() {
			renderable.borrow_mut().destroy();
		}
		self.lineRenderer.destroy();
	}
	
	pub fn lineRendererMut(&mut self) -> &mut LineRenderer {
		&mut self.lineRenderer
	}
	
	pub fn sunLight(&self) -> &Light {
		&self.sunLight
	}
	
	pub fn sunLightMut(&mut self) -> &mut Light {
		&mut self.sunLight
	}
}