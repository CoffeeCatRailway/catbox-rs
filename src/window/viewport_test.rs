#![allow(non_snake_case)]
#![allow(unused)]

use std::rc::Rc;
use dear_imgui_rs::Ui;
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3};
use glow::{Context, HasContext};
use log::info;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window};
use winit_input_helper::WinitInputHelper;
use crate::graphics::Renderer;
use crate::simulation::Transform;
use crate::simulation::camera::{Camera, Direction, Projection};
use crate::window::viewport::Viewport;

pub struct ViewportTest {
	window: Rc<Window>,
	gl: Rc<Context>,
	
	camera: Camera,
	mouseCaptured: bool,
	view2D: bool,
	lastMousePos: Vec2,
	projectionMatrix: Mat4,
	viewMatrix: Mat4,

	renderer: Renderer,
	time: f32,
}

impl ViewportTest {
	pub fn new(window: Rc<Window>, gl: Rc<Context>) -> Self {
		unsafe {
			let size = window.inner_size();
			gl.viewport(0, 0, size.width as i32, size.height as i32);
			info!("Initial viewport: {}/{}", size.width, size.height);

			gl.line_width(10.0);
			gl.enable(glow::DEPTH_TEST);
			// gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
		}

		let camera = Camera {
			transform: Transform {
				position: vec3(0.0, 0.0, 5.0),
				..Transform::default()
			},
			..Camera::default()
		};

		let renderer = Renderer::new(gl.clone());

		let mut sim = ViewportTest {
			window,
			gl,

			camera,
			mouseCaptured: false,
			view2D: false,
			lastMousePos: Vec2::ZERO,
			projectionMatrix: Mat4::IDENTITY,
			viewMatrix: Mat4::IDENTITY,

			renderer,
			time: 0.0,
		};
		sim.updateProjectionMatrix();
		sim
	}

	fn updateProjectionMatrix(&mut self) {
		let size = self.window.inner_size();
		let aspect = size.width as f32 / size.height as f32;

		let projection = if self.view2D {
			Projection::Orthographic(aspect * -1.0, aspect * 1.0, -1.0, 1.0)
		} else {
			Projection::Perspective(aspect)
		};

		self.projectionMatrix = self.camera.getProjectionMatrix(projection);
	}

	fn setMouseCaptured(&mut self, mouseCaptured: bool) {
		info!("Mouse captured: {}", mouseCaptured);
		self.mouseCaptured = mouseCaptured;
		if !self.view2D && self.mouseCaptured {
			#[cfg(not(target_os = "linux"))]
			self.window.set_cursor_grab(CursorGrabMode::Confined).expect("Unable to confine mouse!"); // .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Locked))
			#[cfg(target_os = "linux")]
			self.window.set_cursor_grab(CursorGrabMode::Locked).expect("Unable to confine mouse!");
			self.window.set_cursor_visible(false);
		} else {
			self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
			self.window.set_cursor_visible(true);
		}
	}

	fn cursorToWorldSpace(&mut self, cursor: Vec2) -> Vec3 {
		let size = self.window.inner_size();
		// https://antongerdelan.net/opengl/raycasting.html
		let ndc = vec2((2.0 * cursor.x) / size.width as f32 - 1.0, 1.0 - (2.0 * cursor.y) / size.height as f32);
		let clip = vec4(ndc.x, ndc.y, -1.0, 1.0);
		let mut eye = self.projectionMatrix.inverse() * clip;
		eye.z = -1.0;
		eye.w = 0.0;
		(self.viewMatrix.inverse() * eye).truncate()//.normalize()
	}
}

impl Viewport for ViewportTest {
	fn resize(&mut self, width: u32, height: u32) {
		// #[cfg(not(target_os = "linux"))]
		unsafe {
			self.gl.viewport(0, 0, width as i32, height as i32);
		}
		self.updateProjectionMatrix();
	}
	
	fn handleInput(&mut self, dt: f32, input: &WinitInputHelper, eventLoop: &ActiveEventLoop) {
		if input.key_pressed(KeyCode::Escape) {
			eventLoop.exit();
		}
		
		if input.key_pressed(KeyCode::Digit1) {
			self.setMouseCaptured(!self.mouseCaptured);
		}
		if input.key_pressed(KeyCode::Digit2) {
			self.view2D = !self.view2D;
			self.setMouseCaptured(self.mouseCaptured);
			self.updateProjectionMatrix();
		}
		
		if self.mouseCaptured {
			let scrollDiff = {
				let d = input.scroll_diff();
				vec2(d.0, d.1)
			};
			let mouseDiff = {
				let d = input.mouse_diff();
				vec2(d.0, d.1)
			};
			
			if scrollDiff.y != 0.0 {
				// info!("{}", 1.0 / dt);
				self.camera.frustum.zoom(-scrollDiff.y);
				self.updateProjectionMatrix();
			}
			
			let speed: f32 = 5.0;
			if self.view2D {
				if input.mouse_pressed(MouseButton::Middle) {
					if let Some(cursor) = input.cursor() {
						self.lastMousePos = vec2(cursor.0, cursor.1);
					}
				}
				if input.mouse_held(MouseButton::Middle) {
					if let Some(cursor) = input.cursor() {
						let current = vec2(cursor.0, cursor.1);
						// info!("{} {}", current.x, current.y);
						let diff = self.lastMousePos - current;
						if diff.length() > 0.0 {
							// info!("{}", diff);
							
							// info!("{}", self.cursorToWorldSpace(current));
							
							let lastMouseWorldPos = self.cursorToWorldSpace(self.lastMousePos).truncate();
							let diffWorldSpace = self.cursorToWorldSpace(self.lastMousePos + diff).truncate();
							// info!("{}", diffWorldSpace);
							
							let diff = lastMouseWorldPos - diffWorldSpace;
							// info!("{}", diff);
							
							// self.camera.pos.x = -diffWorldSpace.x;
							// self.camera.pos.y = -diffWorldSpace.y;

							self.camera.transform.position.x -= diff.x;
							self.camera.transform.position.y -= diff.y;
						}
						
						self.lastMousePos = current;
					}
				}
				
				if input.key_held(KeyCode::KeyW) {
					self.camera.walk(Direction::Up, false, speed * dt);
				}
				if input.key_held(KeyCode::KeyS) {
					self.camera.walk(Direction::Down, false, speed * dt);
				}
				if input.key_held(KeyCode::KeyA) {
					self.camera.walk(Direction::Left, false, speed * dt);
				}
				if input.key_held(KeyCode::KeyD) {
					self.camera.walk(Direction::Right, false, speed * dt);
				}
			} else {
				if mouseDiff.length() > 0.0 {
					self.camera.turn(mouseDiff.x, -mouseDiff.y, 89.0);
					let size = self.window.inner_size();
					self.window.set_cursor_position(PhysicalPosition::new(size.width / 2, size.height / 2)).expect("Unable to set cursor position!");
				}
				
				if input.key_held(KeyCode::KeyW) {
					self.camera.walk(Direction::Forward, true, speed * dt);
				}
				if input.key_held(KeyCode::KeyS) {
					self.camera.walk(Direction::Backward, true, speed * dt);
				}
				if input.key_held(KeyCode::KeyA) {
					self.camera.walk(Direction::Left, true, speed * dt);
				}
				if input.key_held(KeyCode::KeyD) {
					self.camera.walk(Direction::Right, true, speed * dt);
				}
				if input.key_held(KeyCode::Space) {
					self.camera.walk(Direction::Up, false, speed * dt);
				}
				if input.key_held(KeyCode::ControlLeft) {
					self.camera.walk(Direction::Down, false, speed * dt);
				}
			}
		}
	}
	
	fn update(&mut self, dt: f32, _eventLoop: &ActiveEventLoop) {
		self.time += dt;
		
		let norm = |v: Vec3| { v.normalize() * 0.5 + 0.5 };
		
		let mut t1 = vec3(-1.0, 1.0, -1.0);
		let mut t2 = vec3(1.0, 1.0, -1.0);
		let mut t3 = vec3(1.0, 1.0, 1.0);
		let mut t4 = vec3(-1.0, 1.0, 1.0);
		let mut b1 = vec3(-1.0, -1.0, -1.0);
		let mut b2 = vec3(1.0, -1.0, -1.0);
		let mut b3 = vec3(1.0, -1.0, 1.0);
		let mut b4 = vec3(-1.0, -1.0, 1.0);
		
		let ct1 = norm(t1);
		let ct2 = norm(t2);
		let ct3 = norm(t3);
		let ct4 = norm(t4);
		let cb1 = norm(b1);
		let cb2 = norm(b2);
		let cb3 = norm(b3);
		let cb4 = norm(b4);
		
		self.renderer.getLineRenderer().pushLine3(b1 * 2.0, cb1, b2 * 2.0, cb2);
		self.renderer.getLineRenderer().pushLine3(b2 * 2.0, cb2, b3 * 2.0, cb3);
		self.renderer.getLineRenderer().pushLine3(b3 * 2.0, cb3, b4 * 2.0, cb4);
		self.renderer.getLineRenderer().pushLine3(b4 * 2.0, cb4, b1 * 2.0, cb1);
		
		t1 = t1.rotate_y(self.time);
		t2 = t2.rotate_y(self.time);
		t3 = t3.rotate_y(self.time);
		t4 = t4.rotate_y(self.time);
		b1 = b1.rotate_y(self.time);
		b2 = b2.rotate_y(self.time);
		b3 = b3.rotate_y(self.time);
		b4 = b4.rotate_y(self.time);
		
		self.renderer.getLineRenderer().pushLine3(t1, ct1, t2, ct2);
		self.renderer.getLineRenderer().pushLine3(t2, ct2, t3, ct3);
		self.renderer.getLineRenderer().pushLine3(t3, ct3, t4, ct4);
		self.renderer.getLineRenderer().pushLine3(t4, ct4, t1, ct1);
		
		self.renderer.getLineRenderer().pushLine3(b1, cb1, b2, cb2);
		self.renderer.getLineRenderer().pushLine3(b2, cb2, b3, cb3);
		self.renderer.getLineRenderer().pushLine3(b3, cb3, b4, cb4);
		self.renderer.getLineRenderer().pushLine3(b4, cb4, b1, cb1);
		
		self.renderer.getLineRenderer().pushLine3(t1, ct1, b1, cb1);
		self.renderer.getLineRenderer().pushLine3(t2, ct2, b2, cb2);
		self.renderer.getLineRenderer().pushLine3(t3, ct3, b3, cb3);
		self.renderer.getLineRenderer().pushLine3(t4, ct4, b4, cb4);
		
		self.renderer.getLineRenderer().pushLine3(vec3(0.0, 2.0, 0.0), Vec3::ZERO, vec3(0.0, 2.0, 0.0) + self.camera.transform.rotation.mul_vec3(Vec3::NEG_Z), Vec3::ONE);
		
		self.renderer.getShapeRenderer().pushCircle(Vec2::ZERO, Vec3::ONE, 1.0, 0.1);
	}
	
	fn render(&mut self, dt: f32) {
		// let size = self.window.inner_size();
		// let aspect = size.width as f32 / size.height as f32;
		
		// let projection = Mat4::orthographic_rh(aspect * -1.0, aspect * 1.0, -1.0, 1.0, 0.0, 1.0);
		// let view = Mat4::IDENTITY;
		
		// let projection = if self.view2D {
		// 	Projection::Orthographic(aspect * -1.0, aspect * 1.0, -1.0, 1.0)
		// } else {
		// 	Projection::Perspective(aspect)
		// };
		let projection = self.projectionMatrix;//self.camera.getProjectionMatrix(projection);
		self.viewMatrix = self.camera.getViewMatrix();
		
		let pvm = projection * self.viewMatrix;
		self.renderer.render(dt, &pvm);
	}
	
	fn gui(&mut self, _ui: &mut Ui) {}
	
	fn destroy(&mut self) {
		self.setMouseCaptured(false);
		
		self.renderer.destroy();
	}
}
