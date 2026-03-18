use bool_flags::Flags8;
use bytemuck::{cast_slice, offset_of, Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use glow::{Buffer, HasContext, VertexArray};
use tracing::{error, info, warn};
use crate::gl_check_error;
use crate::types::{GlRef, ShaderRef};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
	pub position: Vec3,
	pub color: Vec3,
}

impl Default for Vertex {
	fn default() -> Self {
		Self {
			position: Vec3::ZERO,
			color: Vec3::ONE,
		}
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct InstanceMeshData {
	pub matrix: Mat4,
	pub color: Vec4,
}

const F_DESTROYED: u8 = 0;
const F_INSTANCE: u8 = 1;
const F_UPLOADED: u8 = 2;

pub struct Mesh {
	gl: GlRef,
	flags: Flags8,
	
	vao: Option<VertexArray>,
	vboMesh: Option<Buffer>,
	ibo: Option<Buffer>,
	
	vboInstance: Option<Buffer>,
	instanceAmount: i32,
	
	vertices: Vec<Vertex>,
	indices: Option<Vec<u32>>,
}

impl Mesh {
	pub fn simple(gl: GlRef, vertices: Vec<Vertex>, indices: Option<Vec<u32>>) -> Self {
		let flags = Flags8::none();
		Self {
			gl,
			flags,
			
			vao: None,
			vboMesh: None,
			ibo: None,
			
			vboInstance: None,
			instanceAmount: 0,
			
			vertices,
			indices,
		}
	}
	
	pub fn instance(gl: GlRef, vertices: Vec<Vertex>, indices: Option<Vec<u32>>) -> Self {
		let mut flags = Flags8::none();
		flags.set(F_INSTANCE);
		Self {
			gl,
			flags,
			
			vao: None,
			vboMesh: None,
			ibo: None,
			
			vboInstance: None,
			instanceAmount: 0,
			
			vertices,
			indices,
		}
	}
	
	pub fn isDestroyed(&self) -> bool {
		self.flags.get(F_DESTROYED)
	}
	
	pub fn isInstance(&self) -> bool {
		self.flags.get(F_INSTANCE)
	}
	
	pub fn isUploaded(&self) -> bool {
		self.flags.get(F_UPLOADED)
	}
	
	fn checkDestroyed(&self) -> Result<(), String> {
		if self.isDestroyed() {
			return Err("Mesh is already destroyed!".to_string());
		}
		Ok(())
	}
	
	pub fn uploadInstanceData(&mut self, modelMatrices: &Vec<InstanceMeshData>) -> Result<(), String> {
		self.checkDestroyed()?;
		if !self.isInstance() {
			return Err("Can't upload instance data to non-instance mesh!".to_string());
		}
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
	
	pub fn updateInstanceData(&mut self, modelMatrices: &Vec<InstanceMeshData>) -> Result<(), String> {
		self.checkDestroyed()?;
		if !self.isInstance() {
			return Err("Can't update instance data to non-instance mesh!".to_string());
		}
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
	
	pub fn upload(&mut self, shader: ShaderRef) -> Result<(), String> {
		self.checkDestroyed()?;
		unsafe {
			let vao = self.gl.create_vertex_array()?;
			let vbo = self.gl.create_named_buffer()?;
			self.gl.bind_vertex_array(Some(vao));
			gl_check_error!(self.gl);
			info!("Uploading mesh {:?}", vao.0);
			
			let stride = size_of::<Vertex>() as i32;
			self.gl.named_buffer_data_u8_slice(vbo, cast_slice(&self.vertices), glow::STATIC_DRAW);
			self.gl.vertex_array_vertex_buffer(vao, 0, Some(vbo), 0, stride);
			gl_check_error!(self.gl);
			
			if let Some(indices) = &self.indices {
				let ibo = self.gl.create_named_buffer()?;
				gl_check_error!(self.gl);
				
				self.gl.named_buffer_data_u8_slice(ibo, cast_slice(indices), glow::STATIC_DRAW);
				self.gl.vertex_array_element_buffer(vao, Some(ibo));
				gl_check_error!(self.gl);
				
				self.ibo = Some(ibo);
			}
			
			self.vao = Some(vao);
			self.vboMesh = Some(vbo);
			
			let locPos = shader.read().unwrap().getAttribLocation("i_position").unwrap();
			let locCol = shader.read().unwrap().getAttribLocation("i_color").unwrap();
			
			self.gl.enable_vertex_array_attrib(vao, locPos);
			self.gl.vertex_array_attrib_format_f32(vao, locPos, 3, glow::FLOAT, false, offset_of!(Vertex, position) as u32);
			self.gl.vertex_array_attrib_binding_f32(vao, locPos, 0);
			gl_check_error!(self.gl);
			
			if self.isInstance() {
				let locModel = shader.read().unwrap().getAttribLocation("i_model").unwrap();
				
				// instance model matrix
				let vec4Size = size_of::<Vec4>() as u32;
				self.gl.vertex_array_vertex_buffer(vao, 1, self.vboInstance, 0, vec4Size as i32 * 5);
				gl_check_error!(self.gl);
				
				self.gl.enable_vertex_array_attrib(vao, locModel + 0);
				self.gl.vertex_array_attrib_format_f32(vao, locModel + 0, 4, glow::FLOAT, false, vec4Size * 0);
				self.gl.vertex_array_attrib_binding_f32(vao, locModel + 0, 1);
				gl_check_error!(self.gl);
				
				self.gl.enable_vertex_array_attrib(vao, locModel + 1);
				self.gl.vertex_array_attrib_format_f32(vao, locModel + 1, 4, glow::FLOAT, false, vec4Size * 1);
				self.gl.vertex_array_attrib_binding_f32(vao, locModel + 1, 1);
				gl_check_error!(self.gl);
				
				self.gl.enable_vertex_array_attrib(vao, locModel + 2);
				self.gl.vertex_array_attrib_format_f32(vao, locModel + 2, 4, glow::FLOAT, false, vec4Size * 2);
				self.gl.vertex_array_attrib_binding_f32(vao, locModel + 2, 1);
				gl_check_error!(self.gl);
				
				self.gl.enable_vertex_array_attrib(vao, locModel + 3);
				self.gl.vertex_array_attrib_format_f32(vao, locModel + 3, 4, glow::FLOAT, false, vec4Size * 3);
				self.gl.vertex_array_attrib_binding_f32(vao, locModel + 3, 1);
				gl_check_error!(self.gl);
				
				// instance color
				self.gl.enable_vertex_array_attrib(vao, locCol);
				self.gl.vertex_array_attrib_format_f32(vao, locCol, 4, glow::FLOAT, false, vec4Size * 4);
				self.gl.vertex_array_attrib_binding_f32(vao, locCol, 1);
				gl_check_error!(self.gl);
				
				self.gl.vertex_binding_divisor(1, 1);
				gl_check_error!(self.gl);
				// self.gl.vertex_attrib_divisor(locModel + 0, 1);
				// self.gl.vertex_attrib_divisor(locModel + 1, 1);
				// self.gl.vertex_attrib_divisor(locModel + 2, 1);
				// self.gl.vertex_attrib_divisor(locModel + 3, 1);
			} else {
				self.gl.enable_vertex_array_attrib(vao, locCol);
				self.gl.vertex_array_attrib_format_f32(vao, locCol, 3, glow::FLOAT, false, offset_of!(Vertex, color) as u32);
				self.gl.vertex_array_attrib_binding_f32(vao, locCol, 0);
				gl_check_error!(self.gl);
			}
			
			self.gl.bind_vertex_array(None);
			self.flags.set(F_UPLOADED);
			Ok(())
		}
	}
	
	pub fn draw(&self) {
		if self.checkDestroyed().is_err() {
			return;
		}
		if !self.isUploaded() {
			error!("Upload mesh before drawing!");
			return;
		}
		
		unsafe {
			self.gl.bind_vertex_array(self.vao);
			gl_check_error!(self.gl);
			
			if self.isInstance() {
				if let Some(indices) = &self.indices {
					self.gl.draw_elements_instanced(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_INT, 0, self.instanceAmount);
				} else {
					self.gl.draw_arrays_instanced(glow::TRIANGLES, 0, self.vertices.len() as i32, self.instanceAmount);
				}
			} else {
				if let Some(indices) = &self.indices {
					self.gl.draw_elements(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_INT, 0);
				} else {
					self.gl.draw_arrays(glow::TRIANGLES, 0, self.vertices.len() as i32);
				}
			}
			gl_check_error!(self.gl);
			self.gl.bind_vertex_array(None);
		}
	}
	
	pub fn destroy(&mut self) {
		if self.isDestroyed() || !self.isUploaded() {
			return;
		}
		unsafe {
			warn!("Destroying mesh {}", self.vao.unwrap().0);
			if self.isInstance() {
				self.gl.delete_buffer(self.vboInstance.unwrap());
			}
			
			self.gl.delete_buffer(self.ibo.unwrap());
			self.gl.delete_buffer(self.vboMesh.unwrap());
			self.gl.delete_vertex_array(self.vao.unwrap());
			self.flags.set(F_DESTROYED);
		}
	}
}

impl Drop for Mesh {
	fn drop(&mut self) {
		self.destroy();
	}
}