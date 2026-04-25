#![allow(unused)]

use std::fmt::Debug;
use glam::{Mat4, Vec3};
use tracing::warn;
use crate::graphics::{LineRenderer, Renderable};
use crate::simulation::region::AABB;
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
	
	pub fn depth(&self) -> usize {
		let northWest = match self.northWest {
			None => 0,
			Some(ref leaf) => leaf.depth(),
		};
		let northEast = match self.northEast {
			None => 0,
			Some(ref leaf) => leaf.depth(),
		};
		let southWest = match self.southWest {
			None => 0,
			Some(ref leaf) => leaf.depth(),
		};
		let southEast = match self.southEast {
			None => 0,
			Some(ref leaf) => leaf.depth(),
		};
		1 + northWest.max(northEast).max(southWest).max(southEast)
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
		if !lineRenderer.isEnabled() {
			return Ok(())
		}
		
		let percent = self.values.len() as f32 / self.capacity as f32;
		let color = Vec3::new(percent, 1.0 - percent, 0.0);
		
		lineRenderer.pushAABB(&self.bounds, color);
        
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
