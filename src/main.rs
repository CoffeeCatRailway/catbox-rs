#![allow(non_snake_case)]

mod graphics;
mod window;
mod simulation;

use std::error::Error;
use std::num::NonZeroU32;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::sync::Arc;
use std::time::{Duration, Instant};
use dear_imgui_glow::GlowRenderer;
use dear_imgui_rs::WindowFlags;
use dear_imgui_winit::WinitPlatform;
use glow::HasContext;
use log::{error, info};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit_input_helper::WinitInputHelper;
use crate::window::*;

const WIN_WIDTH: u32 = 1600;
const WIN_HEIGHT: u32 = 900;

const TARGET_UPS: u32 = 60;
const STEP_DT: f32 = 1.0 / TARGET_UPS as f32;

struct ImguiWrapper {
	context: dear_imgui_rs::Context,
	platform: WinitPlatform,
	renderer: GlowRenderer,
	clearColor: [f32; 4],
	lastFrame: Instant,
}

struct AppState {
	window: Arc<Window>,
	surface: Surface<WindowSurface>,
	context: PossiblyCurrentContext,
	
	imgui: ImguiWrapper,
	input: WinitInputHelper,
	viewport: Viewport,
	
	requestRedraw: bool,
	waitCancelled: bool,
}

struct App {
	state: Option<AppState>,
}

impl AppState {
	fn new(eventLoop: &ActiveEventLoop) -> Result<Self, Box<dyn Error>> {
		// Window
		let attributes = WindowAttributes::default()
			.with_inner_size(PhysicalSize::new(WIN_WIDTH, WIN_HEIGHT))
			.with_title("Physics CatBox");
		
		let (window, config) = DisplayBuilder::new()
			.with_window_attributes(Some(attributes))
			.build(eventLoop, ConfigTemplateBuilder::new(), |configs| {
				configs
					.reduce(|accum, config| {
						if config.num_samples() > accum.num_samples() {
							config
						} else {
							accum
						}
					})
					.unwrap()
			})?;
		let window = Arc::new(window.unwrap());
		info!("Window initialized");
		
		// OpenGL
		let contextAttributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
				major: 4,
				minor: 1,
			})))
			.build(Some(window.window_handle()?.as_raw()));
		
		let (gl, surface, context) = unsafe {
			let display = config.display();
			let notCurrentGlContext = display.create_context(&config, &contextAttributes)?;
			
			let surfaceAttributes = window.build_surface_attributes(Default::default())?;
			let surface = display.create_window_surface(&config, &surfaceAttributes)?;
			
			let context = notCurrentGlContext.make_current(&surface)?;
			let gl = glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s));
			
			// surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))?;
			// surface.set_swap_interval(&context, SwapInterval::DontWait)?;
			
			(gl, surface, context)
		};
		info!("OpenGL initialized");
		
		// Imgui
		let mut imguiContext = dear_imgui_rs::Context::create();
		imguiContext.set_ini_filename(Some("imgui.ini"))?;
		
		let mut platform = WinitPlatform::new(&mut imguiContext);
		platform.attach_window(
			&window,
			dear_imgui_winit::HiDpiMode::Default,
			&mut imguiContext
		);
		
		let mut imguiRenderer = GlowRenderer::new(gl, &mut imguiContext)?;
		imguiRenderer.set_framebuffer_srgb_enabled(false);
		imguiRenderer.new_frame()?;
		let imgui = ImguiWrapper {
			context: imguiContext,
			platform,
			renderer: imguiRenderer,
			clearColor: [0.27, 0.59, 0.27, 1.0],
			lastFrame: Instant::now(),
		};
		info!("Imgui initialized");
		
		// Simulation
		let viewport = Viewport::new(window.clone(), imgui.renderer.gl_context().unwrap().clone());
		
		Ok(AppState {
			window,
			surface,
			context,
			
			imgui,
			input: WinitInputHelper::new(),
			viewport,
			
			requestRedraw: false,
			waitCancelled: false,
		})
	}
	
	fn resize(&mut self, size: PhysicalSize<u32>) {
		self.requestRedraw = true;
		let (width, height) = (size.width.max(1), size.height.max(1));
		self.surface.resize(&self.context, NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap());
		self.viewport.resize(width, height);
	}
	
	fn update(&mut self, eventLoop: &ActiveEventLoop) {
		self.input.end_step();
		
		let dt = if let Some(dt) = self.input.delta_time() {
			dt.as_secs_f32()
		} else {
			STEP_DT
		};
		// info!("Delta time: {}s", dt);
		self.viewport.handleInput(dt, &self.input, eventLoop);
		
		if self.requestRedraw && !self.waitCancelled {
			self.window.request_redraw();
			self.requestRedraw = false;
			
			self.viewport.update(STEP_DT, eventLoop);
		}
		
		if !self.waitCancelled {
			let now = Instant::now();
			eventLoop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_secs_f32(STEP_DT)));
			self.requestRedraw = true;
		}
	}
	
	fn render(&mut self) -> Result<(), Box<dyn Error>> {
		let gl = self.imgui.renderer.gl_context().unwrap();
		unsafe {
			gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
			gl.clear_color(self.imgui.clearColor[0], self.imgui.clearColor[1], self.imgui.clearColor[2], self.imgui.clearColor[3]);
		}
		
		// Render simulation
		let dt = self.imgui.lastFrame.elapsed().as_secs_f32();
		self.viewport.render(dt);
		
		// UI
		self.imgui.context.io_mut().set_delta_time(dt);
		self.imgui.lastFrame = Instant::now();
		
		self.imgui.platform.prepare_frame(&self.window, &mut self.imgui.context);
		let ui = self.imgui.context.frame();
		
		ui.window("App Info")
		  .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
		  .build(|| {
			  ui.text(format!("ImGUI FPS: {:.2}", ui.io().framerate()));
			  ui.text(format!("ImGUI dt: {}", dt));
			  // total frames
			  // fps/ups
			  ui.separator();
			  
			  let mousePos = if let Some(cursor) = self.input.cursor() {
				  cursor
			  } else {
				  (0.0, 0.0)
			  };
			  ui.text(format!("Mouse Position: ({:.2},{:.2})", mousePos.0, mousePos.1));
			  
			  let windowSize = self.window.inner_size();
			  ui.text(format!("Window Size: ({},{})", windowSize.width, windowSize.height));
			  ui.separator();
			  
			  let uiWidth = ui.window_width();
			  let itemWidth = ui.push_item_width(uiWidth * 0.6);
			  ui.color_edit4("Clear Color", &mut self.imgui.clearColor);
			  itemWidth.end();
		  });
		self.viewport.gui(ui);
		
		// Render UI
		self.imgui.platform.prepare_render_with_ui(&ui, &self.window);
		let drawData = self.imgui.context.render();
		
		self.imgui.renderer.new_frame()?;
		self.imgui.renderer.render(&drawData)?;
		
		// Swap
		self.window.pre_present_notify();
		self.surface.swap_buffers(&self.context)?;
		Ok(())
	}
}

impl ApplicationHandler for App {
	fn new_events(&mut self, _eventLoop: &ActiveEventLoop, cause: StartCause) {
		if let Some(ref mut state) = self.state {
			state.input.step();
			
			state.waitCancelled = match cause {
				StartCause::WaitCancelled { .. } => true,
				_ => false,
			};
		}
	}
	
	fn resumed(&mut self, eventLoop: &ActiveEventLoop) {
		if self.state.is_none() {
			match AppState::new(eventLoop) {
				Ok(state) => {
					state.window.request_redraw();
					self.state = Some(state);
					info!("App state created");
				},
				Err(e) => {
					error!("Error creating AppState: {}", e);
					eventLoop.exit();
				}
			}
		}
	}
	
	fn window_event(&mut self, eventLoop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
		if let Some(ref mut state) = self.state {
			state.input.process_window_event(&event);
			
			state.imgui.platform.handle_window_event(&mut state.imgui.context, &state.window, &event);
			
			match event {
				WindowEvent::Resized(size) => {
					state.resize(size);
				},
				WindowEvent::CloseRequested => {
					info!("The close button was pressed; stopping");
					eventLoop.exit();
				},
				WindowEvent::RedrawRequested => {
					if let Err(e) = state.render() {
						error!("Error rendering AppState: {}", e);
					}
					// state.window.request_redraw();
				},
				_ => (),
			}
		}
	}
	
	fn device_event(&mut self, _eventLoop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
		if let Some(ref mut state) = self.state {
			state.input.process_device_event(&event);
		}
	}
	
	fn about_to_wait(&mut self, eventLoop: &ActiveEventLoop) {
		if let Some(ref mut state) = self.state {
			state.update(eventLoop);
		}
	}
	
	fn exiting(&mut self, _eventLoop: &ActiveEventLoop) {
		if let Some(ref mut state) = self.state {
			info!("Exiting");
			state.viewport.destroy();
		}
	}
}

fn main() {
	tracing_subscriber::fmt::fmt()
		.with_ansi(true)
		.with_target(false)
		.with_file(true)
		.with_line_number(true)
		.with_thread_names(true)
		.with_thread_ids(false)
		.compact()
		.with_max_level(tracing::Level::INFO)
		.init();
	
	info!("Hello, world!");
	// panic!("hi");
	
	let eventLoop = EventLoop::new().unwrap();
	// eventLoop.set_control_flow(ControlFlow::Poll);
	eventLoop.run_app(&mut App { state: None, }).expect("Failed to run event loop");
}
