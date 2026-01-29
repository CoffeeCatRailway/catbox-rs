#![allow(non_snake_case)]

use std::cell::RefCell;
use std::rc::Rc;
use glam::Mat4;
use glow::Context;
use log::info;
use crate::graphics::{LineRenderer, ShapeRenderer};

pub trait Renderable {
    fn render(&mut self, dt: f32, pvMatrix: &Mat4, shapeRenderer: &mut ShapeRenderer, lineRenderer: &mut LineRenderer);
}

pub struct Renderer {
    renderables: Vec<Rc<RefCell<dyn Renderable>>>,
    destroyed: bool,

    shapeRenderer: ShapeRenderer,
    lineRenderer: LineRenderer,
}

impl Renderer {
    pub fn new(gl: Rc<Context>) -> Self {
        let shapeRenderer = ShapeRenderer::new(gl.clone(), 1024).expect("Failed to create shape renderer");
        let lineRenderer = LineRenderer::new(gl.clone(), 1024).expect("Failed to create line renderer");

        Renderer {
            renderables: Vec::new(),
            destroyed: false,

            shapeRenderer,
            lineRenderer,
        }
    }

    pub fn addRenderable<T: Renderable + 'static>(&mut self, renderable: Rc<RefCell<T>>) {
        self.renderables.push(renderable);
    }

    #[allow(unused)]
    pub fn removeRenderableByIndex(&mut self, i: usize) -> Rc<RefCell<dyn Renderable>> {
        self.renderables.remove(i)
    }

    pub fn getShapeRenderer(&mut self) -> &mut ShapeRenderer {
        &mut self.shapeRenderer
    }

    pub fn getLineRenderer(&mut self) -> &mut LineRenderer {
        &mut self.lineRenderer
    }

    pub fn render(&mut self, dt: f32, pvMatrix: &Mat4) {
        for renderable in self.renderables.iter_mut() {
            renderable.borrow_mut().render(dt, pvMatrix, &mut self.shapeRenderer, &mut self.lineRenderer);
        }

        self.shapeRenderer.drawFlush(pvMatrix);
        self.lineRenderer.drawFlush(pvMatrix)
    }

    pub fn destroy(&mut self) {
        if self.destroyed {
            return;
        }
        info!("Destroying renderer");
        self.shapeRenderer.destroy();
        self.lineRenderer.destroy();
        self.destroyed = true;
    }
}
