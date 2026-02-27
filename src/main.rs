#![allow(non_snake_case)]

use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;
use dear_imgui_glow::{
	GlowRenderer,
	multi_viewport as glow_mvp
};
use dear_imgui_rs::{
	ConfigFlags,
	Context as ImguiContext,
	WindowFlags
};
use glow::{Context as GlowContext, HasContext};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::video::{
	GLProfile,
	GLContext as SdlGLContext,
	SwapInterval,
	Window as SdlWindow
};
use tracing::{info, warn};
use sdl3::timer;

const FPS: u64 = 60;
const OPTIMAL_WAIT_TIME: u64 = 1000 / FPS;

const WIN_TITLE: &str = "Physics CatBox";
const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

const IMGUI_VIEWPORTS: bool = true;

struct ImGui {
	context: ImguiContext,
	renderer: GlowRenderer,
}

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
	GlowContext,
	SdlWindow,
	SdlGLContext,
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
	
	Ok((gl, window, glContext))
}

fn initializeImGui(gl: GlowContext, window: &SdlWindow, glContext: &SdlGLContext) -> Result<(Rc<GlowContext>, ImGui), Box<dyn Error>> {
	info!("Initializing ImGui");
	// Build ImGui context
	let mut imgui = ImguiContext::create();
	{
		let io = imgui.io_mut();
		let mut flags= io.config_flags();
		flags.insert(ConfigFlags::DOCKING_ENABLE);
		if IMGUI_VIEWPORTS {
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

	// Initialize Glow renderer
	let mut renderer = GlowRenderer::new(gl, &mut imgui)?;
	if IMGUI_VIEWPORTS {
		glow_mvp::enable(&mut renderer, &mut imgui);
	}

	Ok((renderer.gl_context().unwrap().clone(), ImGui {
		context: imgui,
		renderer,
	}))
}

fn main() -> Result<(), Box<dyn Error>> {
	// initialize
	initializeTracing()?;

	let (gl, mut window, glContext) = createSdl3GlContext(WIN_TITLE, WIN_WIDTH, WIN_HEIGHT)?;
	let (gl, mut imgui) = initializeImGui(gl, &window, &glContext)?;

	info!("Starting main loop");
	let mut fps: u64 = 0;
	let mut lastTick: u64 = 0;
	let mut dt: f32 = OPTIMAL_WAIT_TIME as f32 / 1000.0;
	// let mut lastFrame = Instant::now();
	'main: loop {
		let startTick = timer::ticks();
		
		// events
		while let Some(raw) = dear_imgui_sdl3::sdl3_poll_event_ll() {
			let _ = dear_imgui_sdl3::process_sys_event(&raw);

			let event = Event::from_ll(raw);
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
		// let now = Instant::now();
		// let idt = (now - lastFrame).as_secs_f32();
		// lastFrame = now;
		imgui.context.io_mut().set_delta_time(dt);
		
		// Imgui
		dear_imgui_sdl3::sdl3_new_frame(&mut imgui.context);
		let ui = imgui.context.frame();

		ui.dockspace_over_main_viewport();

		ui.window("Main")
			.flags(WindowFlags::ALWAYS_AUTO_RESIZE)
			.build(|| {
				ui.text(format!("ImGUI FPS: {:.2}", ui.io().framerate()));
				// ui.text(format!("ImGUI dt: {}", idt));
				// total frames
				ui.text(format!("Calculted dt: {}", dt));
				ui.separator();

				// let mousePos = if let Some(cursor) = self.input.cursor() {
				// 	cursor
				// } else {
				// 	(0.0, 0.0)
				// };
				// ui.text(format!("Mouse Position: ({:.2},{:.2})", mousePos.0, mousePos.1));

				let windowSize = window.size();
				ui.text(format!("Window Size: ({},{})", windowSize.0, windowSize.1));
				ui.separator();

				// let uiWidth = ui.window_width();
				// let itemWidth = ui.push_item_width(uiWidth * 0.6);
				// ui.color_edit4("Clear Color", &mut clearColor);
				// itemWidth.end();
			});

		let drawData = imgui.context.render();

		// render
		unsafe {
			gl.clear(glow::COLOR_BUFFER_BIT);
			gl.clear_color(0.27, 0.59, 0.27, 1.0);
		}

		imgui.renderer.new_frame()?;
		imgui.renderer.render(drawData)?;

		if IMGUI_VIEWPORTS {
			let ioFlags = imgui.context.io().config_flags();
			if ioFlags.contains(ConfigFlags::VIEWPORTS_ENABLE) {
				imgui.context.update_platform_windows();
				imgui.context.render_platform_windows_default();
				// Restore main GL context
				let _ = window.gl_make_current(&glContext);
			}
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
		dt = waitTime as f32 / 1000.0;
		if waitTime > 0 {
			// info!("{}", waitTime);
			timer::delay(waitTime as u32);
		}
	}
	
	// destroy
	info!("Cleaning up");
	if IMGUI_VIEWPORTS {
		glow_mvp::shutdown_multi_viewport_support(&mut imgui.context);
	}
	dear_imgui_sdl3::shutdown(&mut imgui.context);
	
	Ok(())
}