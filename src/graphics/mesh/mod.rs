mod mesh;
mod primitives;
mod builder;

use std::hash::{Hash, Hasher};
use bytemuck::{Pod, Zeroable};
use glam::Vec3;

pub use mesh::*;
pub use primitives::*;
pub use builder::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.x.to_bits().hash(state);
        self.position.y.to_bits().hash(state);
        self.position.z.to_bits().hash(state);
        self.color.x.to_bits().hash(state);
        self.color.y.to_bits().hash(state);
        self.color.z.to_bits().hash(state);
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            color: Vec3::ONE,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle(pub Vertex, pub Vertex, pub Vertex);

impl Triangle {
    pub fn subdivide(&self) -> [Triangle; 4] {
        let a = self.0;
        let b = self.1;
        let c = self.2;
        let ab = Vertex {
            position: (a.position + b.position) / 2.0,
            color: (a.color + b.color) / 2.0,
        };
        let ac = Vertex {
            position: (a.position + c.position) / 2.0,
            color: (a.color + c.color) / 2.0,
        };
        let bc = Vertex {
            position: (b.position + c.position) / 2.0,
            color: (b.color + c.color) / 2.0,
        };
        [
            Triangle(a, ac, ab),
            Triangle(ac, c, bc),
            Triangle(ab, bc, b),
            Triangle(ab, ac, bc),
        ]
    }
}

impl Eq for Triangle {}

impl Hash for Triangle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}