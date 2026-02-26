#![allow(non_snake_case)]

use std::error::Error;
use std::sync::Arc;
use glow::HasContext;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::video::{GLProfile, SwapInterval};
use tracing::{info, warn};
use sdl3::timer;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: u64 = 1000 / FPS;

const WIN_TITLE: &str = "Physics CatBox";
const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

fn initializeTracing() -> Result<(), Box<dyn Error>> {
	use std::fs::File;
	use tracing_subscriber::{fmt, filter, prelude::*};
	
	// console
	let stdoutLog = fmt::layer()
		.with_ansi(true)
		.with_target(false)
		.with_file(true)
		.with_line_number(true)
		.with_thread_names(true)
		.with_thread_ids(false)
		.with_filter(filter::LevelFilter::INFO);
	
	// file
	let file = File::create("latest.log")?;
	let fileLog = fmt::layer()
		.with_writer(Arc::new(file))
		.with_ansi(false)
		.with_target(true)
		.with_file(true)
		.with_line_number(true)
		.with_thread_names(true)
		.with_thread_ids(true)
		.with_filter(filter::LevelFilter::INFO);
	
	// combine
	tracing_subscriber::registry().with(stdoutLog).with(fileLog).init();
	
	info!("Hello world from tracing!");
	Ok(())
}

fn createSdl3GlContext(title: &str, width: u32, height: u32) -> Result<(
	glow::Context,
	sdl3::video::Window,
	sdl3::EventPump,
	sdl3::video::GLContext,
), Box<dyn Error>> {
	info!("Creating SDL3 context");
	let sdl = sdl3::init()?;
	let video = sdl.video()?;
	let glAttributes = video.gl_attr();
	
	glAttributes.set_context_profile(GLProfile::Core);
	glAttributes.set_context_version(4, 3);
	glAttributes.set_depth_size(0);
	
	info!("Creating window and GL context");
	let window = video.window(title, width, height)
		.opengl()
		.resizable()
		.position_centered()
		.build()?;
	let glContext = window.gl_create_context()?;
	window.gl_make_current(&glContext)?;
	let _ = video.gl_set_swap_interval(SwapInterval::Immediate);
	
	let gl = unsafe {
		use std::ffi::c_void;
		glow::Context::from_loader_function(|name| {
			video.gl_get_proc_address(name).map(|f| f as *const c_void).unwrap_or(std::ptr::null())
		})
	};
	let eventLoop = sdl.event_pump()?;
	
	Ok((gl, window, eventLoop, glContext))
}

fn main() -> Result<(), Box<dyn Error>> {
	// initialize
	initializeTracing()?;
	
	let (gl, mut window, mut eventLoop, _glContext) = createSdl3GlContext(WIN_TITLE, WIN_WIDTH, WIN_HEIGHT)?;
	
	info!("Starting main loop");
	let mut fps: u64 = 0;
	let mut lastTick: u64 = 0;
	// let mut dt: f32 = OPTIMAL_WAIT_TIME as f32 / 1000.0;
	'main: loop {
		let startTick = timer::ticks();
		
		// events
		for event in eventLoop.poll_iter() {
			match event {
				Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					warn!("Exiting main loop");
					break 'main
				},
				Event::Window { win_event, .. } => match win_event {
					WindowEvent::Resized(w, h) => unsafe {
						gl.viewport(0, 0, w.max(1), h.max(1));
					}
					_ => {},
				}
				_ => {},
			}
		}
		
		// update
		
		// render
		unsafe {
			gl.clear(glow::COLOR_BUFFER_BIT);
			gl.clear_color(0.27, 0.59, 0.27, 1.0);
		}
		
		window.gl_swap_window();
		
		// fps counter
		fps += 1;
		if startTick > lastTick + 1000 {
			let newTitle = format!("{} - FPS: {}", WIN_TITLE, fps);
			window.set_title(&newTitle)?;
			
			lastTick = startTick;
			fps = 0;
		}
		
		// timing
		let elapsedTicks = timer::ticks() - startTick;
		let waitTime = OPTIMAL_WAIT_TIME.saturating_sub(elapsedTicks);
		// dt = waitTime as f32 / 1000.0;
		if waitTime > 0 {
			// info!("{}", waitTime);
			timer::delay(waitTime as u32);
		}
	}
	
	// destroy
	info!("Cleaning up");
	
	Ok(())
}