use std::error::Error;
use std::f32::consts::{PI, TAU};
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};
use bool_flags::Flags8;
use dear_imgui_glow::{GlowRenderer, SimpleTextureMap};
#[cfg(feature = "multi-viewport")]
use dear_imgui_glow::multi_viewport as glow_mvp;
use dear_imgui_rs::{ChildFlags, ConfigFlags, Context as ImguiContext, WindowFlags};
use glam::{vec3, Mat4, Vec3};
use glow::HasContext;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseUtil;
use sdl3::video::{GLContext, GLProfile, SwapInterval};
use tracing::{info, warn};
use crate::gl_check_error;
use crate::graphics::{RenderManager, SimpleRenderable};
use crate::graphics::mesh::{Primitives2D, Primitives3D};
use crate::graphics::shaders;
use crate::simulation::{Solver, Transform};
use crate::types::{newGlRef, newMeshRef, newRenderableRef, newSdlWindowRef, newSolverRef, GlRef, SdlWindowRef, SolverRef};
use crate::window::InputHelper;
use crate::window::camera::{Camera, Frustum, Projection};

const F_RUNNING: u8 = 0;
const F_MOUSE_CAPTURED: u8 = 1;
const F_WIREFRAME: u8 = 2;

const WIN_TITLE: &str = "Physics CatBox";
const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: f32 = 1000.0 / FPS as f32;
const OPTIMAL_DT: f32 = OPTIMAL_WAIT_TIME / 1000.0;

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
	mouseUtil: MouseUtil,
	
	imgui: Imgui,
	
	solver: SolverRef,
	renderManager: RenderManager,
	clearColor: [f32; 4],
	// lastMousePos: Vec2,
	
	camera: Camera,
	projectionMatrix: Mat4,
	viewMatrix: Mat4,
}

impl CatBox {
	pub fn new() -> Result<CatBox, Box<dyn Error>> {
		info!("Creating CatBox");
		let flags = Flags8::none();
	
		// Initialize sdl, gl and imgui
		info!("SDL3 context");
		let sdl = sdl3::init()?;
		let video = sdl.video()?;
		let glAttributes = video.gl_attr();
		
		let inputHelper = InputHelper::new();
		
		glAttributes.set_context_profile(GLProfile::Core);
		glAttributes.set_context_version(4, 5);
		
		info!("Window and GL context");
		let window = video.window(WIN_TITLE, WIN_WIDTH, WIN_HEIGHT)
						  .opengl()
						  .resizable()
						  .position_centered()
						  .build()?;
		let window = newSdlWindowRef(window);
		
		let glContext = window.gl_create_context()?;
		
		window.gl_make_current(&glContext)?;
		video.gl_set_swap_interval(SwapInterval::Immediate)?;
		
		let gl = unsafe {
			use std::ffi::c_void;
			let gl = glow::Context::from_loader_function(|name| {
				video.gl_get_proc_address(name).map(|f| f as *const c_void).unwrap_or(std::ptr::null())
			});
			newGlRef(gl)
		};
		
		unsafe {
			gl.enable(glow::CULL_FACE);
			gl.cull_face(glow::BACK);
			gl.front_face(glow::CCW);
			
			gl.enable(glow::DEPTH_TEST);
			gl.depth_func(glow::LESS);
			gl.depth_mask(true);
		}
		
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
		dear_imgui_sdl3::init_platform_for_opengl(&mut imgui, &window, &glContext)?;
		
		// Basic style scaling
		let windowScale = window.display_scale();
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
		
		// Initialize renderers, shaders and camera
		info!("Initializing locals");
		let simpleLightShader = shaders::simpleLightShader(gl.clone())?;
		// let instanceShader = shaders::instanceShader(gl.clone())?;
		//
		let solver = newSolverRef(Solver::new()?);
		let mut renderManager = RenderManager::new(gl.clone())?;
		renderManager.lineRendererMut().enable(true);
		// renderManager.addRenderable(solver.clone());
		
		let camera = Camera {
			frustum: Frustum {
				far: 500.0,
				// fov: 500.0,
				// fovMax: 10000.0,
				..Frustum::default()
			},
			transform: Transform {
				position: vec3(0.0, 20.0, 100.0),
				..Transform::default()
			},
			..Camera::default()
		};
		
		let meshNow = Instant::now();
		// let mesh = Primitives2D::circleXY(20, 20.0);
		// let mesh = Primitives2D::squareXY(10.0, 10.0).subdivideMesh();
		
		// let mesh = Primitives3D::sphereUV(10, 10, 10.0);
		// let mesh = Primitives3D::tetrahedron(10.0);
		// let mesh = Primitives3D::cube(10.0, 10.0, 10.0);
		// let mesh = Primitives3D::sphereCube(10.0);
		let mesh = Primitives3D::icosphere(10.0, 2);
		
		info!("Mesh vertex/triangle count: {}/{}", mesh.vertices().len(), mesh.triangles().len());
		let mut mesh = mesh.buildSimpleMesh(gl.clone());
		
		let meshEnd = meshNow.elapsed().as_micros();
		info!("Mesh build took: {}ms", meshEnd as f32 / 1000.0);
		mesh.upload(simpleLightShader.clone())?;
		
		let simpleRenderable = SimpleRenderable {
			transform: {
				let mut transform = Transform::default();
				// transform.setRotationFromDirection(Vec3::NEG_X * PI / 4.0);
				transform
			},
			mesh: newMeshRef(mesh),
			shader: simpleLightShader.clone(),
		};
		let simpleRenderable = newRenderableRef(simpleRenderable);
		renderManager.addRenderable(simpleRenderable);
		
		// Setup physicals
		// let a: u32 = 30*30;
		// let sq = (a as f32).sqrt();
		// let s = 10.0;
		// let sh = s / 2.0;
		// for i in 0..a {
		// 	let x = (i % sq as u32) as f32;
		// 	let y = (i / sq as u32) as f32;
		// 	// let size = Vec3::splat(((i as f32 / 2.0).sin() * 0.25 + 0.75) * s);
		// 	let size = Vec3::splat(s);
		// 	let mut ball = Ball::new(Vec3::new(x * s - sq * sh + sh, y * s - sq * sh + sh, 0.0), size);
		// 	ball.lastTransform.position = ball.transform.position - ball.transform.position.normalize_or_zero() * OPTIMAL_DT * 10.0;
		// 	// ball.elasticity = 0.5;
		//
		// 	ball.color.x = x / sq;
		// 	ball.color.y = y / sq;
		// 	ball.color.z = 0.0;
		//
		// 	let ball = newPhysicalRef(ball);
		// 	solver.borrow_mut().addPhysical(ball);
		// }
		//
		// let ballRenderable = BallRenderable::new(gl.clone(), instanceShader.clone(), solver.clone());
		// ballRenderable.meshRef().unwrap().borrow_mut().upload(instanceShader.clone())?;
		//
		// let ballRenderable = newRenderableRef(ballRenderable);
		// renderManager.addRenderable(ballRenderable.clone());
		
		let mut catbox = CatBox {
			width: WIN_WIDTH,
			height: WIN_HEIGHT,
			flags,
			inputHelper,
			
			gl,
			glContext,
			window,
			mouseUtil: sdl.mouse(),
			
			imgui: Imgui {
				context: imgui,
				renderer: imguiRenderer,
			},
			
			solver,
			renderManager,
			clearColor: [96.0 / 255.0, 190.0 / 255.0, 200.0 / 255.0, 1.0],
			// lastMousePos: Vec2::ZERO,
			
			camera,
			projectionMatrix: Mat4::IDENTITY,
			viewMatrix: Mat4::IDENTITY,
		};
		catbox.updateProjectionMatrix();
		Ok(catbox)
	}
	
	fn updateProjectionMatrix(&mut self) {
		let windowSize = self.window.size();
		let windowAspect = windowSize.0 as f32 / windowSize.1 as f32;
		
		let projection = Projection::Perspective(windowAspect);
		// let projection = Projection::Orthographic(windowAspect * -1.0, windowAspect * 1.0, -1.0, 1.0);
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
					if window_id == self.window.id() {
						self.requestClose();
					}
				},
				_ => {},
			},
			Event::MouseWheel { y, .. } => {
				if !self.imgui.context.io().want_capture_mouse() {
					self.camera.frustum.zoom(-y * 1.0);
					self.updateProjectionMatrix();
				}
			},
			Event::MouseMotion { xrel, yrel, .. } => {
				if self.flags.get(F_MOUSE_CAPTURED) {
					self.camera.turn(xrel, -yrel);
				}
			}
			_ => {},
		}
	}
	
	fn input(&mut self, dt: f32) {
		// Toggle wireframe
		if self.inputHelper.isKeyJustPressed(Keycode::_2) {
			self.flags.flip(F_WIREFRAME);
			unsafe {
				if self.flags.get(F_WIREFRAME) {
					self.gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
				} else {
					self.gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
				}
			}
		}
		
		// Toggle mouse capture
		let mut mouseCaptured = self.flags.get(F_MOUSE_CAPTURED);
		if self.inputHelper.isKeyJustPressed(Keycode::_1) {
			self.flags.flip(F_MOUSE_CAPTURED);
			mouseCaptured = self.flags.get(F_MOUSE_CAPTURED);
			
			self.mouseUtil.set_relative_mouse_mode(&self.window, mouseCaptured);
			info!("Mouse captured: {}", mouseCaptured);
			
			let mut imguiConfigFlags = self.imgui.context.io().config_flags();
			if mouseCaptured {
				imguiConfigFlags.insert(ConfigFlags::NO_MOUSE);
			} else {
				imguiConfigFlags.remove(ConfigFlags::NO_MOUSE);
			}
			self.imgui.context.io_mut().set_config_flags(imguiConfigFlags);
		}
		
		// Camera WASD
		if mouseCaptured && !self.imgui.context.io().want_capture_keyboard() {
			if self.inputHelper.isKeyPressed(Keycode::W) {
				self.camera.transform.translateLocalForward(30.0 * dt);
			}
			if self.inputHelper.isKeyPressed(Keycode::S) {
				self.camera.transform.translateLocalForward(-30.0 * dt);
			}
			if self.inputHelper.isKeyPressed(Keycode::A) {
				self.camera.transform.translateLocalRight(-30.0 * dt);
			}
			if self.inputHelper.isKeyPressed(Keycode::D) {
				self.camera.transform.translateLocalRight(30.0 * dt);
			}
			if self.inputHelper.isKeyPressed(Keycode::Space) {
				self.camera.transform.translateGlobal(Vec3::Y * 30.0 * dt);
			}
			if self.inputHelper.isKeyPressed(Keycode::LCtrl) {
				self.camera.transform.translateGlobal(Vec3::Y * -30.0 * dt);
			}
		}
	}
	
	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
		info!("Starting CatBox loop");
		self.flags.set(F_RUNNING);
		
		let mut fps: u32 = 0;
		let mut frameLast = Instant::now();
		let mut dt: f32 = OPTIMAL_DT;
		let mut totalFrames: u64 = 0;
		let mut t = 0.0;
		while self.flags.get(F_RUNNING) {
			let frameStart = Instant::now();
			
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
			
			self.input(dt);
			
			let sun = self.renderManager.sunLightMut();
			t += dt;
			t = t % TAU;
			sun.propertiesMut().position.x = t.sin();
			sun.propertiesMut().position.z = t.cos();
			
			self.solver.borrow_mut().update(OPTIMAL_DT);
			{
				let s: f32 = 500.0;
				let hs = s / 2.0;
				let n = 50;
				for i in 0..n+1 {
					let x = (i as f32 / n as f32) * s - hs;
					let mut col = Vec3::ONE;
					if x.abs() - 0.0 < f32::EPSILON {
						col = Vec3::X;
					}
					self.renderManager.lineRendererMut().pushLine3(Vec3::new(x, 0.0, hs), col, Vec3::new(x, 0.0, -hs), col);
					(col.x, col.z) = (col.z, col.x);
					self.renderManager.lineRendererMut().pushLine3(Vec3::new(hs, 0.0, x), col, Vec3::new(-hs, 0.0, x), col);
				}
			}
			
			// Imgui
			dear_imgui_sdl3::sdl3_new_frame(&mut self.imgui.context);
			let ui = self.imgui.context.frame();
			
			// ui.dockspace_over_main_viewport();
			
			let wireframe = self.flags.get(F_WIREFRAME);
			let mut updateProjection = false;
			ui.window("App Info")
			  .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
			  .build(|| {
				  let flags = ChildFlags::AUTO_RESIZE_X | ChildFlags::AUTO_RESIZE_Y;
				  ui.child_window("##info")
					  .child_flags(flags)
					  .build(ui, || {
						  ui.text(format!("ImGUI FPS: {:.3}", ui.io().framerate()));
						  ui.text(format!("Delta Time: {}", dt));
						  ui.text(format!("Total frames: {}", totalFrames));
						  ui.separator();
						  
						  ui.text(format!("Mouse captured (Press 1): {}", self.flags.get(F_MOUSE_CAPTURED)));
						  ui.text(format!("Mouse Position: ({:.2},{:.2})", self.inputHelper.mousePos().x, self.inputHelper.mousePos().y));
						  
						  ui.text(format!("Toggle wireframe (Press 2): {}", wireframe));
						  
						  let windowSize = self.window.size();
						  ui.text(format!("Window Size: ({},{})", windowSize.0, windowSize.1));
						  ui.separator();
						  
						  let uiWidth = ui.window_width();
						  let itemWidth = ui.push_item_width(uiWidth * 0.6);
						  ui.color_edit4("Clear Color", &mut self.clearColor);
						  itemWidth.end();
					  });
				  ui.same_line();
				  ui.separator_vertical();
				  ui.same_line();
				  ui.child_window("##controls")
					  .child_flags(flags)
					  .build(ui, || {
						  ui.text("Line Renderer:");
						  let lineRendererMut = self.renderManager.lineRendererMut();
						  ui.text(format!("Buffer capacity: {}", lineRendererMut.getBufferCapacity()));
						  ui.text(format!("Last floats pushed: {}", lineRendererMut.getLastFloatsPushed()));
						  ui.separator();
						  
						  ui.text("Camera:");
						  ui.text(format!("Position: ({:.3}, {:.3}, {:.3})", self.camera.transform.position.x, self.camera.transform.position.y, self.camera.transform.position.z));
						  let uiWidth = ui.window_width();
						  let itemWidth = ui.push_item_width(uiWidth * 0.6);
						  if ui.slider_f32("FOV/Zoom", &mut self.camera.frustum.fov, self.camera.frustum.fovMin, self.camera.frustum.fovMax) {
							  updateProjection = true;
						  }
						  itemWidth.end();
					  });
			  });
			
			self.solver.borrow_mut().gui(ui, OPTIMAL_DT);
			
			if updateProjection {
				self.updateProjectionMatrix();
			}
			
			let drawData = self.imgui.context.render();
			
			// render
			unsafe {
				self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
				self.gl.clear_color(self.clearColor[0], self.clearColor[1], self.clearColor[2], self.clearColor[3]);
				gl_check_error!(self.gl);
				
				// calculate camera matrices
				self.viewMatrix = self.camera.getViewMatrix();
				let projViewMat = self.projectionMatrix * self.viewMatrix;
				
				self.renderManager.draw(&projViewMat, dt, &self.camera)?;
				
				if wireframe {
					self.gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
				}
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
						self.window.gl_make_current(&self.glContext)?;
					}
				}
				if wireframe {
					self.gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
				}
				
				self.window.gl_swap_window();
			}
			
			// fps counter
			fps = fps.saturating_add(1);
			totalFrames = totalFrames.saturating_add(1);
			if frameStart >= frameLast + Duration::from_millis(1000) {
				let newTitle = format!("{} - FPS: {}", WIN_TITLE, fps);
				Rc::get_mut(&mut self.window).unwrap().set_title(&newTitle)?;
				
				frameLast = frameStart;
				fps = 0;
			}
			
			// timing
			let elapsedMillis = frameStart.elapsed().as_micros() as f32 / 1000.0;
			let waitTime = (OPTIMAL_WAIT_TIME - elapsedMillis).max(f32::EPSILON);
			dt = waitTime / 1000.0;
			if waitTime as u32 > 0 {
				// info!("{}", waitTime);
				thread::sleep(Duration::from_millis(waitTime as u64));
			}
		}
		Ok(())
	}
	
	pub fn destroy(&mut self) {
		warn!("Destroying window");
		self.solver.borrow_mut().destroy();
		self.renderManager.destroy();
		#[cfg(feature = "multi-viewport")]
		glow_mvp::shutdown_multi_viewport_support(&mut self.imgui.context);
		dear_imgui_sdl3::shutdown(&mut self.imgui.context);
		self.imgui.renderer.destroy(&self.gl);
		shaders::destroyAllShaders();
	}
}
