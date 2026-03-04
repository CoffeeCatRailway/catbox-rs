use bool_flags::Flags8;
use glam::{Mat4, Vec2, Vec3, Vec4};
use glow::{HasContext, Program};
use tracing::{error, info, warn};
use crate::types::GlRef;

type GlowShader = glow::Shader;

const F_DESTROYED: u8 = 0;
const F_LINKED: u8 = 	1;

#[allow(unused)]
pub enum ShaderType {
	Vertex,
	Fragment,
	Geometry,
	Compute,
}

pub struct Shader {
	gl: GlRef,
	program: Program,
	flags: Flags8,
	shaders: Vec<GlowShader>,
}

#[allow(unused)]
impl Shader {
	fn checks(&self) -> bool {
		if self.flags.get(F_DESTROYED) {
			error!("Shader program was destroyed!");
			return false;
		}
		if !self.flags.get(F_LINKED) {
			error!("Shader program {} is not linked!", self.program.0);
			return false;
		}
		true
	}
	
	pub fn new(gl: GlRef) -> Self {
		unsafe {
			info!("Creating shader program");
			let program = gl.create_program().expect("Failed to create shader program");
			Shader {
				gl,
				program,
				flags: Flags8::none(),
				shaders: Vec::new(),
			}
		}
	}
	
	pub fn attachFromSource(mut self, shaderType: ShaderType, source: &str) -> Self {
		if self.flags.get(F_DESTROYED) {
			panic!("Shader program was destroyed before linking!");
		}
		if self.flags.get(F_LINKED) {
			error!("Shader program {} is already linked! Unable to attach other shaders!", self.program.0);
			return self;
		}
		unsafe {
			let (typeStr, typeGlow) = match shaderType {
				ShaderType::Vertex => ("vertex", glow::VERTEX_SHADER),
				ShaderType::Fragment => ("fragment", glow::FRAGMENT_SHADER),
				ShaderType::Geometry => ("geometry", glow::GEOMETRY_SHADER),
				ShaderType::Compute => ("compute", glow::COMPUTE_SHADER),
			};
			info!("Attaching {} shader to program {}...", typeStr, self.program.0);
			
			let shader = self.gl.create_shader(typeGlow).expect(format!("Failed to create shader of type '{}'", typeStr).as_str());
			self.gl.shader_source(shader, source);
			self.gl.compile_shader(shader);
			
			if !self.gl.get_shader_compile_status(shader) {
				let error = self.gl.get_shader_info_log(shader);
				panic!("Failed to compile shader: {error}");
			}
			self.gl.attach_shader(self.program, shader);
			self.shaders.push(shader);
		}
		self
	}
	
	pub fn link(mut self) -> Self {
		if self.flags.get(F_DESTROYED) {
			panic!("Shader program was destroyed before linking!");
		}
		if self.flags.get(F_LINKED) {
			error!("Shader program {} is already linked!", self.program.0);
			return self;
		}
		unsafe {
			info!("Linking shader program {}...", self.program.0);
			
			self.gl.bind_frag_data_location(self.program, glow::COLOR_ATTACHMENT0, "o_color");
			self.gl.link_program(self.program);
			if !self.gl.get_program_link_status(self.program) {
				let error = self.gl.get_program_info_log(self.program);
				panic!("Failed to link shader: {}", error);
			}
			
			for shader in self.shaders.iter() {
				self.gl.detach_shader(self.program, *shader);
				self.gl.delete_shader(*shader);
			}
			self.shaders = Vec::new(); // Clear and deallocate
			self.flags.set(F_LINKED)
		}
		self
	}
	
	pub fn program(&self) -> Option<&Program> {
		if self.flags.get(F_DESTROYED) || !self.flags.get(F_LINKED) {
			None
		} else {
			Some(&self.program)
		}
	}
	
	pub fn bind(&self) {
		if !self.checks() {
			return;
		}
		unsafe {
			self.gl.use_program(Some(self.program));
		}
	}
	
	pub fn delete(&mut self) {
		if self.flags.get(F_DESTROYED) {
			return;
		}
		unsafe {
			warn!("Deleting shader program {}...", self.program.0);
			self.gl.delete_program(self.program);
			self.flags.set(F_DESTROYED);
		}
	}
	
	pub fn getAttribLocation(&self, name: &str) -> Option<u32> {
		if !self.checks() {
			return None;
		}
		unsafe {
			self.gl.get_attrib_location(self.program, name)
		}
	}
	
	// Uniforms
	pub fn setUniform1i(&self, name: &str, value: i32) {
		if !self.checks() {
			return;
		}
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_1_i32(loc, value);
		}
	}
	
	pub fn setUniform1ui(&self, name: &str, value: u32) {
		if !self.checks() {
			return;
		}
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_1_u32(loc, value);
		}
	}
	
	pub fn setUniform1f(&self, name: &str, value: f32) {
		if !self.checks() {
			return;
		}
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_1_f32(loc, value);
		}
	}
	
	pub fn setUniform2fv(&self, name: &str, value: &Vec2) {
		self.setUniform2f(name, value.x, value.y);
	}
	
	pub fn setUniform2f(&self, name: &str, x: f32, y: f32) {
		if !self.checks() {
			return;
		}
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_2_f32(loc, x, y);
		}
	}
	
	pub fn setUniform3fv(&self, name: &str, value: &Vec3) {
		self.setUniform3f(name, value.x, value.y, value.z);
	}
	
	pub fn setUniform3f(&self, name: &str, x: f32, y: f32, z: f32) {
		if !self.checks() {
			return;
		}
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_3_f32(loc, x, y, z);
		}
	}
	
	pub fn setUniform4fv(&self, name: &str, value: &Vec4) {
		self.setUniform4f(name, value.x, value.y, value.z, value.w);
	}
	
	pub fn setUniform4f(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
		if !self.checks() {
			return;
		}
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_4_f32(loc, x, y, z, w);
		}
	}
	
	pub fn setMatrix4f(&self, name: &str, mat: &Mat4) {
		if !self.checks() {
			return;
		}
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_matrix_4_f32_slice(loc, false, &mat.to_cols_array());
		}
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		self.delete();
	}
}
