use glow::{Buffer, HasContext, VertexArray};
use bytemuck::{cast_slice, offset_of};
use tracing::{error, info, warn};
use crate::gl_check_error;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::types::{GlRef, ShaderRef};

pub struct SimpleMesh {
	gl: GlRef,
	vao: Option<VertexArray>,
	vbo: Option<Buffer>,
	ibo: Option<Buffer>,
	
	vertices: Vec<Vertex>,
	indices: Vec<u32>,
}

impl SimpleMesh {
	#[allow(unused)]
	pub fn withVertices(gl: GlRef, vertices: Vec<Vertex>) -> Self {
		Self {
			gl,
			vao: None,
			vbo: None,
			ibo: None,
			
			vertices,
			indices: vec![],
		}
	}
	
	pub fn withIndices(gl: GlRef, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
		Self {
			gl,
			vao: None,
			vbo: None,
			ibo: None,
			
			vertices,
			indices,
		}
	}
}

impl Mesh for SimpleMesh {
	fn isUploaded(&self) -> bool {
		self.vao.is_some() && self.vbo.is_some()
	}
	
	fn upload(&mut self, shader: ShaderRef) -> Result<(), String> {
		unsafe {
			let vao = self.gl.create_vertex_array()?;
			let vbo = self.gl.create_named_buffer()?;
			self.gl.bind_vertex_array(Some(vao));
			gl_check_error!(self.gl);
			info!("Uploading simple mesh {:?}", vao.0);
			
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
			self.vbo = Some(vbo);
			
			let locPos = shader.getAttribLocation("i_position").unwrap();
			let locCol = shader.getAttribLocation("i_color").unwrap();
			
			self.gl.enable_vertex_array_attrib(vao, locPos);
			self.gl.vertex_array_attrib_format_f32(vao, locPos, 3, glow::FLOAT, false, offset_of!(Vertex, position) as u32);
			self.gl.vertex_array_attrib_binding_f32(vao, locPos, 0);
			gl_check_error!(self.gl);
			
			self.gl.enable_vertex_array_attrib(vao, locCol);
			self.gl.vertex_array_attrib_format_f32(vao, locCol, 3, glow::FLOAT, false, offset_of!(Vertex, color) as u32);
			self.gl.vertex_array_attrib_binding_f32(vao, locCol, 0);
			gl_check_error!(self.gl);
			
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
				self.gl.draw_elements(glow::TRIANGLES, self.indices.len() as i32, glow::UNSIGNED_INT, 0);
			} else {
				self.gl.draw_arrays(glow::TRIANGLES, 0, self.indices.len() as i32);
			}
			self.gl.bind_vertex_array(None);
		}
	}
	
	fn destroy(&mut self) {
		unsafe {
			warn!("Destroying simple mesh {}", self.vao.unwrap().0);
			self.gl.delete_buffer(self.ibo.unwrap());
			self.gl.delete_buffer(self.vbo.unwrap());
			self.gl.delete_vertex_array(self.vao.unwrap());
		}
	}
}

impl Drop for SimpleMesh {
	fn drop(&mut self) {
		self.destroy();
	}
}