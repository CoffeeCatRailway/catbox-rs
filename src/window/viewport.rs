#![allow(non_snake_case)]

use std::rc::Rc;
use crate::graphics::Renderer;
use crate::simulation::camera::{Camera, Direction, Frustum, Projection};
use crate::simulation::{SimpleSolver, Transform, VerletObject};
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3};
use glow::{Context, HasContext};
use log::info;
use std::sync::{Arc, Mutex};
use dear_imgui_rs::Ui;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;

pub struct Viewport {
    window: Arc<Window>,
    gl: Rc<Context>,

    camera: Camera,
    lastMousePos: Vec2,
    projectionMatrix: Mat4,
    viewMatrix: Mat4,
    renderer: Renderer,
	
	solver: Arc<Mutex<SimpleSolver>>,
}

impl Viewport {
	pub fn new(window: Arc<Window>, gl: Rc<Context>, solver: Arc<Mutex<SimpleSolver>>) -> Self {
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
                fov: 500.0,
                fovMin: 1.0,
                fovMax: 1000.0
            },
            transform: Transform {
                position: vec3(0.0, 0.0, 5.0),
                ..Transform::default()
            },
			..Camera::default()
		};
        let mut renderer = Renderer::new(gl.clone());
        renderer.getLineRenderer().enabled = false;
		
		if let Ok(mut solver) = solver.lock() {
			solver.worldSize = Vec2::splat(1000.0);
			solver.subSteps = 8;
			solver.gravity.y = -400.0;
		}
		renderer.addRenderable(solver.clone());

		let mut sim = Viewport {
			window,
			gl,
			
			camera,
			lastMousePos: Vec2::ZERO,
			projectionMatrix: Mat4::IDENTITY,
			viewMatrix: Mat4::IDENTITY,
            renderer,
			
			solver,
		};
		sim.updateProjectionMatrix();
		info!("Viewport initialized");
		sim
	}
	
	fn updateProjectionMatrix(&mut self) {
		let windowSize = self.window.inner_size();
		let windowAspect = windowSize.width as f32 / windowSize.height as f32;
		
		// Doesn't work if worldSize is not square
		// let solverSize = self.solver.borrow().worldSize;
		// let solverAspect = solverSize.x / solverSize.y;
		//
		// let projection = if windowAspect >= solverAspect {
		// 	let aspect = windowAspect / solverAspect;
		// 	Projection::Orthographic(aspect * -1.0, aspect * 1.0, -1.0, 1.0)
		// } else {
		// 	let aspect = solverAspect / windowAspect;
		// 	Projection::Orthographic(-1.0, 1.0, aspect * -1.0, aspect * 1.0)
		// };
		
		let projection = Projection::Orthographic(windowAspect * -1.0, windowAspect * 1.0, -1.0, 1.0);
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
	
    pub fn resize(&mut self, width: u32, height: u32) {
        // #[cfg(not(target_os = "linux"))]
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
        }
        self.updateProjectionMatrix();
    }

    pub fn handleInput(&mut self, dt: f32, input: &WinitInputHelper, eventLoop: &ActiveEventLoop) {
        if input.key_pressed(KeyCode::Escape) {
            eventLoop.exit();
        }
        if input.key_pressed(KeyCode::Digit1) {
            if let Ok(mut solver) = self.solver.lock() {
				solver.pause = !solver.pause;
			}
		}

        let scrollDiff = {
            let d = input.scroll_diff();
            vec2(d.0, d.1)
        };

        if scrollDiff.y != 0.0 {
            // info!("{}", 1.0 / dt);
            self.camera.frustum.zoom(-scrollDiff.y * 10.0);
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
    }

    pub fn update(&mut self, dt: f32, _eventLoop: &ActiveEventLoop) {
        // self.renderer.getShapeRenderer().pushBox(Vec2::ZERO, Vec3::splat(0.15), self.solver.borrow().worldSize, 0.0, 10.0);
		
		if let Ok(mut solver) = self.solver.lock() {
			if !solver.pause {
				if solver.getTotalSteps() % 2 == 0 && solver.getObjectCount() <= 2000 {
					// info!("{}", solver.getObjectCount());
					let t = solver.getObjectCount() as f32 * 0.5;
					let color = vec3(
						t.sin() * 0.5 + 0.5,
						t.cos() * 0.5 + 0.5,
						(t + t.cos()).cos() * 0.5 + 0.5,
					);
					let mut obj = VerletObject {
						color,
						elasticity: 1.0,
						..VerletObject::default()
					};
					obj.position.y = solver.worldSize.y * 0.25;
					obj.positionLast.y = solver.worldSize.y * 0.25;
					obj.setVelocity(vec2(100.0, 50.0), dt);
					// obj.setVelocity(vec2(100.0 * (t * 0.5).cos(), 100.0 * (t * 0.5).sin()), dt);
					solver.addObject(Arc::new(Mutex::new(obj)));
				}
			}
			
			// solver.update(dt);
		}
	}
	
	pub fn render(&mut self, dt: f32) {
		let projection = self.projectionMatrix;
		self.viewMatrix = self.camera.getViewMatrix();
		
		let pvm = projection * self.viewMatrix;
        self.renderer.render(dt, &pvm);
    }
	
	pub fn gui(&mut self, _ui: &mut Ui) {
	
	}

    pub fn destroy(&mut self) {
        self.renderer.destroy();
		if let Ok(mut solver) = self.solver.lock() {
			solver.destroy();
		}
	}
}
