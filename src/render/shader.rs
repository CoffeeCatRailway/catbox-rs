#![allow(non_snake_case)]

use std::rc::Rc;
use glam::{Mat4, Vec2, Vec3, Vec4};
use glow::{Context, HasContext, Program};

type GlShader = glow::Shader;

pub struct Shader {
	gl: Rc<Context>,
	pub program: Program,
	destroyed: bool,
}

#[allow(dead_code)]
impl Shader {
	pub fn newVertFrag(gl: Rc<Context>, vertSource: &str, fragSource: &str) -> Result<Self, String> {
		unsafe {
			let program = gl.create_program().map_err(|e| format!("Failed to create program: {}", e))?;
			
			let vs = compileShader(&gl, vertSource, glow::VERTEX_SHADER)?;
			let fs = compileShader(&gl, fragSource, glow::FRAGMENT_SHADER)?;
			
			gl.attach_shader(program, vs);
			gl.attach_shader(program, fs);
			
			gl.bind_frag_data_location(program, glow::COLOR_ATTACHMENT0, "o_color");
			gl.link_program(program);
			
			gl.delete_shader(vs);
			gl.delete_shader(fs);
			
			Ok(Shader {
				gl,
				program,
				destroyed: false,
			})
		}
	}
	
	pub fn newVertGeomFrag(gl: Rc<Context>, vertexPath: &str, geometryPath: &str, fragmentPath: &str) -> Result<Self, String> {
		unsafe {
			let program = gl.create_program().map_err(|e| format!("Failed to create program: {}", e))?;
			
			let vs = compileShader(&gl, vertexPath, glow::VERTEX_SHADER)?;
			let gs = compileShader(&gl, geometryPath, glow::GEOMETRY_SHADER)?;
			let fs = compileShader(&gl, fragmentPath, glow::FRAGMENT_SHADER)?;
			
			gl.attach_shader(program, vs);
			gl.attach_shader(program, gs);
			gl.attach_shader(program, fs);
			
			gl.bind_frag_data_location(program, glow::COLOR_ATTACHMENT0, "o_color");
			gl.link_program(program);
			
			gl.delete_shader(vs);
			gl.delete_shader(gs);
			gl.delete_shader(fs);
			
			Ok(Shader {
				gl,
				program,
				destroyed: false,
			})
		}
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
			self.gl.delete_program(self.program);
		}
		self.destroyed = true;
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

fn compileShader(gl: &Context, source: &str, shaderType: u32) -> Result<GlShader, String> {
	// let src = fs::read_to_string(path).map_err(|e| format!("Failed to read shader file ({}): {}", path, e))?;
	
	unsafe {
		let shader = gl.create_shader(shaderType).map_err(|e| format!("Failed to create shader: {}", e))?;
		gl.shader_source(shader, source);
		gl.compile_shader(shader);
		
		if !gl.get_shader_compile_status(shader) {
			let error = gl.get_shader_info_log(shader);
			return Err(format!("Failed to compile shader: {}", error));
		}
		Ok(shader)
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		self.delete();
	}
}
