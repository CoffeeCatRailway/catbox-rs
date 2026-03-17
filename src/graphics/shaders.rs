use std::error::Error;
use std::ptr::null;
use std::sync::OnceLock;
use crate::graphics::shader::{Shader, ShaderType};
use crate::graphics::shaders;
use crate::types::{newShaderRef, GlRef, ShaderRef};

pub const BASE_VERTEX: &str = include_str!("../../resources/shaders/base.vert");
pub const BASE_FRAGMENT: &str = include_str!("../../resources/shaders/base.frag");

pub const INSTANCE_VERTEX: &str = include_str!("../../resources/shaders/instance.vert");
pub const INSTANCE_FRAGMENT: &str = include_str!("../../resources/shaders/instance.frag");

// static BASE_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();
//
// static INSTANCE_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

// todo: check dear-imgui-glow bug report
pub fn baseShader(gl: GlRef) -> Result<ShaderRef, String> {
	let shader = Shader::new(gl)?
		.attachFromSource(ShaderType::Vertex, BASE_VERTEX)?
		.attachFromSource(ShaderType::Fragment, BASE_FRAGMENT)?
		.link()?;
	Ok(newShaderRef(shader))
	// if BASE_SHADER_REF.get().is_none() {
	// 	let shader = Shader::new(gl)?
	// 		.attachFromSource(ShaderType::Vertex, BASE_VERTEX)?
	// 		.attachFromSource(ShaderType::Fragment, BASE_FRAGMENT)?
	// 		.link()?;
	// 	BASE_SHADER_REF.set(newShaderRef(shader));
	// }
	// Ok(BASE_SHADER_REF.get().unwrap().clone())
}

pub fn instanceShader(gl: GlRef) -> Result<ShaderRef, String> {
	let shader = Shader::new(gl)?
		.attachFromSource(ShaderType::Vertex, INSTANCE_VERTEX)?
		.attachFromSource(ShaderType::Fragment, INSTANCE_FRAGMENT)?
		.link()?;
	Ok(newShaderRef(shader))
	// if INSTANCE_SHADER_REF.get().is_none() {
	// 	let shader = Shader::new(gl)?
	// 		.attachFromSource(ShaderType::Vertex, INSTANCE_VERTEX)?
	// 		.attachFromSource(ShaderType::Fragment, INSTANCE_FRAGMENT)?
	// 		.link()?;
	// 	INSTANCE_SHADER_REF.set(newShaderRef(shader));
	// }
	// Ok(INSTANCE_SHADER_REF.get().unwrap().clone())
}