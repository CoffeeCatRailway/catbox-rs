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

pub const DEPTH_MAP_VERTEX: &str = include_str!("../../resources/shaders/depth_map.vert");
pub const DEPTH_MAP_FRAGMENT: &str = include_str!("../../resources/shaders/depth_map.frag");
static DEPTH_MAP_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

fn getOrInitSimpleShader(gl: GlRef, shaderRef: &OnceLock<ShaderRef>, vertex: &str, fragment: &str) -> ShaderRef {
	shaderRef.get_or_init(|| {
		newShaderRef(Shader::new(gl).logErr().unwrap()
		                            .attachFromSource(ShaderType::Vertex, vertex).logErr().unwrap()
		                            .attachFromSource(ShaderType::Fragment, fragment).logErr().unwrap()
		                            .link().logErr().unwrap())
	}).clone()
}

pub fn simpleAttribColorShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &SIMPLE_ATTRIB_COLOR_SHADER_REF, SIMPLE_ATTRIB_COLOR_VERTEX, SIMPLE_ATTRIB_COLOR_FRAGMENT)
}

pub fn simpleMatColorShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &SIMPLE_MAT_COLOR_SHADER_REF, SIMPLE_MAT_COLOR_VERTEX, SIMPLE_MAT_COLOR_FRAGMENT)
}

pub fn simpleLightShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &SIMPLE_LIGHT_SHADER_REF, SIMPLE_LIGHT_VERTEX, SIMPLE_LIGHT_FRAGMENT)
}

pub fn instanceShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &INSTANCE_SHADER_REF, INSTANCE_VERTEX, INSTANCE_FRAGMENT)
}

pub fn depthMapShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &DEPTH_MAP_SHADER_REF, DEPTH_MAP_VERTEX, DEPTH_MAP_FRAGMENT)
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
	destroyShaderRef(&DEPTH_MAP_SHADER_REF);
}
