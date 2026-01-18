#![allow(non_snake_case)]

use std::rc::Rc;
use glam::{Mat4, Vec2, Vec3, Vec4};
use glow::{Context, HasContext, Program};
use log::{error, info};

type GlShader = glow::Shader;

pub struct Shader {
	gl: Rc<Context>,
	program: Program,
	destroyed: bool,
	linked: bool,
	shaders: Vec<GlShader>,
}

#[allow(dead_code)]
impl Shader {
	pub fn new(gl: Rc<Context>) -> Self {
		unsafe {
			info!("Creating shader program...");
			let program = gl.create_program().expect("Failed create shader program!");
			Shader {
				gl,
				program,
				destroyed: false,
				linked: false,
				shaders: Vec::new(),
			}
		}
	}
	
	pub fn addFromSource(mut self, stype: u32, source: &str) -> Self {
		if self.linked {
			error!("Shader program already linked! You can not attach more shaders!");
			return self;
		}
		// let src = fs::read_to_string(path).map_err(|e| format!("Failed to read shader file ({}): {}", path, e))?;
		unsafe {
			info!("Attaching {} shader to program {}...", match stype {
				glow::VERTEX_SHADER => "vertex",
				glow::TESS_CONTROL_SHADER => "tess-control",
				glow::TESS_EVALUATION_SHADER => "tess-evaluation",
				glow::GEOMETRY_SHADER => "geometry",
				glow::FRAGMENT_SHADER => "fragment",
				glow::COMPUTE_SHADER => "compute",
				_ => "UNKNOWN",
			}, self.program.0);
			
			let shader = self.gl.create_shader(stype).expect(format!("Failed to create shader of type {}!", stype).as_str());
			self.gl.shader_source(shader, source);
			self.gl.compile_shader(shader);
			
			if !self.gl.get_shader_compile_status(shader) {
				let error = self.gl.get_shader_info_log(shader);
				panic!("Failed to compile shader: {}", error);
			}
			self.gl.attach_shader(self.program, shader);
			self.shaders.push(shader);
		}
		self
	}
	
	pub fn link(mut self) -> Self {
		if self.linked {
			error!("Shader program already linked!");
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
				self.gl.delete_shader(*shader);
			}
			self.shaders = Vec::new(); // Clear and deallocate
			self.linked = true;
		}
		self
	}
	
	pub fn program(&self) -> &Program {
		&self.program
	}
	
	pub fn bind(&self) {
		unsafe {
			self.gl.use_program(Some(self.program));
		}
	}
	
	pub fn delete(&mut self) {
		if self.destroyed {
			return;
		}
		unsafe {
			info!("Deleting shader program {}", self.program.0);
			self.gl.delete_program(self.program);
			self.destroyed = true;
		}
	}
	
	pub fn getAttribLocation(&self, name: &str) -> Option<u32> {
		unsafe {
			self.gl.get_attrib_location(self.program, name)
		}
	}
	
	// Uniforms
	pub fn setUniform1i(&self, name: &str, value: i32) {
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_1_i32(loc, value);
		}
	}
	
	pub fn setUniform1ui(&self, name: &str, value: u32) {
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_1_u32(loc, value);
		}
	}
	
	pub fn setUniform1f(&self, name: &str, value: f32) {
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_1_f32(loc, value);
		}
	}
	
	pub fn setUniform2fv(&self, name: &str, value: &Vec2) {
		self.setUniform2f(name, value.x, value.y);
	}
	
	pub fn setUniform2f(&self, name: &str, x: f32, y: f32) {
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_2_f32(loc, x, y);
		}
	}
	
	pub fn setUniform3fv(&self, name: &str, value: &Vec3) {
		self.setUniform3f(name, value.x, value.y, value.z);
	}
	
	pub fn setUniform3f(&self, name: &str, x: f32, y: f32, z: f32) {
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_3_f32(loc, x, y, z);
		}
	}
	
	pub fn setUniform4fv(&self, name: &str, value: &Vec4) {
		self.setUniform4f(name, value.x, value.y, value.z, value.w);
	}
	
	pub fn setUniform4f(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
		unsafe {
			let loc = Some(&self.gl.get_uniform_location(self.program, name).unwrap());
			self.gl.uniform_4_f32(loc, x, y, z, w);
		}
	}
	
	pub fn setMatrix4f(&self, name: &str, mat: &Mat4) {
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
