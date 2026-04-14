use std::fmt::Debug;
use glam::{Mat4, Vec3};
use tracing::warn;
use crate::graphics::line_renderer::LineRenderer;
use crate::graphics::render_manager::Renderable;
use crate::simulation::aabb::AABB;
use crate::types::{MeshRef, ShaderRef};

pub struct QuadTree<T> {
    capacity: usize,
    values: Vec<T>,
    
    bounds: AABB,
    northWest: Option<Box<QuadTree<T>>>,
    northEast: Option<Box<QuadTree<T>>>,
    southWest: Option<Box<QuadTree<T>>>,
    southEast: Option<Box<QuadTree<T>>>,
}

impl<T> QuadTree<T> {
    pub fn new(capacity: usize, bounds: AABB) -> Self {
        Self {
            capacity,
            values: Vec::new(),
            
            bounds,
            northWest: None,
            northEast: None,
            southWest: None,
            southEast: None,
        }
    }
    
    pub fn clear(&mut self) {
        self.values.clear();
        self.northWest = None;
        self.northEast = None;
        self.southWest = None;
        self.southEast = None;
    }
}

impl<T: Clone + Debug> QuadTree<T> {
    pub fn subdivide<F: Fn(&T, &AABB) -> bool>(&mut self, overlaps: &F) {
        if self.northWest.is_some() {
            return;
        }
        
        let size = self.bounds.size / 2.0;
        self.northWest = Some(Box::new(QuadTree::new(self.capacity, AABB::new(self.bounds.position + Vec3::new(0.0, size.y, 0.0), size))));
        self.northEast = Some(Box::new(QuadTree::new(self.capacity, AABB::new(self.bounds.position + Vec3::new(size.x, size.y, 0.0), size))));
        self.southWest = Some(Box::new(QuadTree::new(self.capacity, AABB::new(self.bounds.position + Vec3::new(0.0, 0.0, 0.0), size))));
        self.southEast = Some(Box::new(QuadTree::new(self.capacity, AABB::new(self.bounds.position + Vec3::new(size.x, 0.0, 0.0), size))));
        
        let mut moved;
        for value in self.values.drain(..) {
            moved = false;
            moved |= self.northWest.as_mut().unwrap().insert(value.clone(), overlaps);
            moved |= self.northEast.as_mut().unwrap().insert(value.clone(), overlaps);
            moved |= self.southWest.as_mut().unwrap().insert(value.clone(), overlaps);
            moved |= self.southEast.as_mut().unwrap().insert(value.clone(), overlaps);
            if !moved {
                warn!("Value {:?} lost when subdividing QuadTree!", value);
            }
        }
    }
    
    pub fn insert<F: Fn(&T, &AABB) -> bool>(&mut self, value: T, overlaps: &F) -> bool {
        if !overlaps(&value, &self.bounds) {
            return false;
        }
        
        if self.values.len() < self.capacity && self.northWest.is_none() {
            self.values.push(value);
            return true;
        }
        
        if self.northWest.is_none() {
            self.subdivide(overlaps);
        }
        
        let mut inserted = false;
        inserted |= self.northWest.as_mut().unwrap().insert(value.clone(), overlaps);
        inserted |= self.northEast.as_mut().unwrap().insert(value.clone(), overlaps);
        inserted |= self.southWest.as_mut().unwrap().insert(value.clone(), overlaps);
        inserted |= self.southEast.as_mut().unwrap().insert(value.clone(), overlaps);
        inserted
    }
    
    pub fn findInArea<F: Fn(&T, &AABB) -> bool>(&self, area: &AABB, overlaps: &F) -> Vec<T> {
        let mut found = Vec::new();
        if !self.bounds.overlaps(area) {
            return found;
        }
        
        for value in self.values.iter() {
            if overlaps(value, area) {
                found.push(value.clone());
            }
        }
        if self.northWest.is_none() {
            return found;
        }
        
        found.append(&mut self.northWest.as_ref().unwrap().findInArea(area, overlaps));
        found.append(&mut self.northEast.as_ref().unwrap().findInArea(area, overlaps));
        found.append(&mut self.southWest.as_ref().unwrap().findInArea(area, overlaps));
        found.append(&mut self.southEast.as_ref().unwrap().findInArea(area, overlaps));
        found
    }
}

impl<T> Renderable for QuadTree<T> {
    fn meshRef(&self) -> Option<&MeshRef> {
        None
    }
    
    fn shaderRef(&self) -> Option<&ShaderRef> {
        None
    }
    
    fn render(&self, _projViewMat: &Mat4, _dt: f32, lineRenderer: &mut LineRenderer) -> Result<(), String> {
        let start = self.bounds.start();
        let end = self.bounds.end();
        
        let topLeft = Vec3::new(start.x, end.y, 0.0);
        let topRight = Vec3::new(end.x, end.y, 0.0);
        let bottomLeft = Vec3::new(start.x, start.y, 0.0);
        let bottomRight = Vec3::new(end.x, start.y, 0.0);
        
        lineRenderer.pushLine3(topLeft, Vec3::ONE, topRight, Vec3::ONE);
        lineRenderer.pushLine3(topRight, Vec3::ONE, bottomRight, Vec3::ONE);
        lineRenderer.pushLine3(bottomRight, Vec3::ONE, bottomLeft, Vec3::ONE);
        lineRenderer.pushLine3(bottomLeft, Vec3::ONE, topLeft, Vec3::ONE);
        
        if self.northWest.is_none() {
            return Ok(());
        }
        
        self.northWest.as_ref().unwrap().render(_projViewMat, _dt, lineRenderer)?;
        self.northEast.as_ref().unwrap().render(_projViewMat, _dt, lineRenderer)?;
        self.southWest.as_ref().unwrap().render(_projViewMat, _dt, lineRenderer)?;
        self.southEast.as_ref().unwrap().render(_projViewMat, _dt, lineRenderer)?;
        
        Ok(())
    }
}
