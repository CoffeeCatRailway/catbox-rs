use std::sync::Arc;
use glam::Vec3;
use crate::graphics::texture::Texture;
use crate::types::{GlRef, ShaderRef, TextureRef};

#[derive(Debug, Clone)]
pub struct Material {
	pub shader: ShaderRef,
	pub texture: Option<TextureRef>,
	pub color: Vec3,
}

impl Material {
	pub fn new(shader: ShaderRef) -> Self {
		Material {
			shader,
			texture: None,
			color: Vec3::ONE,
		}
	}
	
	pub fn shader(&self) -> &ShaderRef {
		&self.shader
	}
	
	// pub fn shaderMut(&mut self) -> &mut Shader {
	// 	Arc::get_mut(&mut self.shader).unwrap()
	// }
	
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