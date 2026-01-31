#![allow(non_snake_case)]

use std::cell::RefCell;
use crate::graphics::Renderer;
use crate::simulation::camera::{Camera, Direction, Frustum, Projection};
use crate::simulation::{SimpleSolver, Transform, VerletObject};
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3};
use glow::{Context, HasContext};
use log::info;
use std::rc::Rc;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;
use crate::TIME_STEP;

pub trait Viewport {
    fn resize(&mut self, width: u32, height: u32);

    fn handleInput(&mut self, dt: f32, input: &WinitInputHelper, eventLoop: &ActiveEventLoop);

    fn update(&mut self, dt: f32, _eventLoop: &ActiveEventLoop);

    fn render(&mut self, dt: f32);

    fn destroy(&mut self);
}

pub struct ViewportSim {
    window: Rc<Window>,
    gl: Rc<Context>,

    camera: Camera,
    lastMousePos: Vec2,
    projectionMatrix: Mat4,
    viewMatrix: Mat4,
    renderer: Renderer,

    solver: Rc<RefCell<SimpleSolver>>,
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

        let solver = Rc::new(RefCell::new(SimpleSolver::new(vec2(1000.0, 1000.0), 8)));
        renderer.addRenderable(solver.clone());

        // let obj = solver.borrow_mut().addObject(VerletObject {
		// 	elasticity: 0.9,
		// 	..VerletObject::default()
		// });
        // obj.borrow_mut().position.y = solver.borrow().worldSize.y * 0.25;
		// obj.borrow_mut().positionLast.y = solver.borrow().worldSize.y * 0.25;
		// obj.borrow_mut().setVelocity(vec2(100.0, 0.0), TIME_STEP);

		let mut sim = ViewportSim {
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
            let mut solver = self.solver.borrow_mut();
            solver.pause = !solver.pause;
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

    fn update(&mut self, dt: f32, _eventLoop: &ActiveEventLoop) {
        // self.renderer.getShapeRenderer().pushBox(Vec2::ZERO, Vec3::splat(0.15), self.solver.borrow().worldSize, 0.0, 10.0);

        let mut solver = self.solver.borrow_mut();
        if !solver.pause {
            if solver.getTotalSteps() % 2 == 0 && solver.getObjectCount() <= 1000 {
                info!("{}", solver.getObjectCount());
                let t = solver.getObjectCount() as f32 * 0.5;
                let color = vec3(
                    t.sin() * 0.5 + 0.5,
                    t.cos() * 0.5 + 0.5,
                    (t + t.cos()).cos() * 0.5 + 0.5,
                );
                let obj = solver.addObject(VerletObject {
                    color,
                    elasticity: 1.0,
                    ..VerletObject::default()
                });
                obj.borrow_mut().position.y = solver.worldSize.y * 0.25;
                obj.borrow_mut().positionLast.y = solver.worldSize.y * 0.25;
                obj.borrow_mut().setVelocity(vec2(100.0, 50.0), TIME_STEP);
            }
        }

        solver.update(dt);
    }

    fn render(&mut self, dt: f32) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.clear_color(0.27, 0.59, 0.27, 1.0);
        }

        let projection = self.projectionMatrix;
        self.viewMatrix = self.camera.getViewMatrix();

        let pvm = projection * self.viewMatrix;
        self.renderer.render(dt, &pvm);
    }

    fn destroy(&mut self) {
        self.renderer.destroy();
        self.solver.borrow_mut().destroy();
    }
}
