#![allow(non_snake_case)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use glam::{vec2, Mat4, Vec2, Vec3};
use crate::graphics::{LineRenderer, Renderable, ShapeRenderer};
use crate::simulation::VerletObject;

pub struct SimpleSolver {
    pub gravity: Vec2,
    pub worldSize: Vec2,

    objects: Vec<Rc<RefCell<VerletObject>>>,

    subSteps: u32,
    totalSteps: u32,

    pub pause: bool,
    btnStep: bool,

    time: f32,
    // frameDt: f32, // Use crate::TIME_STEP
    updateTime: f32
}

impl SimpleSolver {
    pub fn new(worldSize: Vec2, subSteps: u32) -> Self {
        SimpleSolver {
            gravity: vec2(0.0, -400.0),
            worldSize,

            objects: Vec::new(),

            subSteps,
            totalSteps: 0,

            pause: false,
            btnStep: false,

            time: 0.0,
            updateTime: 0.0,
        }
    }

    pub fn addObject(&mut self, object: VerletObject) -> Rc<RefCell<VerletObject>> {
        self.objects.push(Rc::new(RefCell::new(object)));
        self.objects.last_mut().unwrap().clone()
    }

    fn updateObjects(&mut self, dt: f32) {
        for obj in self.objects.iter_mut() {
            let mut obj = obj.borrow_mut();
            obj.accelerate(self.gravity);
            obj.update(dt);
        }
    }

    fn step(&mut self, dt: f32) {
        // sort
        // collide
        // constrain
        self.updateObjects(dt);
    }

    pub fn update(&mut self, dt: f32) {
        if !self.pause || self.btnStep {
            let then = Instant::now();

            self.time += dt;
            let stepDt = dt / self.subSteps as f32;
            for _ in 0..self.subSteps {
                self.step(stepDt);
            }

            let elapsed = then.elapsed().as_secs_f32();
            self.updateTime = elapsed;

            self.totalSteps += 1;
            self.btnStep = false;
        }
    }

    pub fn destroy(&mut self) {

    }
}

impl Renderable for SimpleSolver {
    fn render(&mut self, dt: f32, pvMatrix: &Mat4, shapeRenderer: &mut ShapeRenderer, lineRenderer: &mut LineRenderer) {
        shapeRenderer.pushBox(Vec2::ZERO, Vec3::splat(0.15), self.worldSize, 0.0, 10.0);

        for vobj in self.objects.iter_mut() {
            vobj.borrow_mut().render(dt, pvMatrix, shapeRenderer, lineRenderer);
        }
    }
}
