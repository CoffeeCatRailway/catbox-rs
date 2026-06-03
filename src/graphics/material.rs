use glam::Vec3;
use crate::graphics::Texture;
use crate::types::{GlRef, ShaderRef, TextureRef};

#[derive(Debug, Clone)]
pub struct Material {
	pub shader: ShaderRef,
	pub color: Vec3,
	pub diffuse: Option<TextureRef>,
	pub specular: Vec3,
	pub shininess: f32,
}

impl Material {
	pub fn apply(&self, gl: &GlRef) {
		let shader = self.shader.read().unwrap();
		shader.bind();
		
		shader.setUniform3fv("u_material.color", &self.color);
		shader.setUniform1i("u_material.diffuse", 0);
		let diffuse = match &self.diffuse {
			None => Texture::default1x1White(gl),
			Some(texture) => texture.clone(),
		};
		diffuse.bind(0);
		
		shader.setUniform4fv("u_material.specular", &(self.specular.extend(self.shininess))); // w is shininess
	}
}
