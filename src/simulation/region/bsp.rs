use std::fmt::Debug;
use glam::{Mat4, Vec3};
use tracing::warn;
use crate::graphics::{LineRenderer, Renderable};
use crate::simulation::region::AABB;
use crate::types::{MeshRef, ShaderRef};

#[derive(Copy, Clone, Debug)]
pub enum Orientation {
	Vertical,
	Horizontal,
}

impl Orientation {
	pub fn next(&self) -> Orientation {
		match self {
			Orientation::Vertical => Orientation::Horizontal,
			Orientation::Horizontal => Orientation::Vertical,
		}
	}
}

pub struct BSPGrid<T> {
	capacity: usize,
	values: Vec<T>,
	
	bounds: AABB,
	orientation: Orientation,
	left: Option<Box<BSPGrid<T>>>,
	right: Option<Box<BSPGrid<T>>>,
}

impl<T> BSPGrid<T> {
	pub fn new(capacity: usize, bounds: AABB) -> BSPGrid<T> {
		Self::withOrientation(capacity, bounds, Orientation::Vertical)
	}
	
	pub fn withOrientation(capacity: usize, bounds: AABB, orientation: Orientation) -> BSPGrid<T> {
		Self {
			capacity,
			values: Vec::new(),
			
			bounds,
			orientation,
			left: None,
			right: None,
		}
	}
	
	pub fn clear(&mut self) {
		self.values.clear();
		self.left = None;
		self.right = None;
	}
	
	pub fn depth(&self) -> usize {
		let left = match self.left {
			None => 0,
			Some(ref leaf) => leaf.depth(),
		};
		let right = match self.right {
			None => 0,
			Some(ref leaf) => leaf.depth(),
		};
		1 + left.max(right)
	}
}

impl<T: Clone + Debug> BSPGrid<T> {
	fn split<F: Fn(&T, &AABB) -> bool>(&mut self, overlaps: &F) {
		if self.left.is_some() {
			return;
		}
		
		let size = match self.orientation {
			Orientation::Vertical => self.bounds.size / Vec3::new(1.0, 2.0, 1.0),
			Orientation::Horizontal => self.bounds.size / Vec3::new(2.0, 1.0, 1.0),
		};
		let orientation = self.orientation.next();
		
		let offset = match self.orientation {
			Orientation::Vertical => Vec3::new(0.0, 0.0, 0.0),
			Orientation::Horizontal => Vec3::new(0.0, 0.0, 0.0),
		};
		self.left = Some(Box::new(BSPGrid::withOrientation(self.capacity, AABB::new(self.bounds.position + offset, size), orientation)));
		
		let offset = match self.orientation {
			Orientation::Vertical => Vec3::new(0.0, size.y, 0.0),
			Orientation::Horizontal => Vec3::new(size.x, 0.0, 0.0),
		};
		self.right = Some(Box::new(BSPGrid::withOrientation(self.capacity, AABB::new(self.bounds.position + offset, size), orientation)));
		
		let mut moved;
		for value in self.values.drain(..) {
			moved = false;
			moved |= self.left.as_mut().unwrap().insert(value.clone(), overlaps);
			moved |= self.right.as_mut().unwrap().insert(value.clone(), overlaps);
			if !moved {
				warn!("Value {:?} lost when spliting BSPGrid!", value);
			}
		}
	}
	
	pub fn insert<F: Fn(&T, &AABB) -> bool>(&mut self, value: T, overlaps: &F) -> bool {
		if !overlaps(&value, &self.bounds) {
			return false;
		}
		
		if self.values.len() < self.capacity && self.left.is_none() {
			self.values.push(value);
			return true;
		}
		
		if self.left.is_none() {
			self.split(overlaps);
		}
		
		let mut inserted = false;
		inserted |= self.left.as_mut().unwrap().insert(value.clone(), overlaps);
		inserted |= self.right.as_mut().unwrap().insert(value.clone(), overlaps);
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
		if self.left.is_none() {
			return found;
		}
		
		found.append(&mut self.left.as_ref().unwrap().findInArea(area, overlaps));
		found.append(&mut self.right.as_ref().unwrap().findInArea(area, overlaps));
		found
	}
}

impl<T> Renderable for BSPGrid<T> {
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
		
		if self.left.is_none() {
			return Ok(());
		}
		
		self.left.as_ref().unwrap().render(_projViewMat, _dt, lineRenderer)?;
		self.right.as_ref().unwrap().render(_projViewMat, _dt, lineRenderer)?;
		
		Ok(())
	}
}
