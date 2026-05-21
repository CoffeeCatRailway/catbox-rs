use std::rc::Rc;
use glam::Vec3;
use crate::graphics::texture::Texture;
use crate::types::{newTextureRef, GlRef, ShaderRef, TextureRef};

#[derive(Debug, Clone)]
pub struct Material {
	pub shader: ShaderRef,
	pub texture: Option<TextureRef>,
	pub color: Vec3,
	defaultTexture: TextureRef,
}

impl Material {
	pub fn new(gl: GlRef, shader: ShaderRef) -> Result<Self, String> {
		let defaultTexture = newTextureRef(Texture::defaultTexture(gl)?); // todo: Find way to make global
		Ok(Material {
			shader,
			texture: None,
			color: Vec3::ONE,
			defaultTexture,
		})
	}
	
	pub fn shader(&self) -> &ShaderRef {
		&self.shader
	}
	
	// pub fn shaderMut(&mut self) -> &mut Shader {
	//
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
			Some(tex) => Some(Rc::get_mut(tex).unwrap())
		}
	}
	
	pub fn apply(&self) {
		self.shader.read().unwrap().bind();
		
		match &self.texture {
			None => self.defaultTexture.bind(),
			Some(tex) => tex.bind(),
		}
	}
}