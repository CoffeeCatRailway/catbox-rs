use std::error::Error;
use bool_flags::Flags8;
use dear_imgui_glow::{GlowRenderer, SimpleTextureMap};
#[cfg(feature = "multi-viewport")]
use dear_imgui_glow::multi_viewport as glow_mvp;
use dear_imgui_rs::{ConfigFlags, Context as ImguiContext, TreeNodeFlags, WindowFlags};
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3};
use glow::HasContext;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::timer;
use sdl3::video::{GLContext, GLProfile, SwapInterval};
use tracing::{info, warn};
use crate::gl_check_error;
use crate::graphics::line_renderer::LineRenderer;
use crate::graphics::mesh::InstanceMeshData;
use crate::graphics::render_manager::{RenderManager, Renderable};
use crate::graphics::shaders;
use crate::simulation::ball::BallRenderable;
use crate::simulation::camera::{screenToWorldSpace, Camera, Frustum, Projection};
use crate::simulation::transform::Transform;
use crate::types::{newGlRef, newLineRendererRef, newRenderableRef, newSdlWindowRef, GlRef, LineRendererRef, SdlWindowRef};
use crate::window::input_helper::InputHelper;

const F_RUNNING: u8 = 0;

const WIN_TITLE: &str = "Physics CatBox";
const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: u64 = 1000 / FPS;
const OPTIMAL_DT: f32 = OPTIMAL_WAIT_TIME as f32 / 1000.0;

struct Imgui {
	context: ImguiContext,
	renderer: GlowRenderer,
}

pub struct CatBox {
	width: u32,
	height: u32,
	flags: Flags8,
	inputHelper: InputHelper,
	
	gl: GlRef,
	#[cfg_attr(not(feature = "multi-viewport"), allow(unused))]
	glContext: GLContext,
	window: SdlWindowRef,
	
	imgui: Imgui,
	
	lineRenderer: LineRendererRef,
	renderManager: RenderManager,
	clearColor: [f32; 4],
	lastMousePos: Vec2,
	
	camera: Camera,
	projectionMatrix: Mat4,
	viewMatrix: Mat4,
}

impl CatBox {
	pub fn new() -> Result<CatBox, Box<dyn Error>> {
		info!("Creating CatBox");
		let flags = Flags8::none();
	
		info!("SDL3 context");
		let sdl = sdl3::init()?;
		let video = sdl.video()?;
		let glAttributes = video.gl_attr();
		
		let inputHelper = InputHelper::new();
		
		glAttributes.set_context_profile(GLProfile::Core);
		glAttributes.set_context_version(4, 5);
		glAttributes.set_depth_size(0);
		
		info!("Window and GL context");
		let window = video.window(WIN_TITLE, WIN_WIDTH, WIN_HEIGHT)
						  .opengl()
						  .resizable()
						  .position_centered()
						  .build()?;
		let window = newSdlWindowRef(window);
		
		let glContext = window.read().unwrap().gl_create_context()?;
		
		window.read().unwrap().gl_make_current(&glContext)?;
		video.gl_set_swap_interval(SwapInterval::Immediate)?;
		
		let gl = unsafe {
			use std::ffi::c_void;
			let gl = glow::Context::from_loader_function(|name| {
				video.gl_get_proc_address(name).map(|f| f as *const c_void).unwrap_or(std::ptr::null())
			});
			newGlRef(gl)
		};
		
		info!("Imgui context");
		let mut imgui = ImguiContext::create();
		imgui.set_ini_filename(Some("imgui.ini"))?;
		{
			let io = imgui.io_mut();
			let mut flags= io.config_flags();
			flags.insert(ConfigFlags::DOCKING_ENABLE);
			#[cfg(feature = "multi-viewport")]
			{
				info!("Feature: Imgui multi-viewport");
				flags.insert(ConfigFlags::VIEWPORTS_ENABLE);
			}
			io.set_config_flags(flags);
		}
		
		// Initial SDL3 platform backend
		dear_imgui_sdl3::init_platform_for_opengl(&mut imgui, &window.read().unwrap(), &glContext)?;
		
		// Basic style scaling
		let windowScale = window.read().unwrap().display_scale();
		{
			let style = imgui.style_mut();
			style.set_font_scale_dpi(windowScale);
		}
		
		info!("Imgui glow renderer");
		// #[cfg_attr(not(feature = "multi-viewport"), allow(unused))] // What was this for??
		let mut imguiRenderer = {
			let textureMap = Box::new(SimpleTextureMap::default());
			GlowRenderer::with_external_context(&gl, &mut imgui, textureMap)?
		};
		imguiRenderer.set_framebuffer_srgb_enabled(false);
		#[cfg(feature = "multi-viewport")]
		glow_mvp::enable(&mut imguiRenderer, &mut imgui);
		
		info!("Initializing locals");
		let mut lineRenderer = LineRenderer::new(gl.clone(), 1024)?;
		lineRenderer.enable();
		
		let mut renderManager = RenderManager::new();
		
		let camera = Camera {
			frustum: Frustum {
				fov: 500.0,
				fovMax: 10000.0,
				..Frustum::default()
			},
			transform: Transform {
				position: vec3(0.0, 0.0, 5.0),
				..Transform::default()
			},
			..Camera::default()
		};
		
		let instanceShader = shaders::instanceShader(gl.clone())?;
		
		let mut instanceData = Vec::new();
		let a: u32 = 441; // 1_000_000
		let sq = (a as f32).sqrt() as u32;
		let s = 10.0;
		for i in 0..a {
			let x = (i % sq) as f32;
			let y = (i / sq) as f32;
			instanceData.push(InstanceMeshData {
				matrix: {
					let mut transform = Transform::default();
					transform.position.x = x * s - sq as f32 * s / 2.0 + s / 2.0;
					transform.position.y = y * s - sq as f32 * s / 2.0 + s / 2.0;
					transform.scale *= s / 2.0;
					transform.getModelMatrix()
				},
				color: vec4(x / sq as f32, y / sq as f32, 1.0 - i as f32 / a as f32, 1.0),
			});
		}
		
		// In simulation, each 'ball' won't have a renderable component, there will be a single renderable that maps the object list down to model matrices and color
		let ballRenderable = BallRenderable::new(gl.clone(), instanceShader.clone());
		ballRenderable.meshRef().write().unwrap().uploadInstanceData(&instanceData)?;
		ballRenderable.meshRef().write().unwrap().upload(instanceShader.clone())?;
		
		let ballRef = newRenderableRef(ballRenderable);
		renderManager.addRenderable(ballRef.clone());
		
		let mut catbox = CatBox {
			width: WIN_WIDTH,
			height: WIN_HEIGHT,
			flags,
			inputHelper,
			
			gl,
			glContext,
			window,
			
			imgui: Imgui {
				context: imgui,
				renderer: imguiRenderer,
			},
			
			lineRenderer: newLineRendererRef(lineRenderer),
			renderManager,
			clearColor: [0.27, 0.59, 0.27, 1.0],
			lastMousePos: Vec2::ZERO,
			
			camera,
			projectionMatrix: Mat4::IDENTITY,
			viewMatrix: Mat4::IDENTITY,
		};
		catbox.updateProjectionMatrix();
		Ok(catbox)
	}
	
	fn updateProjectionMatrix(&mut self) {
		let windowSize = self.window.read().unwrap().size();
		let windowAspect = windowSize.0 as f32 / windowSize.1 as f32;
		
		let projection = Projection::Orthographic(windowAspect * -1.0, windowAspect * 1.0, -1.0, 1.0);
		self.projectionMatrix = self.camera.getProjectionMatrix(projection);
	}
	
	fn requestClose(&mut self) {
		if !self.flags.get(F_RUNNING)
		{
			return;
		}
		warn!("CatBox loop exit requested");
		self.flags.clear(F_RUNNING);
	}
	
	fn handleEvents(&mut self, event: &Event) {
		match *event {
			Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
				self.requestClose();
			},
			Event::Window { win_event, window_id, .. } => match win_event {
				WindowEvent::Resized(width, height) => unsafe {
					let (width, height) = (width.max(1), height.max(1));
					self.gl.viewport(0, 0, width, height);
					gl_check_error!(self.gl);
					self.width = width as u32;
					self.height = height as u32;
					self.updateProjectionMatrix();
				},
				WindowEvent::CloseRequested => {
					if window_id == self.window.read().unwrap().id() {
						self.requestClose();
					}
				},
				_ => {},
			},
			Event::MouseWheel { y, .. } => {
				self.camera.frustum.zoom(-y * 10.0);
				self.updateProjectionMatrix();
			}
			_ => {},
		}
	}
	
	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
		info!("Starting CatBox loop");
		self.flags.set(F_RUNNING);
		
		let mut fps: u32 = 0;
		let mut lastTick: u64 = 0;
		let mut dt: f32 = OPTIMAL_DT;
		let mut totalFrames: u64 = 0;
		while self.flags.get(F_RUNNING) {
			let startTick = timer::ticks();
			
			// events
			self.inputHelper.update();
			while let Some(raw) = dear_imgui_sdl3::sdl3_poll_event_ll() {
				let _ = dear_imgui_sdl3::process_sys_event(&raw);
			
				let event = Event::from_ll(raw);
				self.inputHelper.handleEvents(&event);
				self.handleEvents(&event);
			}
			
			// update
			self.imgui.context.io_mut().set_delta_time(dt);
			
			if self.inputHelper.isMouseJustPressed(MouseButton::Middle) {
				self.lastMousePos = self.inputHelper.mousePos();
			}
			if self.inputHelper.isMousePressed(MouseButton::Middle) {
				let mouseDiff = self.lastMousePos - self.inputHelper.mousePos();
				if mouseDiff.length() > 0.0 {
					let lastMouseWorld = screenToWorldSpace(self.lastMousePos, self.width, self.height, self.projectionMatrix, self.viewMatrix);
					let mouseDiffWorld = screenToWorldSpace(self.lastMousePos + mouseDiff, self.width, self.height, self.projectionMatrix, self.viewMatrix);
					let worldDiff = lastMouseWorld - mouseDiffWorld;
					// println!("{}", worldDiff.z);
					
					self.camera.transform.position.x -= worldDiff.x;
					self.camera.transform.position.y -= worldDiff.y;
				}
				self.lastMousePos = self.inputHelper.mousePos();
			}
			
			// Imgui
			dear_imgui_sdl3::sdl3_new_frame(&mut self.imgui.context);
			let ui = self.imgui.context.frame();
			
			ui.dockspace_over_main_viewport();
			
			ui.window("App Info")
			  .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
			  .build(|| {
				  ui.text(format!("ImGUI FPS: {:.3}", ui.io().framerate()));
				  ui.text(format!("Delta Time: {}", dt));
				  ui.text(format!("Total frames: {}", totalFrames));
				  ui.separator();
				  
				  ui.text(format!("Mouse Position: ({:.2},{:.2})", self.inputHelper.mousePos().x, self.inputHelper.mousePos().y));
				  
				  let windowSize = self.window.read().unwrap().size();
				  ui.text(format!("Window Size: ({},{})", windowSize.0, windowSize.1));
				  ui.separator();
				  
				  let uiWidth = ui.window_width();
				  let itemWidth = ui.push_item_width(uiWidth * 0.6);
				  ui.color_edit4("Clear Color", &mut self.clearColor);
				  itemWidth.end();
			  });
			
			let mut updateProjection = false;
			ui.window("Controls")
			  .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
			  .build(|| {
				  // ui.text(format!("Update time: {}s", self.updateTime));
				  // ui.separator();
				  
				  // ui.text(format!("Render time: {}s", self.renderTime));
				  // if ui.collapsing_header("Shape Renderer", TreeNodeFlags::COLLAPSING_HEADER) {
					//   let shapeRenderer = self.renderer.getShapeRenderer();
					//   ui.checkbox("Enabled##shape", &mut shapeRenderer.enabled);
					//   ui.text(format!("Buffer capacity: {}", shapeRenderer.getBufferCapacity()));
					//   ui.text(format!("Last floats pushed: {}", shapeRenderer.getLastFloatsPushed()));
				  // }
				  if ui.collapsing_header("Line Renderer", TreeNodeFlags::COLLAPSING_HEADER) {
					  if ui.small_button("Enable##LineRenderer") {
						  self.lineRenderer.write().unwrap().enable();
					  }
					  ui.same_line();
					  if ui.small_button("Disable##LineRenderer") {
						  self.lineRenderer.write().unwrap().disable();
					  }
					  ui.text(format!("Buffer capacity: {}", self.lineRenderer.read().unwrap().getBufferCapacity()));
					  ui.text(format!("Last floats pushed: {}", self.lineRenderer.read().unwrap().getLastFloatsPushed()));
				  }
				  ui.separator();
				  
				  if ui.collapsing_header("Camera", TreeNodeFlags::COLLAPSING_HEADER) {
					  ui.text(format!("Position: ({:.3}, {:.3})", self.camera.transform.position.x, self.camera.transform.position.y));
					  let uiWidth = ui.window_width();
					  let itemWidth = ui.push_item_width(uiWidth * 0.6);
					  if ui.slider_f32("FOV/Zoom", &mut self.camera.frustum.fov, self.camera.frustum.fovMin, self.camera.frustum.fovMax) {
						  updateProjection = true;
					  }
					  itemWidth.end();
					  
					  if ui.small_button("Reset") {
						  self.camera.transform.position = Vec3::ZERO;
						  self.camera.frustum.fov = 500.0;
						  updateProjection = true;
					  }
				  }
			  });
			if updateProjection {
				self.updateProjectionMatrix();
			}
			
			let drawData = self.imgui.context.render();
			
			// render
			unsafe {
				self.gl.clear(glow::COLOR_BUFFER_BIT);
				self.gl.clear_color(self.clearColor[0], self.clearColor[1], self.clearColor[2], self.clearColor[3]);
				gl_check_error!(self.gl);
			}
			
			self.lineRenderer.write().unwrap().pushLine2(vec2(-100.0, 100.0), Vec3::X, vec2(100.0, 100.0), Vec3::Y);
			self.lineRenderer.write().unwrap().pushLine2(vec2(100.0, 100.0), Vec3::Y, vec2(100.0, -100.0), Vec3::Z);
			self.lineRenderer.write().unwrap().pushLine2(vec2(100.0, -100.0), Vec3::Z, vec2(-100.0, -100.0), Vec3::ONE);
			self.lineRenderer.write().unwrap().pushLine2(vec2(-100.0, -100.0), Vec3::ONE, vec2(-100.0, 100.0), Vec3::X);
			
			// calculate camera matrices
			self.viewMatrix = self.camera.getViewMatrix();
			let projViewMat = self.projectionMatrix * self.viewMatrix;
			
			self.renderManager.draw(&projViewMat, &self.camera);
			
			self.lineRenderer.write().unwrap().drawFlush(&projViewMat);
			
			if self.imgui.renderer.is_destroyed {
				self.imgui.renderer.create_device_objects(&self.gl)?;
			}
			self.imgui.renderer.render_with_context(&self.gl, drawData)?;
			
			#[cfg(feature = "multi-viewport")]
			{
				let ioFlags = self.imgui.context.io().config_flags();
				if ioFlags.contains(ConfigFlags::VIEWPORTS_ENABLE) {
					self.imgui.context.update_platform_windows();
					self.imgui.context.render_platform_windows_default();
					// Restore main GL context
					self.window.read().unwrap().gl_make_current(&self.glContext)?;
				}
			}
			
			self.window.read().unwrap().gl_swap_window();
			
			// fps counter
			fps = fps.saturating_add(1);
			totalFrames = totalFrames.saturating_add(1);
			if startTick >= lastTick + 1000 {
				let newTitle = format!("{} - FPS: {}", WIN_TITLE, fps);
				self.window.write().unwrap().set_title(&newTitle)?;
				
				lastTick = startTick;
				fps = 0;
			}
			
			// timing
			let elapsedTicks = timer::ticks() - startTick;
			let waitTime = OPTIMAL_WAIT_TIME.saturating_sub(elapsedTicks);
			dt = waitTime as f32 / 1000.0;
			if waitTime > 0 {
				// info!("{}", waitTime);
				timer::delay(waitTime as u32);
			}
		}
		Ok(())
	}
	
	pub fn destroy(&mut self) {
		warn!("Destroying window");
		self.lineRenderer.write().unwrap().destroy();
		#[cfg(feature = "multi-viewport")]
		glow_mvp::shutdown_multi_viewport_support(&mut self.imgui.context);
		dear_imgui_sdl3::shutdown(&mut self.imgui.context);
		self.imgui.renderer.destroy(&self.gl);
		shaders::destroyAllShaders();
	}
}