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
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::rc::Rc;
use std::time::{Duration, Instant};
use log::{error, info};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit_input_helper::WinitInputHelper;
use crate::window::*;

const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

const FPS: u32 = 60;
const TIME_STEP: f32 = 1.0 / FPS as f32;

struct AppState {
	window: Rc<Window>,
	surface: Surface<WindowSurface>,
	context: PossiblyCurrentContext,
	input: WinitInputHelper,
	requestRedraw: bool,
	waitCancelled: bool,
	viewport: Box<dyn Viewport>,
}

#[derive(Default)]
struct App {
	state: Option<AppState>,
}

impl AppState {
	fn new(eventLoop: &ActiveEventLoop) -> Result<Self, Box<dyn Error>> {
		let attributes = WindowAttributes::default()
			// .with_fullscreen(Some(Fullscreen::Borderless(None)))
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
		let window = Rc::new(window.unwrap());
		
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
			let gl = Rc::new(glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s)));
			// glSurface.set_swap_interval(&glContext, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();
			
			(gl, surface, context)
		};
		
		// let viewport = ViewportTest::new(window.clone(), gl.clone());
		let viewport = ViewportSim::new(window.clone(), gl.clone());
		
		Ok(AppState {
			window,
			surface,
			context,
			input: WinitInputHelper::new(),
			requestRedraw: false,
			waitCancelled: false,
			viewport: Box::new(viewport),
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
		
		// Use `if let` because first cycle is None
		// let dt = state.input.delta_time().unwrap().as_secs_f32();
		let dt = if let Some(time) = self.input.delta_time() {
			time.as_secs_f32()
		} else {
			TIME_STEP
		};
		// info!("Delta time: {}s", dt);
		self.viewport.handleInput(dt, &self.input, eventLoop);
		
		if self.requestRedraw && !self.waitCancelled {
			self.window.request_redraw();
			self.requestRedraw = false;
			
			let dt = TIME_STEP; //self.instant.elapsed().as_secs_f32();
			// info!("Delta time: {}s", dt);
			self.viewport.update(dt, eventLoop);
		}
		
		if !self.waitCancelled {
			let now = Instant::now();
			eventLoop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_secs_f32(TIME_STEP)));
			self.requestRedraw = true;
		}
	}
	
	fn render(&mut self) -> Result<(), Box<dyn Error>> {
		self.viewport.render(TIME_STEP);
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
			}
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
	env_logger::builder().filter_level(log::LevelFilter::Info).init();
	log_panics::init();
	
	info!("Hello, world!");
	// panic!("hi");
	
	let eventLoop = EventLoop::new().unwrap();
	let mut app = App::default();
	eventLoop.run_app(&mut app).expect("Failed to run event loop");
}
