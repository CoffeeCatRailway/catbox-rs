mod mesh;
mod primitives;
mod builder;

use std::hash::{Hash, Hasher};
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};

pub use mesh::*;
pub use primitives::*;
pub use builder::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
	pub uv: Vec2,
}

impl Vertex {
	pub fn new(position: Vec3, normal: Vec3, color: Vec3, uv: Vec2) -> Self {
		Self {
			position,
			normal,
			color,
			uv,
		}
	}
	
	pub fn withColor(position: Vec3, normal: Vec3, color: Vec3) -> Vertex {
		Self {
			position,
			normal,
			color,
			uv: Vec2::ZERO,
		}
	}
	
	pub fn withUV(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
		Self {
			position,
			normal,
			color: Vec3::ONE,
			uv,
		}
	}
	
	pub fn normalPos(mut self) -> Self {
		self.normal = self.position.normalize_or_zero();
		self
	}
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.x.to_bits().hash(state);
        self.position.y.to_bits().hash(state);
        self.position.z.to_bits().hash(state);
        self.normal.x.to_bits().hash(state);
        self.normal.y.to_bits().hash(state);
        self.normal.z.to_bits().hash(state);
        self.color.x.to_bits().hash(state);
        self.color.y.to_bits().hash(state);
        self.color.z.to_bits().hash(state);
		self.uv.x.to_bits().hash(state);
		self.uv.y.to_bits().hash(state);
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::ZERO,
            color: Vec3::ONE,
			uv: Vec2::ZERO,
        }
    }
}
