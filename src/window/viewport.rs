#![allow(non_snake_case)]

use crate::graphics::{LineRenderer, ShapeRenderer};
use crate::simulation::camera::{Camera, Direction, Frustum, Projection};
use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, vec2, vec3, vec4};
use glow::{Context, HasContext};
use log::info;
use std::rc::Rc;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;
use crate::simulation::Transform;

pub trait Viewport {
    fn resize(&mut self, width: u32, height: u32);

    fn handleInput(&mut self, dt: f64, input: &WinitInputHelper, eventLoop: &ActiveEventLoop);

    fn update(&mut self, dt: f64, _eventLoop: &ActiveEventLoop);

    fn render(&mut self);

    fn destroy(&mut self);
}

pub struct ViewportSim {
    window: Rc<Window>,
    gl: Rc<Context>,

    camera: Camera,
    lastMousePos: Vec2,
    projectionMatrix: Mat4,
    viewMatrix: Mat4,

    lineRenderer: LineRenderer,
    shapeRenderer: ShapeRenderer,
    time: f64,
}

impl ViewportSim {
	pub fn new(window: Rc<Window>, gl: Rc<Context>) -> Self {
		unsafe {
			let size = window.inner_size();
			gl.viewport(0, 0, size.width as i32, size.height as i32);
			info!("Initial viewport: {}/{}", size.width, size.height);
			
			// gl.line_width(10.0);
			// gl.enable(glow::DEPTH_TEST);
			// gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
		}
		
		let camera = Camera {
			frustum: Frustum {
                near: 0.1,
                far: 100.0,
                fov: 10.0,
                fovMin: 1.0,
                fovMax: 10.0
            },
            transform: Transform {
                position: vec3(0.0, 0.0, 5.0),
                ..Transform::default()
            },
			..Camera::default()
		};
		
		let lineRenderer = LineRenderer::new(gl.clone(), 1024).unwrap();
		let shapeRenderer = ShapeRenderer::new(gl.clone(), 1024).unwrap();
		
		let mut sim = ViewportSim {
			window,
			gl,
			
			camera,
			lastMousePos: Vec2::ZERO,
			projectionMatrix: Mat4::IDENTITY,
			viewMatrix: Mat4::IDENTITY,
			
			lineRenderer,
			shapeRenderer,
			time: 0.0,
		};
		sim.updateProjectionMatrix();
		sim
	}
	
	fn updateProjectionMatrix(&mut self) {
		let size = self.window.inner_size();
		let aspect = size.width as f32 / size.height as f32;
		
		let projection = Projection::Orthographic(aspect * -1.0, aspect * 1.0, -1.0, 1.0);
		self.projectionMatrix = self.camera.getProjectionMatrix(projection);
	}
	
	fn cursorToWorldSpace(&mut self, cursor: Vec2) -> Vec3 {
		let size = self.window.inner_size();
		// https://antongerdelan.net/opengl/raycasting.html
		let ndc = vec2(
			(2.0 * cursor.x) / size.width as f32 - 1.0,
			1.0 - (2.0 * cursor.y) / size.height as f32,
		);
		let clip = vec4(ndc.x, ndc.y, -1.0, 1.0);
		let mut eye = self.projectionMatrix.inverse() * clip;
		eye.z = -1.0;
		eye.w = 0.0;
		(self.viewMatrix.inverse() * eye).truncate() //.normalize()
	}
}

impl Viewport for ViewportSim {
    fn resize(&mut self, _width: u32, _height: u32) {
        // TODO: WTF?! Fix stretching on Linux. Tested on Arch Linux Wayland
        #[cfg(not(target_os = "linux"))]
        unsafe {
            let size = self.window.inner_size();
            self.gl.viewport(0, 0, size.width as i32, size.height as i32);
        }
        self.updateProjectionMatrix();
    }

    fn handleInput(&mut self, dt: f64, input: &WinitInputHelper, eventLoop: &ActiveEventLoop) {
        if input.key_pressed(KeyCode::Escape) {
            eventLoop.exit();
        }

        let scrollDiff = {
            let d = input.scroll_diff();
            vec2(d.0, d.1)
        };

        if scrollDiff.y != 0.0 {
            // info!("{}", 1.0 / dt);
            self.camera.frustum.zoom(-scrollDiff.y);
            self.updateProjectionMatrix();
        }

        let speed: f32 = 5.0;
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
            self.camera.walk(Direction::Up, false, speed * dt as f32);
        }
        if input.key_held(KeyCode::KeyS) {
            self.camera.walk(Direction::Down, false, speed * dt as f32);
        }
        if input.key_held(KeyCode::KeyA) {
            self.camera.walk(Direction::Left, false, speed * dt as f32);
        }
        if input.key_held(KeyCode::KeyD) {
            self.camera.walk(Direction::Right, false, speed * dt as f32);
        }
    }

    fn update(&mut self, dt: f64, _eventLoop: &ActiveEventLoop) {
        self.time += dt;

        {
            let norm = |v: Vec2| {
                let n = v.normalize();
                vec3(n.x * 0.5 + 0.5, n.y * 0.5 + 0.5, 0.0)
            };

            let p1 = vec2(-1.0, -1.0);
            let p2 = vec2(1.0, -1.0);
            let p3 = vec2(1.0, 1.0);
            let p4 = vec2(-1.0, 1.0);

            let c1 = norm(p1);
            let c2 = norm(p2);
            let c3 = norm(p3);
            let c4 = norm(p4);

            self.lineRenderer.pushLine2(p1, c1, p2, c2);
            self.lineRenderer.pushLine2(p2, c2, p3, c3);
            self.lineRenderer.pushLine2(p3, c3, p4, c4);
            self.lineRenderer.pushLine2(p4, c4, p1, c1);

            self.lineRenderer.pushLine2(p1, c1, p3, c3);
            self.lineRenderer.pushLine2(p2, c2, p4, c4);

            let cp = self.camera.transform.position.xy();
            self.lineRenderer.pushLine2(Vec2::ZERO, Vec3::ZERO, cp, cp.extend(0.0).normalize());
        }

        self.shapeRenderer.pushCircle(Vec2::ZERO, Vec3::ONE, 1.0, 0.1);
    }

    fn render(&mut self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.clear_color(0.0, 0.1, 0.0, 1.0);
        }

        let projection = self.projectionMatrix;
        self.viewMatrix = self.camera.getViewMatrix();

        let pvm = projection * self.viewMatrix;
        self.shapeRenderer.drawFlush(&pvm);
        self.lineRenderer.drawFlush(&pvm);
    }

    fn destroy(&mut self) {
        self.lineRenderer.destroy();
        self.shapeRenderer.destroy();
    }
}
