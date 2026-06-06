use std::rc::Rc;
use bool_flags::Flags8;
use glam::Mat4;
use tracing::warn;
use crate::graphics::LineRenderer;
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::LogError;
use crate::simulation::Transform;
use crate::types::{GlRef, LightRef, MaterialRef, MeshRef, RenderableRef};
use crate::window::camera::Camera;

pub const MAX_LIGHTS: usize = 8;

#[allow(unused)]
pub struct SimpleRenderable {
	pub transform: Transform,
	pub mesh: MeshRef,
	pub material: MaterialRef,
}

impl Renderable for SimpleRenderable {
	fn mesh(&self) -> Option<&Mesh> {
		Some(&self.mesh)
	}
	
	fn meshMut(&mut self) -> Option<&mut Mesh> {
		Rc::get_mut(&mut self.mesh)
	}
	
	fn material(&self) -> Option<&Material> {
		Some(&self.material)
	}
	
	fn materialMut(&mut self) -> Option<&mut Material> {
		Rc::get_mut(&mut self.material)
	}
	
	fn renderPost(&self, _gl: &GlRef, _projViewMat: &Mat4, _dt: f32, lineRenderer: &mut LineRenderer, _lights: &Vec<LightRef>, _camera: &Camera) -> Result<(), String> {
		// let vertices = self.mesh.vertices();
		// for vertex in vertices.iter() {
		// 	let p = self.modelMatrix().mul_vec4(vertex.position.extend(1.0)).truncate();
		// 	let n = self.modelMatrix().mul_vec4(vertex.normal.extend(1.0)).truncate().clamp_length_max(1.0);
		// 	lineRenderer.pushLine3(p, Vec3::Z, p + n, Vec3::Z);
		// }
		Ok(())
	}
	
	fn modelMatrix(&self) -> Mat4 {
		self.transform.getModelMatrix()
	}
}

#[allow(unused)]
pub trait Renderable {
	fn mesh(&self) -> Option<&Mesh>;
	
	fn meshMut(&mut self) -> Option<&mut Mesh>;
	
	fn material(&self) -> Option<&Material>;
	
	fn materialMut(&mut self) -> Option<&mut Material>;
	
	fn render(&self, gl: &GlRef, projViewMat: &Mat4, dt: f32, _lineRenderer: &mut LineRenderer, lights: &Vec<LightRef>, camera: &Camera) -> Result<(), String> {
		if let Some(mesh) = self.mesh() && let Some(material) = self.material() {
			let shader = material.shader.read().unwrap();
			
			material.apply(gl);
			shader.setMatrix4f("u_projViewMatrix", projViewMat);
			shader.setMatrix4f("u_modelMatrix", &self.modelMatrix());
			
			shader.setUniform3fv("u_viewPos", &camera.transform.position);
			
			// todo: closest to camera priority
			for i in 0..lights.len() {
				let borrow = lights[i].borrow();
				let sunProperties = borrow.properties();
				shader.setUniform4fv(format!("u_lights[{}].position", i).as_str(), &(sunProperties.position.extend(borrow.toU8() as f32))); // w is type
				
				shader.setUniform3fv(format!("u_lights[{}].color", i).as_str(), &sunProperties.color);
				shader.setUniform3f(format!("u_lights[{}].strength", i).as_str(), sunProperties.ambient, sunProperties.diffuse, sunProperties.specular);
				
				shader.setUniform2f(format!("u_lights[{}].attenuation", i).as_str(), sunProperties.intensity, sunProperties.radius);
			}
			
			mesh.draw();
		}
		Ok(())
	}
	
	fn renderPost(&self, _gl: &GlRef, _projViewMat: &Mat4, _dt: f32, _lineRenderer: &mut LineRenderer, _lights: &Vec<LightRef>, _camera: &Camera) -> Result<(), String> {
		Ok(())
	}
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::IDENTITY
	}
	
	fn visible(&self) -> bool {
		true
	}
	
	fn destroy(&mut self) {
		if let Some(mesh) = self.meshMut() {
			mesh.destroy();
		}
	}
}

const F_DESTROYED: u8 = 0;

pub struct RenderManager {
	flags: Flags8,
	gl: GlRef,
	lineRenderer: LineRenderer,
	renderables: Vec<RenderableRef>,
	lights: Vec<LightRef>,
}

impl RenderManager {
	pub fn new(gl: GlRef) -> Result<Self, String> {
		let mut lineRenderer = LineRenderer::new(gl.clone(), 1024).logErr()?;
		lineRenderer.enable(false);
		lineRenderer.setLineWidth(1.5);
		Ok(Self {
			flags: Flags8::none(),
			gl,
			lineRenderer,
			renderables: Vec::new(),
			lights: Vec::new(),
		})
	}
	
	pub fn addRenderable(&mut self, renderable: RenderableRef) {
		self.renderables.push(renderable);
	}
	
	pub fn addLight(&mut self, light: LightRef) {
		self.lights.push(light);
	}
	
	pub fn draw(&mut self, projViewMat: &Mat4, dt: f32, camera: &Camera) -> Result<(), String> {
		if self.flags.get(F_DESTROYED) {
			return Err("Tried drawing render manager after it was destroyed!".into());
		}
		
		for renderable in self.renderables.iter() {
			let renderable = renderable.borrow();
			if renderable.visible() {
				renderable.render(&self.gl, projViewMat, dt, &mut self.lineRenderer, &self.lights, camera).logErr()?;
				renderable.renderPost(&self.gl, projViewMat, dt, &mut self.lineRenderer, &self.lights, camera).logErr()?;
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
}
