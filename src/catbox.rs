use std::error::Error;
use bool_flags::Flags8;
use dear_imgui_glow::GlowRenderer;
#[cfg(feature = "multi-viewport")]
use dear_imgui_glow::multi_viewport as glow_mvp;
use dear_imgui_rs::{
	ConfigFlags,
	Context as ImguiContext,
	WindowFlags
};
use glam::{Mat4, Vec2, Vec3};
use glow::HasContext;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::timer;
use sdl3::video::{GLContext, GLProfile, SwapInterval};
use tracing::{info, warn};
use crate::graphics::line_renderer::LineRenderer;
use crate::types::{newLineRendererRef, newSdlWindowRef, GlRef, LineRendererRef, SdlWindowRef};

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
	
	gl: GlRef,
	#[allow(unused)]
	glContext: GLContext,
	window: SdlWindowRef,
	
	imgui: Imgui,
	
	lineRenderer: LineRendererRef,
	clearColor: [f32; 4],
	mousePos: Vec2,
}

impl CatBox {
	pub fn new() -> Result<CatBox, Box<dyn Error>> {
		info!("Creating CatBox");
		let flags = Flags8::none();
	
		info!("SDL3 context");
		let sdl = sdl3::init()?;
		let video = sdl.video()?;
		let glAttributes = video.gl_attr();
		
		glAttributes.set_context_profile(GLProfile::Core);
		glAttributes.set_context_version(4, 3);
		glAttributes.set_depth_size(0);
		
		info!("Window and GL context");
		let window = video.window(WIN_TITLE, WIN_WIDTH, WIN_HEIGHT)
						  .opengl()
						  .resizable()
						  .position_centered()
						  .build()?;
		let window = newSdlWindowRef(window);
		
		let glContext = window.borrow().gl_create_context()?;
		
		window.borrow().gl_make_current(&glContext)?;
		let _ = video.gl_set_swap_interval(SwapInterval::Immediate);
		
		let gl = unsafe {
			use std::ffi::c_void;
			glow::Context::from_loader_function(|name| {
				video.gl_get_proc_address(name).map(|f| f as *const c_void).unwrap_or(std::ptr::null())
			})
		};
		
		info!("Imgui context");
		let mut imgui = ImguiContext::create();
		{
			let io = imgui.io_mut();
			let mut flags= io.config_flags();
			flags.insert(ConfigFlags::DOCKING_ENABLE);
			#[cfg(feature = "multi-viewport")]
			flags.insert(ConfigFlags::VIEWPORTS_ENABLE);
			io.set_config_flags(flags);
		}
		
		// Initial SDL3 platform backend
		dear_imgui_sdl3::init_platform_for_opengl(&mut imgui, &window.borrow(), &glContext)?;
		
		// Basic style scaling
		let windowScale = window.borrow().display_scale();
		{
			let style = imgui.style_mut();
			style.set_font_scale_dpi(windowScale);
		}
		
		info!("Imgui glow renderer");
		#[allow(unused_mut)]
		let mut renderer = GlowRenderer::new(gl, &mut imgui)?;
		#[cfg(feature = "multi-viewport")]
		glow_mvp::enable(&mut renderer, &mut imgui);
		
		let gl = renderer.gl_context().unwrap().clone();
		
		info!("Initializing locals");
		let mut lineRenderer = LineRenderer::new(gl.clone(), 1024)?;
		lineRenderer.enable();
	
		Ok(CatBox {
			width: WIN_WIDTH,
			height: WIN_HEIGHT,
			flags,
			
			gl,
			glContext,
			window,
			
			imgui: Imgui {
				context: imgui,
				renderer,
			},
			
			lineRenderer: newLineRendererRef(lineRenderer),
			clearColor: [0.27, 0.59, 0.27, 1.0],
			mousePos: Vec2::ZERO,
		})
	}
	
	fn handleEvents(&mut self, event: Event) {
		match event {
			Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
				warn!("CatBox loop exit requested");
				self.flags.clear(F_RUNNING);
			},
			Event::Window { win_event, .. } => match win_event {
				WindowEvent::Resized(newWidth, newHeight) => unsafe {
					self.gl.viewport(0, 0, newWidth.max(1), newHeight.max(1));
					self.width = newWidth as u32;
					self.height = newHeight as u32;
				},
				_ => {},
			},
			Event::MouseMotion { x, y, .. } => {
				self.mousePos.x = x;
				self.mousePos.y = y;
			},
			_ => {},
		}
	}
	
	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
		info!("Starting CatBox loop");
		self.flags.set(F_RUNNING);
		
		let mut fps: u64 = 0;
		let mut lastTick: u64 = 0;
		let mut dt: f32 = OPTIMAL_DT;
		while self.flags.get(F_RUNNING) {
			let startTick = timer::ticks();
			
			// events
			while let Some(raw) = dear_imgui_sdl3::sdl3_poll_event_ll() {
				let _ = dear_imgui_sdl3::process_sys_event(&raw);
				
				let event = Event::from_ll(raw);
				self.handleEvents(event);
			}
			
			// update
			self.imgui.context.io_mut().set_delta_time(dt);
			
			// Imgui
			dear_imgui_sdl3::sdl3_new_frame(&mut self.imgui.context);
			let ui = self.imgui.context.frame();
			
			ui.dockspace_over_main_viewport();
			
			ui.window("App Info")
			  .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
			  .build(|| {
				  ui.text(format!("ImGUI FPS: {:.3}", ui.io().framerate()));
				  // total frames
				  ui.text(format!("Delta Time: {}", dt));
				  ui.separator();
				  
				  ui.text(format!("Mouse Position: ({:.2},{:.2})", self.mousePos.x, self.mousePos.y));
				  
				  let windowSize = self.window.borrow().size();
				  ui.text(format!("Window Size: ({},{})", windowSize.0, windowSize.1));
				  ui.separator();
				  
				  let uiWidth = ui.window_width();
				  let itemWidth = ui.push_item_width(uiWidth * 0.6);
				  ui.color_edit4("Clear Color", &mut self.clearColor);
				  itemWidth.end();
			  });
			
			let drawData = self.imgui.context.render();
			
			// render
			unsafe {
				self.gl.clear(glow::COLOR_BUFFER_BIT);
				self.gl.clear_color(self.clearColor[0], self.clearColor[1], self.clearColor[2], self.clearColor[3]);
			}
			
			self.lineRenderer.borrow_mut().pushLine2(Vec2::new(0.0, 0.0), Vec3::splat(0.0), Vec2::new(1.0, 1.0), Vec3::splat(1.0));
			
			self.lineRenderer.borrow_mut().drawFlush(&Mat4::IDENTITY);
			
			self.imgui.renderer.new_frame()?;
			self.imgui.renderer.render(drawData)?;
			
			#[cfg(feature = "multi-viewport")]
			{
				let ioFlags = self.imgui.context.io().config_flags();
				if ioFlags.contains(ConfigFlags::VIEWPORTS_ENABLE) {
					self.imgui.context.update_platform_windows();
					self.imgui.context.render_platform_windows_default();
					// Restore main GL context
					let _ = self.window.gl_make_current(&self.glContext);
				}
			}
			
			self.window.borrow().gl_swap_window();
			
			// fps counter
			fps += 1;
			if startTick >= lastTick + 1000 {
				let newTitle = format!("{} - FPS: {}", WIN_TITLE, fps);
				self.window.borrow_mut().set_title(&newTitle)?;
				
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
		info!("Destroying window");
		self.lineRenderer.borrow_mut().destroy();
		#[cfg(feature = "multi-viewport")]
		glow_mvp::shutdown_multi_viewport_support(&mut self.imgui.context);
		dear_imgui_sdl3::shutdown(&mut self.imgui.context);
	}
}
