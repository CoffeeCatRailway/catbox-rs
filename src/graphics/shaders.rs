use std::sync::OnceLock;
use crate::graphics::shader::{Shader, ShaderType};
use crate::LogError;
use crate::types::{newShaderRef, GlRef, ShaderRef};

pub const COLOR_ATTRIBUTE_VERTEX: &str = include_str!("../../resources/shaders/color_attribute.vert");
pub const COLOR_ATTRIBUTE_FRAGMENT: &str = include_str!("../../resources/shaders/color_attribute.frag");
static COLOR_ATTRIBUTE_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const COLOR_MATERIAL_VERTEX: &str = include_str!("../../resources/shaders/color_material.vert");
pub const COLOR_MATERIAL_FRAGMENT: &str = include_str!("../../resources/shaders/color_material.frag");
static COLOR_MATERIAL_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const LIGHT_SIMPLE_SHADOW_VERTEX: &str = include_str!("../../resources/shaders/light_simple_shadow.vert");
pub const LIGHT_SIMPLE_SHADOW_FRAGMENT: &str = include_str!("../../resources/shaders/light_simple_shadow.frag");
static LIGHT_SIMPLE_SHADOW_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const LIGHT_CASCADE_SHADOW_VERTEX: &str = include_str!("../../resources/shaders/light_cascade_shadow.vert");
pub const LIGHT_CASCADE_SHADOW_FRAGMENT: &str = include_str!("../../resources/shaders/light_cascade_shadow.frag");
static LIGHT_CASCADE_SHADOW_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const INSTANCE_VERTEX: &str = include_str!("../../resources/shaders/instance.vert");
pub const INSTANCE_FRAGMENT: &str = include_str!("../../resources/shaders/instance.frag");
static INSTANCE_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const DEPTH_MAP_VERTEX: &str = include_str!("../../resources/shaders/depth_map.vert");
pub const DEPTH_MAP_FRAGMENT: &str = include_str!("../../resources/shaders/depth_map.frag");
static DEPTH_MAP_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

pub const DEPTH_MAP_ARRAY_VERTEX: &str = include_str!("../../resources/shaders/depth_map_array.vert");
pub const DEPTH_MAP_ARRAY_GEOMETRY: &str = include_str!("../../resources/shaders/depth_map_array.geom");
static DEPTH_MAP_ARRAY_SHADER_REF: OnceLock<ShaderRef> = OnceLock::new();

fn getOrInitSimpleShader(gl: GlRef, shaderRef: &OnceLock<ShaderRef>, vertex: &str, fragment: &str) -> ShaderRef {
	shaderRef.get_or_init(|| {
		newShaderRef(Shader::new(gl).logErr().unwrap()
		                            .attachFromSource(ShaderType::Vertex, vertex).logErr().unwrap()
		                            .attachFromSource(ShaderType::Fragment, fragment).logErr().unwrap()
		                            .link().logErr().unwrap())
	}).clone()
}

pub fn colorAttributeShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &COLOR_ATTRIBUTE_SHADER_REF, COLOR_ATTRIBUTE_VERTEX, COLOR_ATTRIBUTE_FRAGMENT)
}

pub fn colorMaterialShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &COLOR_MATERIAL_SHADER_REF, COLOR_MATERIAL_VERTEX, COLOR_MATERIAL_FRAGMENT)
}

pub fn lightSimpleShadowShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &LIGHT_SIMPLE_SHADOW_SHADER_REF, LIGHT_SIMPLE_SHADOW_VERTEX, LIGHT_SIMPLE_SHADOW_FRAGMENT)
}

pub fn lightCascadeShadowShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &LIGHT_CASCADE_SHADOW_SHADER_REF, LIGHT_CASCADE_SHADOW_VERTEX, LIGHT_CASCADE_SHADOW_FRAGMENT)
}

pub fn instanceShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &INSTANCE_SHADER_REF, INSTANCE_VERTEX, INSTANCE_FRAGMENT)
}

pub fn depthMapShader(gl: GlRef) -> ShaderRef {
	getOrInitSimpleShader(gl, &DEPTH_MAP_SHADER_REF, DEPTH_MAP_VERTEX, DEPTH_MAP_FRAGMENT)
}

pub fn depthMapArrayShader(gl: GlRef) -> ShaderRef {
	DEPTH_MAP_ARRAY_SHADER_REF.get_or_init(|| {
		newShaderRef(Shader::new(gl).logErr().unwrap()
		                            .attachFromSource(ShaderType::Vertex, DEPTH_MAP_ARRAY_VERTEX).logErr().unwrap()
		                            .attachFromSource(ShaderType::Geometry, DEPTH_MAP_ARRAY_GEOMETRY).logErr().unwrap()
		                            .attachFromSource(ShaderType::Fragment, DEPTH_MAP_FRAGMENT).logErr().unwrap()
		                            .link().logErr().unwrap())
	}).clone()
}

fn destroyShaderRef(shader: &OnceLock<ShaderRef>) {
	if let Some(shader) = shader.get() {
		shader.write().unwrap().destroy();
	}
}

pub fn destroyAllShaders() {
	destroyShaderRef(&COLOR_ATTRIBUTE_SHADER_REF);
	destroyShaderRef(&COLOR_MATERIAL_SHADER_REF);
	
	destroyShaderRef(&LIGHT_SIMPLE_SHADOW_SHADER_REF);
	destroyShaderRef(&LIGHT_CASCADE_SHADOW_SHADER_REF);
	
	destroyShaderRef(&INSTANCE_SHADER_REF);
	
	destroyShaderRef(&DEPTH_MAP_SHADER_REF);
	destroyShaderRef(&DEPTH_MAP_ARRAY_SHADER_REF);
}
