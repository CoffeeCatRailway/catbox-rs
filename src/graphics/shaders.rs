use std::sync::OnceLock;
use crate::graphics::shader::{Shader, ShaderType};
use crate::types::{newShaderRef, GlRef, ShaderRef};

pub const SIMPLE_VERTEX: &str = include_str!("../../resources/shaders/simple.vert");
pub const SIMPLE_FRAGMENT: &str = include_str!("../../resources/shaders/simple.frag");
static SIMPLE_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const SIMPLE_LIGHT_VERTEX: &str = include_str!("../../resources/shaders/simple_light.vert");
pub const SIMPLE_LIGHT_FRAGMENT: &str = include_str!("../../resources/shaders/simple_light.frag");
static SIMPLE_LIGHT_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const INSTANCE_VERTEX: &str = include_str!("../../resources/shaders/instance.vert");
pub const INSTANCE_FRAGMENT: &str = include_str!("../../resources/shaders/instance.frag");
static INSTANCE_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub fn simpleShader(gl: GlRef) -> Result<ShaderRef, String> {
	if SIMPLE_SHADER_REF.get().is_none() {
		let shader = Shader::new(gl)?
			.attachFromSource(ShaderType::Vertex, SIMPLE_VERTEX)?
			.attachFromSource(ShaderType::Fragment, SIMPLE_FRAGMENT)?
			.link()?;
		SIMPLE_SHADER_REF.set(newShaderRef(shader)).expect("Failed to set simple shader reference!");
	}
	Ok(SIMPLE_SHADER_REF.get().unwrap().clone())
}

pub fn simpleLightShader(gl: GlRef) -> Result<ShaderRef, String> {
	if SIMPLE_LIGHT_SHADER_REF.get().is_none() {
		let shader = Shader::new(gl)?
			.attachFromSource(ShaderType::Vertex, SIMPLE_LIGHT_VERTEX)?
			.attachFromSource(ShaderType::Fragment, SIMPLE_LIGHT_FRAGMENT)?
			.link()?;
		SIMPLE_LIGHT_SHADER_REF.set(newShaderRef(shader)).expect("Failed to set simple light shader reference!");
	}
	Ok(SIMPLE_LIGHT_SHADER_REF.get().unwrap().clone())
}

pub fn instanceShader(gl: GlRef) -> Result<ShaderRef, String> {
	if INSTANCE_SHADER_REF.get().is_none() {
		let shader = Shader::new(gl)?
			.attachFromSource(ShaderType::Vertex, INSTANCE_VERTEX)?
			.attachFromSource(ShaderType::Fragment, INSTANCE_FRAGMENT)?
			.link()?;
		INSTANCE_SHADER_REF.set(newShaderRef(shader)).expect("Failed to set instance shader reference!");
	}
	Ok(INSTANCE_SHADER_REF.get().unwrap().clone())
}

fn destroyShaderRef(shader: &OnceLock<ShaderRef>) {
	if let Some(shader) = shader.get() {
		shader.write().unwrap().destroy();
	}
}

pub fn destroyAllShaders() {
	destroyShaderRef(&SIMPLE_SHADER_REF);
	destroyShaderRef(&SIMPLE_LIGHT_SHADER_REF);
	destroyShaderRef(&INSTANCE_SHADER_REF);
}