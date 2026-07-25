use std::f32::consts::PI;
use std::rc::Rc;
use std::sync::Arc;
use bool_flags::Flags8;
use dear_imgui_glow::GlowRenderer;
use dear_imgui_rs::{TextureFormat, TextureId, Ui, WindowFlags};
use glam::{Mat4, Vec3, Vec4};
use glam::camera::rh::proj::opengl::{orthographic, perspective};
use glam::camera::rh::view::look_at_mat4;
use glow::{HasContext, NativeBuffer};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use tracing::{info, warn};
use crate::graphics::{shaders, DepthComponent, LineRenderer, Texture};
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::{gl_check_error, LogError};
use crate::graphics::light::Light;
use crate::simulation::Transform;
use crate::types::{newLightRef, newTextureRef, GlRef, LightRef, MaterialRef, MeshRef, RenderableRef, SdlWindowRef, ShaderRef, TextureRef};
use crate::window::camera::{Camera, Projection};
use crate::window::InputHelper;

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

const SHADOW_MAP_RES: u32 = 1024 * 1;

pub struct RenderManager {
	flags: Flags8,
	gl: GlRef,
	window: SdlWindowRef,
	
	camera: Camera,
	projectionMatrix: Mat4,
	lineRenderer: LineRenderer,
	renderables: Vec<RenderableRef>,
	
	lights: Vec<LightRef>,
	sunLight: LightRef,
	
	shadowMap: TextureRef,
	// shadowMapId: TextureId,
	shadowMapShader: ShaderRef,
	shadowCascadeLevels: Vec<f32>,
	lightSpaceMatUBO: NativeBuffer,
}

impl RenderManager {
	pub fn new(gl: GlRef, window: SdlWindowRef, camera: Camera, imguiRenderer: &mut GlowRenderer, sunLight: Light) -> Result<Self, String> {
		let mut lineRenderer = LineRenderer::new(gl.clone(), 1024).logErr()?;
		lineRenderer.enable(false);
		lineRenderer.setLineWidth(1.5);
		
		// calculate cascade planes for shadows
		// let maxCascade: u32 = 4;
		// let mut shadowCascadeLevels = Vec::new();
		// for i in 1..=maxCascade {
		// 	let bias = 1.0; // 1 = uniform, >1 = bunched close
		// 	let p = (i as f32 / maxCascade as f32).powf(bias) / 2.0;
		// 	shadowCascadeLevels.push(camera.frustum.far * p);
		// 	// info!("cascade layer {}: {:?}", i, shadowCascadeLevels.last());
		// }
		let farPlane = camera.frustum.far;
		let shadowCascadeLevels = vec![farPlane / 8.0, farPlane / 6.0, farPlane / 4.0, farPlane / 2.0];
		
		let shadowMap = Texture::createDepthMapArray(gl.clone(), SHADOW_MAP_RES, SHADOW_MAP_RES, (shadowCascadeLevels.len() + 1) as u32, DepthComponent::U16).logErr()?;
		// let shadowMapId = imguiRenderer.texture_map_mut().register_texture(shadowMap.handleTex.unwrap(), SHADOW_MAP_RES, SHADOW_MAP_RES, TextureFormat::Alpha8);
		let shadowMapShader = shaders::depthMapArrayShader(gl.clone());
		
		let lightSpaceMatUBO = unsafe {
			let ubo = gl.create_buffer().logErr()?;
			gl.bind_buffer(glow::UNIFORM_BUFFER, Some(ubo));
			gl.buffer_data_size(glow::UNIFORM_BUFFER, (size_of::<Mat4>() * 16) as i32, glow::STATIC_DRAW);
			gl.bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(ubo));
			gl.bind_buffer(glow::UNIFORM_BUFFER, None);
			ubo
		};
		
		let mut manager = Self {
			flags: Flags8::none(),
			gl,
			window,
			
			camera,
			projectionMatrix: Mat4::IDENTITY,
			lineRenderer,
			renderables: Vec::new(),
			
			lights: Vec::new(),
			sunLight: newLightRef(sunLight),
			
			shadowMap: newTextureRef(shadowMap),
			// shadowMapId,
			shadowMapShader,
			shadowCascadeLevels,
			lightSpaceMatUBO,
		};
		manager.updateProjectionMatrix();
		Ok(manager)
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
	
	pub fn updateProjectionMatrix(&mut self) {
		let (winWidth, winHeight) = self.window.borrow().size();
		let windowAspect = winWidth as f32 / winHeight as f32;
		let projection = Projection::Perspective(windowAspect);
		// let projection = Projection::Orthographic(windowAspect * -1.0, windowAspect * 1.0, -1.0, 1.0);
		self.projectionMatrix = self.camera.getProjectionMatrix(projection);
	}
	
	fn getFrustumCornersWorldSpace(&self, projViewMat: Mat4) -> Vec<Vec4> {
		let invProjViewMat = projViewMat.inverse();
		let mut corners = Vec::new();
		for x in 0..2 {
			for y in 0..2 {
				for z in 0..2 {
					let pt = invProjViewMat * Vec4::new(2.0 * x as f32 - 1.0, 2.0 * y as f32 - 1.0, 2.0 * z as f32 - 1.0, 1.0);
					corners.push(pt / pt.w);
				}
			}
		}
		corners
	}
	
	fn calcLightSpaceMat(&self, near: f32, far: f32) -> Mat4 {
		let (winWidth, winHeight) = self.window.borrow().size();
		let proj = perspective(self.camera.frustum.fov.to_radians(), winWidth as f32 / winHeight as f32, near, far);
		let corners = self.getFrustumCornersWorldSpace(proj * self.camera.getViewMatrix());
		let mut center = Vec3::ZERO;
		for corner in corners.iter() {
			center += corner.truncate();
		}
		center /= corners.len() as f32;
		
		let lightView = look_at_mat4(center - self.sunLight.borrow().properties().position, center, Vec3::Y);
		
		let mut minX = f32::MAX;
		let mut maxX = f32::MIN;
		let mut minY = f32::MAX;
		let mut maxY = f32::MIN;
		let mut minZ = f32::MAX;
		let mut maxZ = f32::MIN;
		for corner in corners.iter() {
			let p = lightView * corner;
			minX = minX.min(p.x);
			maxX = maxX.max(p.x);
			minY = minY.min(p.y);
			maxY = maxY.max(p.y);
			minZ = minZ.min(p.z);
			maxZ = maxZ.max(p.z);
		}
		
		let zMult = 10.0;
		if minZ < 0.0 {
			minZ *= zMult;
		} else {
			minZ /= zMult;
		}
		if maxZ < 0.0 {
			maxZ /= zMult;
		} else {
			maxZ *= zMult;
		}
		
		let worldTexelX = (maxX - minX) / SHADOW_MAP_RES as f32;
		let worldTexelY = (maxY - minY) / SHADOW_MAP_RES as f32;
		minX = (minX / worldTexelX).floor() * worldTexelX;
		maxX = (maxX / worldTexelX).floor() * worldTexelX;
		minY = (minY / worldTexelY).floor() * worldTexelY;
		maxY = (maxY / worldTexelY).floor() * worldTexelY;
		
		let lightProj = orthographic(minX, maxX, minY, maxY, minZ, maxZ);
		lightProj * lightView
		
		// let frustumBounds = self.camera.calcFrustumBoundsForView(winWidth, winHeight, lightView, near, far);
		// let worldTexelSizeX = (frustumBounds.1.x - frustumBounds.0.x) / SHADOW_MAP_RES as f32;
		// let worldTexelSizeY = (frustumBounds.1.y - frustumBounds.0.y) / SHADOW_MAP_RES as f32;
		// let minX = (frustumBounds.0.x / worldTexelSizeX).floor() * worldTexelSizeX;
		// let maxX = (frustumBounds.1.x / worldTexelSizeX).floor() * worldTexelSizeX;
		// let minY = (frustumBounds.0.y / worldTexelSizeY).floor() * worldTexelSizeY;
		// let maxY = (frustumBounds.1.y / worldTexelSizeY).floor() * worldTexelSizeY;
		//
		// let lightProj = orthographic(minX, maxX, minY, maxY, -frustumBounds.1.z, -frustumBounds.0.z);
		// lightProj * lightView
	}
	
	fn calcCascadeMats(&self) -> Vec<Mat4> {
		let mut mats = Vec::new();
		for i in 0..self.shadowCascadeLevels.len() + 1 {
			if i == 0 {
				mats.push(self.calcLightSpaceMat(self.camera.frustum.near, self.shadowCascadeLevels[i]));
			} else if i < self.shadowCascadeLevels.len() {
				mats.push(self.calcLightSpaceMat(self.shadowCascadeLevels[i - 1], self.shadowCascadeLevels[i]));
			} else {
				mats.push(self.calcLightSpaceMat(self.shadowCascadeLevels[i - 1], self.camera.frustum.far));
			}
		}
		mats
	}
	
	pub fn draw(&mut self, dt: f32) -> Result<(), String> {
		if self.flags.get(F_DESTROYED) {
			return Err("Tried drawing render manager after it was destroyed!".into());
		}
		
		unsafe {
			// light space mats ubo setup
			let lightSpaceMats: Vec<Mat4> = self.calcCascadeMats();
			self.gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.lightSpaceMatUBO));
			for i in 0..lightSpaceMats.len() {
				self.gl.buffer_sub_data_u8_slice(glow::UNIFORM_BUFFER, (i * size_of::<Mat4>()) as i32, bytemuck::cast_slice(&lightSpaceMats[i].to_cols_array()));
			}
			self.gl.bind_buffer(glow::UNIFORM_BUFFER, None);
			
			// shadow pass
			self.shadowMap.bindDepthMapFBO();
			self.gl.viewport(0, 0, SHADOW_MAP_RES as i32, SHADOW_MAP_RES as i32);
			self.gl.clear(glow::DEPTH_BUFFER_BIT);
			gl_check_error!(self.gl);
			
			// self.gl.cull_face(glow::FRONT);
			let shader = self.shadowMapShader.read().unwrap();
			shader.bind();
			// for i in 0..lightSpaceMats.len() {
			// 	shader.setMatrix4f(format!("u_lightSpaceMats[{}]", i).as_str(), &lightSpaceMats[i])
			// }
			for renderable in self.renderables.iter() {
				let renderable = renderable.borrow();
				if renderable.visible() && let Some(mesh) = renderable.mesh() {
					shader.setMatrix4f("u_modelMatrix", &renderable.modelMatrix());
					mesh.draw();
				}
			}
			// self.gl.cull_face(glow::BACK);
			
			// render pass
			self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
			gl_check_error!(self.gl);
			
			let (winWidth, winHeight) = self.window.borrow().size();
			self.gl.viewport(0, 0, winWidth as i32, winHeight as i32);
			self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
			gl_check_error!(self.gl);
			
			for renderable in self.renderables.iter() {
				let renderable = renderable.borrow();
				if renderable.visible() && let Some(mesh) = renderable.mesh() && let Some(material) = renderable.material() {
					material.apply(&self.gl);
					let shader = material.shader.read().unwrap();
					
					let camViewMat = self.camera.getViewMatrix();
					shader.setMatrix4f("u_projViewMatrix", &(self.projectionMatrix * camViewMat));
					shader.setMatrix4f("u_viewMatrix", &camViewMat);
					shader.setMatrix4f("u_modelMatrix", &renderable.modelMatrix());
					
					shader.setUniform3fv("u_viewPos", &self.camera.transform.position);
					shader.setUniform1f("u_farPlane", self.camera.frustum.far);
					
					let borrow = self.sunLight.borrow();
					let sunProperties = borrow.properties();
					shader.setUniform4fv("u_sunLight.position", &(sunProperties.position.extend(borrow.toU8() as f32))); // w is type
					
					shader.setUniform3fv("u_sunLight.color", &sunProperties.color);
					shader.setUniform3f("u_sunLight.strength", sunProperties.ambient, sunProperties.diffuse, sunProperties.specular);
					
					shader.setUniform2f("u_sunLight.attenuation", sunProperties.intensity, sunProperties.radius);
					
					// todo: closest to camera priority
					for i in 0..self.lights.len().min(MAX_LIGHTS) {
						let borrow = self.lights[i].borrow();
						let sunProperties = borrow.properties();
						shader.setUniform4fv(format!("u_lights[{}].position", i).as_str(), &(sunProperties.position.extend(borrow.toU8() as f32))); // w is type
						
						shader.setUniform3fv(format!("u_lights[{}].color", i).as_str(), &sunProperties.color);
						shader.setUniform3f(format!("u_lights[{}].strength", i).as_str(), sunProperties.ambient, sunProperties.diffuse, sunProperties.specular);
						
						shader.setUniform2f(format!("u_lights[{}].attenuation", i).as_str(), sunProperties.intensity, sunProperties.radius);
					}
					
					shader.setUniform1i("u_shadowMap", 1);
					self.shadowMap.bind(1);
					
					// for i in 0..lightSpaceMats.len() {
					// 	shader.setMatrix4f(format!("u_lightSpaceMats[{}]", i).as_str(), &lightSpaceMats[i])
					// }
					for i in 0..self.shadowCascadeLevels.len() {
						shader.setUniform1f(format!("u_cascadePlaneDists[{}]", i).as_str(), self.shadowCascadeLevels[i]);
					}
					shader.setUniform1i("u_cascadeCount", self.shadowCascadeLevels.len() as i32);
					
					mesh.draw();
				}
			}
		}
		
		self.lineRenderer.drawFlush(&(self.projectionMatrix * self.camera.getViewMatrix()));
		Ok(())
	}
	
	pub fn inputMouse(&mut self, event: &Event, guiMouseCaptured: bool, mouseCaptured: bool) {
		match *event {
			Event::MouseWheel { y, .. } => {
				if !guiMouseCaptured {
					self.camera.frustum.zoom(-y * 1.0);
					self.updateProjectionMatrix();
				}
			},
			Event::MouseMotion { xrel, yrel, .. } => {
				if mouseCaptured {
					self.camera.turn(xrel, -yrel);
				}
			}
			_ => {},
		}
	}
	
	pub fn inputKeyboard(&mut self, inputHelper: &InputHelper, dt: f32) {
		if inputHelper.isKeyPressed(Keycode::W) {
			self.camera.transform.translateLocalForward(10.0 * dt);
		}
		if inputHelper.isKeyPressed(Keycode::S) {
			self.camera.transform.translateLocalForward(-10.0 * dt);
		}
		if inputHelper.isKeyPressed(Keycode::A) {
			self.camera.transform.translateLocalRight(-10.0 * dt);
		}
		if inputHelper.isKeyPressed(Keycode::D) {
			self.camera.transform.translateLocalRight(10.0 * dt);
		}
		if inputHelper.isKeyPressed(Keycode::Space) {
			self.camera.transform.translateGlobal(Vec3::Y * 10.0 * dt);
		}
		if inputHelper.isKeyPressed(Keycode::LCtrl) {
			self.camera.transform.translateGlobal(Vec3::Y * -10.0 * dt);
		}
	}
	
	pub fn gui(&mut self, ui: &mut Ui) {
		ui.window("Render Manager")
		  .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
		  .build(|| {
			  ui.text(format!("Camera: ({:.2}, {:.2}, {:.2})", self.camera.transform.position.x, self.camera.transform.position.y, self.camera.transform.position.z));
			  let uiWidth = ui.window_width();
			  let itemWidth = ui.push_item_width(uiWidth * 0.6);
			  if ui.slider_f32("FOV/Zoom", &mut self.camera.frustum.fov, self.camera.frustum.fovMin, self.camera.frustum.fovMax) {
				  self.updateProjectionMatrix();
			  }
			  itemWidth.end();
			  ui.separator();
			  
			  ui.text("Line Renderer:");
			  ui.text(format!("Buffer capacity: {}", self.lineRenderer.getBufferCapacity()));
			  ui.text(format!("Last floats pushed: {}", self.lineRenderer.getLastFloatsPushed()));
			  ui.separator();
			  
			  ui.text("Sun");
			  let mut sunLight = self.sunLight.borrow_mut();
			  
			  let mut sunColor = sunLight.properties().color.to_array();
			  let uiWidth = ui.window_width();
			  let itemWidth = ui.push_item_width(uiWidth * 0.8);
			  if ui.color_edit3("Color", &mut sunColor) {
				  sunLight.propertiesMut().color = Vec3::from_array(sunColor);
			  }
			  itemWidth.end();
			  
			  let itemWidth = ui.push_item_width(uiWidth * 0.3);
			  ui.slider_f32("##sunAmbient", &mut sunLight.propertiesMut().ambient, 0.0, 1.0);
			  if ui.is_item_hovered() {
				  ui.tooltip_text("Ambient");
			  }
			  ui.same_line();
			  ui.slider_f32("##sunDiffuse", &mut sunLight.propertiesMut().diffuse, 0.0, 1.0);
			  if ui.is_item_hovered() {
				  ui.tooltip_text("Diffuse");
			  }
			  ui.same_line();
			  ui.slider_f32("##sunSpecular", &mut sunLight.propertiesMut().specular, 0.0, 1.0);
			  if ui.is_item_hovered() {
				  ui.tooltip_text("Specular");
			  }
			  itemWidth.end();
			  
			  let mut sunAngle = sunLight.properties().position.z.atan2(sunLight.properties().position.x);
			  let itemWidth = ui.push_item_width(uiWidth * 0.6);
			  if ui.slider_f32("Angle", &mut sunAngle, -PI, PI) {
				  sunLight.propertiesMut().position = Vec3::new(sunAngle.cos(), -1.0, sunAngle.sin());
			  }
			  itemWidth.end();
			  // if ui.collapsing_header("Shadow Map", TreeNodeFlags::COLLAPSING_HEADER) {
				//   ui.image_config(self.shadowMapId, [200.0, 200.0]).uv0([0.0, 1.0]).uv1([1.0, 0.0]).build();
			  // }
		  });
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
	
	pub fn cameraMut(&mut self) -> &mut Camera {
		&mut self.camera
	}
	
	pub fn lineRendererMut(&mut self) -> &mut LineRenderer {
		&mut self.lineRenderer
	}
	
	pub fn shadowMap(&self) -> &TextureRef {
		&self.shadowMap
	}
}
