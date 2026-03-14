use bytemuck::{cast_slice, offset_of};
use glam::{Mat4, Vec4};
use glow::{Buffer, HasContext, VertexArray};
use tracing::{error, warn};
use crate::gl_check_error;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::types::{GlRef, ShaderRef};

pub struct InstanceMesh {
	gl: GlRef,
	vao: Option<VertexArray>,
	vboMesh: Option<Buffer>,
	ibo: Option<Buffer>,
	
	vboInstance: Option<Buffer>,
	instanceAmount: i32,
	
	vertices: Vec<Vertex>,
	indices: Vec<u32>,
}

impl InstanceMesh {
	pub fn withVertices(gl: GlRef, vertices: Vec<Vertex>) -> Self {
		Self {
			gl,
			vao: None,
			vboMesh: None,
			ibo: None,
			
			vboInstance: None,
			instanceAmount: 0,
			
			vertices,
			indices: vec![],
		}
	}
	
	pub fn withIndices(gl: GlRef, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
		Self {
			gl,
			vao: None,
			vboMesh: None,
			ibo: None,
			
			vboInstance: None,
			instanceAmount: 0,
			
			vertices,
			indices,
		}
	}
	
	pub fn uploadInstanceData(&mut self, modelMatrices: &Vec<Mat4>) -> Result<(), String> {
		unsafe {
			let vbo = self.gl.create_named_buffer()?;
			gl_check_error!(self.gl);
			
			self.gl.named_buffer_data_u8_slice(vbo, cast_slice(modelMatrices), glow::STREAM_DRAW);
			gl_check_error!(self.gl);
			
			self.vboInstance = Some(vbo);
			self.instanceAmount = modelMatrices.len() as i32;
			Ok(())
		}
	}
	
	pub fn updateInstanceData(&mut self, modelMatrices: &Vec<Mat4>) -> Result<(), String> {
		unsafe {
			let instanceAmount = modelMatrices.len() as i32;
			if instanceAmount > self.instanceAmount {
				self.gl.named_buffer_data_u8_slice(self.vboInstance.unwrap(), cast_slice(modelMatrices), glow::STREAM_DRAW);
			} else {
				self.gl.named_buffer_sub_data_u8_slice(self.vboInstance.unwrap(), 0, cast_slice(modelMatrices));
			}
			gl_check_error!(self.gl);
			self.instanceAmount = instanceAmount;
			Ok(())
		}
	}
}

impl Mesh for InstanceMesh {
	fn isUploaded(&self) -> bool {
		self.vao.is_some() && self.vboMesh.is_some() && self.vboInstance.is_some()
	}
	
	fn upload(&mut self, shader: ShaderRef) -> Result<(), String> {
		unsafe {
			let vao = self.gl.create_vertex_array()?;
			let vbo = self.gl.create_named_buffer()?;
			self.gl.bind_vertex_array(Some(vao));
			gl_check_error!(self.gl);
			
			let stride = size_of::<Vertex>() as i32;
			self.gl.named_buffer_data_u8_slice(vbo, cast_slice(&self.vertices), glow::STATIC_DRAW);
			self.gl.vertex_array_vertex_buffer(vao, 0, Some(vbo), 0, stride);
			gl_check_error!(self.gl);
			
			if !self.indices.is_empty() {
				let ibo = self.gl.create_named_buffer()?;
				gl_check_error!(self.gl);
				
				self.gl.named_buffer_data_u8_slice(ibo, cast_slice(&self.indices), glow::STATIC_DRAW);
				self.gl.vertex_array_element_buffer(vao, Some(ibo));
				gl_check_error!(self.gl);
				
				self.ibo = Some(ibo);
			}
			
			self.vao = Some(vao);
			self.vboMesh = Some(vbo);
			
			let locPos = shader.getAttribLocation("i_position").unwrap();
			let locCol = shader.getAttribLocation("i_color").unwrap();
			let locModel = shader.getAttribLocation("i_model").unwrap();
			
			self.gl.enable_vertex_array_attrib(vao, locPos);
			self.gl.vertex_array_attrib_format_f32(vao, locPos, 3, glow::FLOAT, false, offset_of!(Vertex, position) as u32);
			self.gl.vertex_array_attrib_binding_f32(vao, locPos, 0);
			gl_check_error!(self.gl);
			
			self.gl.enable_vertex_array_attrib(vao, locCol);
			self.gl.vertex_array_attrib_format_f32(vao, locCol, 3, glow::FLOAT, false, offset_of!(Vertex, color) as u32);
			self.gl.vertex_array_attrib_binding_f32(vao, locCol, 0);
			gl_check_error!(self.gl);
			
			// bind instance model matrix vbo
			let vec4Size = size_of::<Vec4>() as u32;
			self.gl.vertex_array_vertex_buffer(vao, 1, self.vboInstance, 0, vec4Size as i32 * 4);
			gl_check_error!(self.gl);
			
			self.gl.enable_vertex_array_attrib(vao, locModel + 0);
			self.gl.vertex_array_attrib_format_f32(vao, locModel + 0, 4, glow::FLOAT, false, vec4Size * 0);
			gl_check_error!(self.gl);
			self.gl.vertex_array_attrib_binding_f32(vao, locModel + 0, 1);
			
			self.gl.enable_vertex_array_attrib(vao, locModel + 1);
			self.gl.vertex_array_attrib_format_f32(vao, locModel + 1, 4, glow::FLOAT, false, vec4Size * 1);
			gl_check_error!(self.gl);
			self.gl.vertex_array_attrib_binding_f32(vao, locModel + 1, 1);
			
			self.gl.enable_vertex_array_attrib(vao, locModel + 2);
			self.gl.vertex_array_attrib_format_f32(vao, locModel + 2, 4, glow::FLOAT, false, vec4Size * 2);
			gl_check_error!(self.gl);
			self.gl.vertex_array_attrib_binding_f32(vao, locModel + 2, 1);
			
			self.gl.enable_vertex_array_attrib(vao, locModel + 3);
			self.gl.vertex_array_attrib_format_f32(vao, locModel + 3, 4, glow::FLOAT, false, vec4Size * 3);
			gl_check_error!(self.gl);
			self.gl.vertex_array_attrib_binding_f32(vao, locModel + 3, 1);
			
			self.gl.vertex_binding_divisor(1, 1);
			gl_check_error!(self.gl);
			// self.gl.vertex_attrib_divisor(locModel + 0, 1);
			// self.gl.vertex_attrib_divisor(locModel + 1, 1);
			// self.gl.vertex_attrib_divisor(locModel + 2, 1);
			// self.gl.vertex_attrib_divisor(locModel + 3, 1);
			
			self.gl.bind_vertex_array(None);
			
			Ok(())
		}
	}
	
	fn draw(&self) {
		if !self.isUploaded() {
			error!("Mesh not uploaded to GPU!");
			return;
		}
		
		unsafe {
			self.gl.bind_vertex_array(self.vao);
			gl_check_error!(self.gl);
			
			if let Some(_) = self.ibo {
				self.gl.draw_elements_instanced(glow::TRIANGLES, self.indices.len() as i32, glow::UNSIGNED_INT, 0, self.instanceAmount);
			} else {
				self.gl.draw_arrays_instanced(glow::TRIANGLES, 0, self.indices.len() as i32, self.instanceAmount);
			}
			self.gl.bind_vertex_array(None);
		}
	}
	
	fn destroy(&mut self) {
		unsafe {
			warn!("Destroying instance mesh ({})", self.vao.unwrap().0);
			self.gl.delete_buffer(self.vboInstance.unwrap());
			self.gl.delete_buffer(self.ibo.unwrap());
			self.gl.delete_buffer(self.vboMesh.unwrap());
			self.gl.delete_vertex_array(self.vao.unwrap());
		}
	}
}

impl Drop for InstanceMesh {
	fn drop(&mut self) {
		self.destroy();
	}
}
