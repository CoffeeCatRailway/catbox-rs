#![allow(non_snake_case)]

mod graphics;
mod window;
mod simulation;

use std::num::NonZeroU32;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::rc::Rc;
use std::time::{Duration, Instant};
use log::info;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit_input_helper::WinitInputHelper;
use crate::window::*;//{Viewport, ViewportTest};

const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

const FPS: u32 = 60;
const TIME_STEP: f32 = 1.0 / FPS as f32;

struct State {
	glSurface: Surface<WindowSurface>,
	glContext: PossiblyCurrentContext,
	viewport: Box<dyn Viewport>,
}

struct App {
	window: Option<Rc<Window>>,
	state: Option<State>,
	input: WinitInputHelper,
	requestRedraw: bool,
	waitCancelled: bool,
	instant: Instant,
}

impl Default for App {
	fn default() -> Self {
		App {
			window: None,
			state: None,
			input: WinitInputHelper::new(),
			requestRedraw: false,
			waitCancelled: false,
			instant: Instant::now(),
		}
	}
}

impl ApplicationHandler for App {
	fn new_events(&mut self, _eventLoop: &ActiveEventLoop, cause: StartCause) {
		self.input.step();
		
		self.waitCancelled = match cause {
			StartCause::WaitCancelled { .. } => true,
			_ => false,
		}
	}
	
	fn resumed(&mut self, eventLoop: &ActiveEventLoop) {
		if self.state.is_some() {
			return;
		}
		
		let attributes = WindowAttributes::default()
			// .with_fullscreen(Some(Fullscreen::Borderless(None)))
			.with_inner_size(PhysicalSize::new(WIN_WIDTH, WIN_HEIGHT))
			.with_title("CatBox Native");
		
		let template = ConfigTemplateBuilder::new();
		let displayBuilder = DisplayBuilder::new().with_window_attributes(Some(attributes));
		
		let (window, glConfig) = displayBuilder
			.build(eventLoop, template, |configs| {
				configs
					.reduce(|accum, config| {
						if config.num_samples() > accum.num_samples() {
							config
						} else {
							accum
						}
					})
					.unwrap()
			})
			.unwrap();
		let rwh: Option<RawWindowHandle> = window
			.as_ref()
			.and_then(|w| w.window_handle().map(Into::into).ok());
		
		let glDisplay = glConfig.display();
		let contextAttributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
				major: 4,
				minor: 1,
			})))
			.build(rwh);
		
		let (window, gl, glSurface, glContext) = unsafe {
			let notCurrentGlContext = glDisplay
				.create_context(&glConfig, &contextAttributes)
				.unwrap();
			let window = Rc::new(window.unwrap());
			
			let surfaceAttributes = window.build_surface_attributes(Default::default()).unwrap();
			let glSurface = glDisplay
				.create_window_surface(&glConfig, &surfaceAttributes)
				.unwrap();
			
			let glContext = notCurrentGlContext.make_current(&glSurface).unwrap();
			let gl = Rc::new(glow::Context::from_loader_function_cstr(|s| glDisplay.get_proc_address(s)));
			// glSurface.set_swap_interval(&glContext, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();
			
			(window, gl, glSurface, glContext)
		};
		
		// let viewport = ViewportTest::new(window.clone(), gl.clone());
		let viewport = ViewportSim::new(window.clone(), gl.clone());
		
		self.window = Some(window.clone());
		self.state = Some(State {
			glSurface,
			glContext,
			viewport: Box::new(viewport),
		});
	}
	
	fn window_event(&mut self, eventLoop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
		self.input.process_window_event(&event);
		match event {
			WindowEvent::Resized(size) => {
				self.requestRedraw = true;
				if let Some(ref mut state) = self.state {
					// #[cfg(target_os = "linux")]
					state.glSurface.resize(&state.glContext, NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap());
					state.viewport.resize(size.width, size.height);
				}
			},
			WindowEvent::CloseRequested => {
				info!("The close button was pressed; stopping");
				eventLoop.exit();
			},
			WindowEvent::RedrawRequested => {
				if let Some(ref mut state) = self.state {
					state.viewport.render(TIME_STEP);
					self.window.as_ref().unwrap().pre_present_notify();
					state.glSurface.swap_buffers(&state.glContext).unwrap();
				}
				
				// self.window.as_ref().unwrap().request_redraw();
			},
			_ => (),
		}
	}
	
	fn device_event(&mut self, _eventLoop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
		self.input.process_device_event(&event);
	}
	
	fn about_to_wait(&mut self, eventLoop: &ActiveEventLoop) {
		self.input.end_step();
		
		if let Some(ref mut state) = self.state {
			let dt = self.input.delta_time().unwrap().as_secs_f32();
			// info!("Delta time: {}s", dt);
			state.viewport.handleInput(dt, &self.input, eventLoop);
		}
		
		if self.requestRedraw && !self.waitCancelled {
			self.window.as_ref().unwrap().request_redraw();
			self.requestRedraw = false;
			
			if let Some(ref mut state) = self.state {
				let dt = TIME_STEP;//self.instant.elapsed().as_secs_f32();
				// info!("Delta time: {}s", dt);
				state.viewport.update(dt, eventLoop);
			}
		}
		
		if !self.waitCancelled {
			self.instant = Instant::now();
			eventLoop.set_control_flow(ControlFlow::WaitUntil(self.instant + Duration::from_secs_f32(TIME_STEP)));
			self.requestRedraw = true;
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
	eventLoop
		.run_app(&mut App::default()).expect("Failed to run event loop");
}
