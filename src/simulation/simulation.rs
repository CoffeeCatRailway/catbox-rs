#![allow(non_snake_case)]

use std::rc::Rc;
use glam::{vec2, vec3, Mat4, Vec2, Vec3};
use glow::{Context, HasContext};
use log::info;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;
use crate::graphics::LineRenderer;

pub struct Simulation {
	window: Rc<Window>,
	gl: Rc<Context>,
	// camera: Camera,
	lineRenderer: LineRenderer,
}

impl Simulation {
	pub fn new(window: Rc<Window>, gl: Rc<Context>) -> Self {
		unsafe {
			let size = window.inner_size();
			gl.viewport(0, 0, size.width as i32, size.height as i32);
			info!("Initial viewport: {}/{}", size.width, size.height);
			
			gl.line_width(10.0);
			// gl.enable(glow::DEPTH_TEST);
			// gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
		}
		
		// let camera = Camera {
		// 	pos: Vec3::new(0.0, 0.0, 5.0),
		// 	..Camera::default()
		// };
		
		let lineRenderer = LineRenderer::new(gl.clone(), 1024).unwrap();
		
		Simulation {
			window,
			gl,
			// camera,
			lineRenderer,
		}
	}
	
	pub fn resize(&mut self, _width: u32, _height: u32) {
		// TODO: Fix stretching on Linux. Test on Arch Linux Wayland
		#[cfg(not(target_os = "linux"))]
		unsafe {
			// info!("{} {}", width, height);
			// self.gl.viewport(0, 0, width as i32, height as i32);
			// let scaleFactor = self.window.scale_factor();
			let size = self.window.inner_size();
			// info!("{} {}", size.width, size.height);
			self.gl.viewport(0, 0, size.width as i32, size.height as i32);
		}
	}
	
	pub fn update(&mut self, _dt: f64, input: &WinitInputHelper, eventLoop: &ActiveEventLoop) {
		if input.key_pressed(KeyCode::Escape) {
			eventLoop.exit();
		}
		
		let p1 = vec2(-1.0, -1.0);
		let p2 = vec2(1.0, -1.0);
		let p3 = vec2(1.0, 1.0);
		let p4 = vec2(-1.0, 1.0);
		
		let c1 = v2normv3(p1);
		let c2 = v2normv3(p2);
		let c3 = v2normv3(p3);
		let c4 = v2normv3(p4);
		
		self.lineRenderer.pushLine2(p1, c1, p2, c2);
		self.lineRenderer.pushLine2(p2, c2, p3, c3);
		self.lineRenderer.pushLine2(p3, c3, p4, c4);
		self.lineRenderer.pushLine2(p4, c4, p1, c1);
	}
	
	pub fn render(&mut self) {
		unsafe {
			self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
			self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
		}
		
		let size = self.window.inner_size();
		let aspect = size.width as f32 / size.height as f32;
		let projection = Mat4::orthographic_rh(aspect * -1.0, aspect * 1.0, -1.0, 1.0, 0.0, 1.0);//Mat4::perspective_rh(self.camera.fov.to_radians(), aspect, 0.1, 100.0);
		// let view = self.camera.getViewMatrix();
		let pvm = projection;// * view;
		self.lineRenderer.drawFlush(&pvm);
	}
	
	pub fn destroy(&mut self) {
		self.lineRenderer.destroy();
	}
}

fn v2normv3(v: Vec2) -> Vec3 {
	let n = v.normalize();
	vec3(n.x * 0.5 + 0.5, 0.0, n.y * 0.5 + 0.5)
}
