#![allow(non_snake_case)]

use std::rc::Rc;
use glam::Mat4;
use glow::Context;
use log::info;
use crate::graphics::{LineRenderer, ShapeRenderer};

pub struct Renderer {
    renderables: Vec<Box<dyn Renderable>>,
    destroyed: bool,

    shapeRenderer: ShapeRenderer,
    lineRenderer: LineRenderer,
}

impl Renderer {
    pub fn new(gl: Rc<Context>) -> Self {
        let shapeRenderer = ShapeRenderer::new(gl.clone(), 1024).expect("Failed to create shape renderer");
        let lineRenderer = LineRenderer::new(gl.clone(), 1024).expect("Failed to create line renderer");

        Renderer {
            destroyed: false,

            shapeRenderer,
            lineRenderer,
        }
    }

    pub fn getShapeRenderer(&mut self) -> &mut ShapeRenderer {
        &mut self.shapeRenderer
    }

    pub fn getLineRenderer(&mut self) -> &mut LineRenderer {
        &mut self.lineRenderer
    }

    pub fn render(&mut self, dt: f32, pvMatrix: &Mat4) {
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
