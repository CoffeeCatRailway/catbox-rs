use std::rc::Rc;
use std::sync::Arc;
use bool_flags::Flags8;
use dear_imgui_glow::GlowRenderer;
use dear_imgui_rs::{TextureFormat, TextureId};
use glam::{Mat4, Vec3, Vec4};
use glow::HasContext;
use tracing::warn;
use crate::graphics::{shaders, LineRenderer, Texture};
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::{gl_check_error, LogError};
use crate::graphics::light::Light;
use crate::simulation::Transform;
use crate::types::{newLightRef, newTextureRef, GlRef, LightRef, MaterialRef, MeshRef, RenderableRef, ShaderRef, TextureRef};
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
	
	fn renderPost(&self, _gl: &GlRef, _projViewMat: &Mat4, _dt: f32, lineRenderer: &mut LineRenderer) -> Result<(), String> {
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
	
	fn render(&self, gl: &GlRef, projectMat: &Mat4, lightSpaceMat: &Mat4, dt: f32, _lineRenderer: &mut LineRenderer, sunLight: &LightRef, lights: &Vec<LightRef>, camera: &Camera, shadowMapShader: Option<ShaderRef>, shadowMap: Option<TextureRef>) -> Result<(), String> {
		if let Some(mesh) = self.mesh() && let Some(material) = self.material() {
			let shader = if let Some(shader) = shadowMapShader {
				shader.read().unwrap().bind();
				shader.clone()
			} else {
				material.apply(gl);
				material.shader.clone()
			};
			let shader = shader.read().unwrap();
			
			let camViewMat = camera.getViewMatrix();
			shader.setMatrix4f("u_projViewMatrix", &(projectMat * camViewMat));
			shader.setMatrix4f("u_viewMatrix", &camViewMat);
			shader.setMatrix4f("u_modelMatrix", &self.modelMatrix());
			shader.setMatrix4f("u_lightSpaceMatrix", lightSpaceMat);
			
			shader.setUniform3fv("u_viewPos", &camera.transform.position);
			
			let borrow = sunLight.borrow();
			let sunProperties = borrow.properties();
			shader.setUniform4fv("u_sunLight.position", &(sunProperties.position.extend(borrow.toU8() as f32))); // w is type
			
			shader.setUniform3fv("u_sunLight.color", &sunProperties.color);
			shader.setUniform3f("u_sunLight.strength", sunProperties.ambient, sunProperties.diffuse, sunProperties.specular);
			
			shader.setUniform2f("u_sunLight.attenuation", sunProperties.intensity, sunProperties.radius);
			
			// todo: closest to camera priority
			for i in 0..lights.len().min(MAX_LIGHTS) {
				let borrow = lights[i].borrow();
				let sunProperties = borrow.properties();
				shader.setUniform4fv(format!("u_lights[{}].position", i).as_str(), &(sunProperties.position.extend(borrow.toU8() as f32))); // w is type
				
				shader.setUniform3fv(format!("u_lights[{}].color", i).as_str(), &sunProperties.color);
				shader.setUniform3f(format!("u_lights[{}].strength", i).as_str(), sunProperties.ambient, sunProperties.diffuse, sunProperties.specular);
				
				shader.setUniform2f(format!("u_lights[{}].attenuation", i).as_str(), sunProperties.intensity, sunProperties.radius);
			}
			
			if shadowMap.is_some() {
				shader.setUniform1i("u_shadowMap", 1);
				shadowMap.unwrap().bind(1);
			}
			
			mesh.draw();
		}
		Ok(())
	}
	
	fn renderPost(&self, _gl: &GlRef, _projectMat: &Mat4, _dt: f32, _lineRenderer: &mut LineRenderer) -> Result<(), String> {
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

const SHADOW_MAP_RES: i32 = 1024;
const SHADOW_MAP_BIAS_MAT: Mat4 = Mat4 {
	x_axis: Vec4::new(0.5, 0.0, 0.0, 0.0),
	y_axis: Vec4::new(0.0, 0.5, 0.0, 0.0),
	z_axis: Vec4::new(0.0, 0.0, 0.5, 0.0),
	w_axis: Vec4::new(0.5, 0.5, 0.5, 1.0),
};

pub struct RenderManager {
	flags: Flags8,
	gl: GlRef,
	lineRenderer: LineRenderer,
	renderables: Vec<RenderableRef>,
	
	lights: Vec<LightRef>,
	sunLight: LightRef,
	
	shadowMap: TextureRef,
	shadowMapId: TextureId,
	shadowMapShader: ShaderRef,
}

impl RenderManager {
	pub fn new(gl: GlRef, imguiRenderer: &mut GlowRenderer, sunLight: Light) -> Result<Self, String> {
		let mut lineRenderer = LineRenderer::new(gl.clone(), 1024).logErr()?;
		lineRenderer.enable(false);
		lineRenderer.setLineWidth(1.5);
		
		let shadowMap = Texture::createDepthMap(gl.clone(), SHADOW_MAP_RES as u32, SHADOW_MAP_RES as u32).logErr()?;
		let shadowMapId = imguiRenderer.texture_map_mut().register_texture(shadowMap.handleTex.unwrap(), SHADOW_MAP_RES as u32, SHADOW_MAP_RES as u32, TextureFormat::Alpha8);
		let shadowMapShader = shaders::shadowMapShader(gl.clone());
		
		Ok(Self {
			flags: Flags8::none(),
			gl,
			lineRenderer,
			renderables: Vec::new(),
			
			lights: Vec::new(),
			sunLight: newLightRef(sunLight),
			
			shadowMap: newTextureRef(shadowMap),
			shadowMapId,
			shadowMapShader,
		})
	}
	
	pub fn addRenderable(&mut self, renderable: RenderableRef) {
		self.renderables.push(renderable);
	}
	
	pub fn addLight(&mut self, light: LightRef) {
		self.lights.push(light);
	}
	
	pub fn sunLight(&self) -> &LightRef {
		&self.sunLight
	}
	
	fn drawRenderables(&mut self, projectMat: &Mat4, lightSpaceMat: &Mat4, dt: f32, camera: &Camera, shadowMapShader: Option<ShaderRef>, shadowMap: Option<TextureRef>) -> Result<(), String> {
		for renderable in self.renderables.iter() {
			let renderable = renderable.borrow();
			if renderable.visible() {
				renderable.render(&self.gl, projectMat, lightSpaceMat, dt, &mut self.lineRenderer, &self.sunLight, &self.lights, camera, shadowMapShader.clone(), shadowMap.clone()).logErr()?;
				renderable.renderPost(&self.gl, projectMat, dt, &mut self.lineRenderer).logErr()?;
			}
		}
		Ok(())
	}
	
	pub fn draw(&mut self, winWidth: u32, winHeight: u32, projectMat: &Mat4, dt: f32, camera: &Camera) -> Result<(), String> {
		if self.flags.get(F_DESTROYED) {
			return Err("Tried drawing render manager after it was destroyed!".into());
		}
		
		unsafe {
			// shadow pass
			self.gl.viewport(0, 0, SHADOW_MAP_RES, SHADOW_MAP_RES);
			self.shadowMap.bindDepthMapFBO();
			self.gl.clear(glow::DEPTH_BUFFER_BIT);
			gl_check_error!(self.gl);
			
			// let lightProj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, -10.0, 20.0);
			let lightView = Mat4::look_at_rh(-self.sunLight.borrow().properties().position, Vec3::ZERO, Vec3::Y);
			
			// todo: cascade
			let frustumBounds = camera.calcFrustumBoundsForView(winWidth, winHeight, lightView);
			let worldTexelSizeX = (frustumBounds.1.x - frustumBounds.0.x) / SHADOW_MAP_RES as f32;
			let worldTexelSizeY = (frustumBounds.1.y - frustumBounds.0.y) / SHADOW_MAP_RES as f32;
			let minX = (frustumBounds.0.x / worldTexelSizeX).floor() * worldTexelSizeX;
			let maxX = (frustumBounds.1.x / worldTexelSizeX).floor() * worldTexelSizeX;
			let minY = (frustumBounds.0.y / worldTexelSizeY).floor() * worldTexelSizeY;
			let maxY = (frustumBounds.1.y / worldTexelSizeY).floor() * worldTexelSizeY;
			
			let lightProj = Mat4::orthographic_rh(minX, maxX, minY, maxY, -frustumBounds.1.z, -frustumBounds.0.z);
			let lightSpaceMat = lightProj * lightView;
			
			self.gl.cull_face(glow::FRONT);
			self.drawRenderables(projectMat, &lightSpaceMat, dt, camera, Some(self.shadowMapShader.clone()), None).logErr()?;
			
			self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
			gl_check_error!(self.gl);
			
			// render pass
			self.gl.viewport(0, 0, winWidth as i32, winHeight as i32);
			self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
			gl_check_error!(self.gl);
			
			let lightSpaceMat = SHADOW_MAP_BIAS_MAT * lightSpaceMat; // -1,1 to 0,1
			self.gl.cull_face(glow::BACK);
			self.drawRenderables(projectMat, &lightSpaceMat, dt, camera, None, Some(self.shadowMap.clone())).logErr()?;
		}
		
		self.lineRenderer.drawFlush(&(projectMat * camera.getViewMatrix()));
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
		Arc::get_mut(&mut self.shadowMap).unwrap().delete();
	}
	
	pub fn lineRendererMut(&mut self) -> &mut LineRenderer {
		&mut self.lineRenderer
	}
	
	pub fn shadowMap(&self) -> &TextureRef {
		&self.shadowMap
	}
	
	pub fn shadowMapId(&self) -> &TextureId {
		&self.shadowMapId
	}
}
