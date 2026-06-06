use std::sync::OnceLock;
use crate::graphics::shader::{Shader, ShaderType};
use crate::LogError;
use crate::types::{newShaderRef, GlRef, ShaderRef};

pub const SIMPLE_ATTRIB_COLOR_VERTEX: &str = include_str!("../../resources/shaders/simple_attrib_color.vert");
pub const SIMPLE_ATTRIB_COLOR_FRAGMENT: &str = include_str!("../../resources/shaders/simple_attrib_color.frag");
static SIMPLE_ATTRIB_COLOR_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const SIMPLE_MAT_COLOR_VERTEX: &str = include_str!("../../resources/shaders/simple_mat_color.vert");
pub const SIMPLE_MAT_COLOR_FRAGMENT: &str = include_str!("../../resources/shaders/simple_mat_color.frag");
static SIMPLE_MAT_COLOR_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const SIMPLE_LIGHT_VERTEX: &str = include_str!("../../resources/shaders/simple_light.vert");
pub const SIMPLE_LIGHT_FRAGMENT: &str = include_str!("../../resources/shaders/simple_light.frag");
static SIMPLE_LIGHT_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const INSTANCE_VERTEX: &str = include_str!("../../resources/shaders/instance.vert");
pub const INSTANCE_FRAGMENT: &str = include_str!("../../resources/shaders/instance.frag");
static INSTANCE_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub fn simpleAttribColorShader(gl: GlRef) -> ShaderRef {
	SIMPLE_ATTRIB_COLOR_SHADER_REF.get_or_init(|| {
		newShaderRef(Shader::new(gl).logErr().unwrap()
			.attachFromSource(ShaderType::Vertex, SIMPLE_ATTRIB_COLOR_VERTEX).logErr().unwrap()
			.attachFromSource(ShaderType::Fragment, SIMPLE_ATTRIB_COLOR_FRAGMENT).logErr().unwrap()
			.link().logErr().unwrap())
	}).clone()
}

pub fn simpleMatColorShader(gl: GlRef) -> ShaderRef {
	SIMPLE_MAT_COLOR_SHADER_REF.get_or_init(|| {
		newShaderRef(Shader::new(gl).logErr().unwrap()
			.attachFromSource(ShaderType::Vertex, SIMPLE_MAT_COLOR_VERTEX).logErr().unwrap()
			.attachFromSource(ShaderType::Fragment, SIMPLE_MAT_COLOR_FRAGMENT).logErr().unwrap()
			.link().logErr().unwrap())
	}).clone()
}

pub fn simpleLightShader(gl: GlRef) -> ShaderRef {
	SIMPLE_LIGHT_SHADER_REF.get_or_init(|| {
		newShaderRef(Shader::new(gl).logErr().unwrap()
			.attachFromSource(ShaderType::Vertex, SIMPLE_LIGHT_VERTEX).logErr().unwrap()
			.attachFromSource(ShaderType::Fragment, SIMPLE_LIGHT_FRAGMENT).logErr().unwrap()
			.link().logErr().unwrap())
	}).clone()
}

pub fn instanceShader(gl: GlRef) -> ShaderRef {
	INSTANCE_SHADER_REF.get_or_init(|| {
		newShaderRef(Shader::new(gl).logErr().unwrap()
			.attachFromSource(ShaderType::Vertex, INSTANCE_VERTEX).logErr().unwrap()
			.attachFromSource(ShaderType::Fragment, INSTANCE_FRAGMENT).logErr().unwrap()
			.link().logErr().unwrap())
	}).clone()
}

fn destroyShaderRef(shader: &OnceLock<ShaderRef>) {
	if let Some(shader) = shader.get() {
		shader.write().unwrap().destroy();
	}
}

pub fn destroyAllShaders() {
	destroyShaderRef(&SIMPLE_ATTRIB_COLOR_SHADER_REF);
	destroyShaderRef(&SIMPLE_MAT_COLOR_SHADER_REF);
	destroyShaderRef(&SIMPLE_LIGHT_SHADER_REF);
	destroyShaderRef(&INSTANCE_SHADER_REF);
}
