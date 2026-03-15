use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use crate::types::ShaderRef;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
	pub position: Vec3,
	pub color: Vec3,
}

impl Default for Vertex {
	fn default() -> Self {
		Self {
			position: Vec3::ZERO,
			color: Vec3::ONE,
		}
	}
}

pub trait Mesh {
	fn isUploaded(&self) -> bool;
	
	fn upload(&mut self, shader: ShaderRef) -> Result<(), String>;
	
	fn draw(&self);
	
	fn destroy(&mut self);
}