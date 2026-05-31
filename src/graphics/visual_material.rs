use std::sync::Arc;
use glam::Vec3;
use crate::graphics::Texture;
use crate::types::{GlRef, ShaderRef, TextureRef};

#[derive(Debug, Clone)]
pub struct VisualMaterial {
	pub shader: ShaderRef,
	// pub modulateColor: Vec3,
	pub diffuse: Option<TextureRef>,
	pub specular: Option<TextureRef>,
	pub shininess: f32,
}

impl VisualMaterial {
	// pub fn diffuse(&self) -> Option<&Texture> {
	// 	match &self.diffuse {
	// 		None => None,
	// 		Some(tex) => Some(&tex),
	// 	}
	// }
	// 
	// pub fn diffuseMut(&mut self) -> Option<&mut Texture> {
	// 	match &mut self.diffuse {
	// 		None => None,
	// 		Some(tex) => Some(Arc::get_mut(tex).unwrap())
	// 	}
	// }
	// 
	// pub fn specular(&self) -> Option<&Texture> {
	// 	match &self.specular {
	// 		None => None,
	// 		Some(tex) => Some(&tex),
	// 	}
	// }
	// 
	// pub fn specularMut(&mut self) -> Option<&mut Texture> {
	// 	match &mut self.specular {
	// 		None => None,
	// 		Some(tex) => Some(Arc::get_mut(tex).unwrap())
	// 	}
	// }
	
	pub fn apply(&self, gl: &GlRef) {
		let shader = self.shader.read().unwrap();
		shader.bind();
		// shader.setUniform3fv("u_modulateColor", &self.modulateColor);
		shader.setUniform1f("u_material.shininess", self.shininess);
		
		shader.setUniform1i("u_material.diffuse", 0);
		let diffuse = match &self.diffuse {
			None => Texture::defaultTexture(gl),
			Some(texture) => texture.clone(),
		};
		diffuse.bind(0);
		
		shader.setUniform1i("u_material.specular", 1);
		let specular = match &self.specular {
			None => Texture::defaultTexture(gl),
			Some(texture) => texture.clone(),
		};
		specular.bind(1);
	}
}
