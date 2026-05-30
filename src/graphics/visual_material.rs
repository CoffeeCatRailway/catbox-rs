use std::sync::Arc;
use glam::Vec3;
use crate::graphics::Texture;
use crate::types::{GlRef, ShaderRef, TextureRef};

#[derive(Debug, Clone)]
pub struct VisualMaterial {
	pub shader: ShaderRef,
	pub texture: Option<TextureRef>,
	pub color: Vec3,
}

impl VisualMaterial {
	pub fn new(shader: ShaderRef) -> Self {
		VisualMaterial {
			shader,
			texture: None,
			color: Vec3::ONE,
		}
	}
	
	pub fn texture(&self) -> Option<&Texture> {
		match &self.texture {
			None => None,
			Some(tex) => Some(&tex),
		}
	}
	
	pub fn textureMut(&mut self) -> Option<&mut Texture> {
		match &mut self.texture {
			None => None,
			Some(tex) => Some(Arc::get_mut(tex).unwrap())
		}
	}
	
	pub fn apply(&self, gl: &GlRef) {
		self.shader.read().unwrap().bind();
		match &self.texture {
			None => Texture::defaultTexture(gl).bind(),
			Some(tex) => tex.bind(),
		}
	}
}