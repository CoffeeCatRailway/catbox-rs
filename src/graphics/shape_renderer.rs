#![allow(non_snake_case)]

use std::rc::Rc;
use glam::{Mat4, Vec2, Vec3};
use glow::{Buffer, Context, HasContext, VertexArray};
use log::info;
use crate::graphics::Shader;

/*
 TODO:
 - Transparency
 - Outline color
 - Triangles (Would need another float for tangent angle)
 - 3D support, billboards or models, not sure yet
 */
pub struct ShapeRenderer {
	gl: Rc<Context>,
	vec: Vec<f32>,
	shader: Shader,
	vao: VertexArray,
	vbo: Buffer,
	floatsPushed: usize,
	lastFloatsPushed: usize,
	pub enabled: bool,
	destroyed: bool,
}

/*
 * Shader data:
 * - float id (circle, box, line)
 * - float2 pos
 * - float3 color
 * - float2 size (x=radius/length)
 * - float rotation
 * - float outline
 *
 * id treated as float (wasteful but convenient)
 * Floats: 10
 * Bytes: 40
 */
const FLOATS: usize = 10;
const FLOAT_SIZE: usize = size_of::<f32>();

const ID_CIRCLE: f32 = 0.0;
const ID_BOX: f32 = 1.0;
const ID_LINE: f32 = 2.0;

const SHADER_VERT: &str = include_str!("../../resources/shaders/shape_renderer.vert");
const SHADER_GEOM: &str = include_str!("../../resources/shaders/shape_renderer.geom");
const SHADER_FRAG: &str = include_str!("../../resources/shaders/shape_renderer.frag");

#[allow(unused)]
impl ShapeRenderer {
	pub fn new(gl: Rc<Context>, capacity: usize) -> Result<Self, String> {
		unsafe {
			let vec = Vec::with_capacity(capacity);
			let shader = Shader::new(gl.clone())
				.addFromSource(glow::VERTEX_SHADER, SHADER_VERT)
				.addFromSource(glow::GEOMETRY_SHADER, SHADER_GEOM)
				.addFromSource(glow::FRAGMENT_SHADER, SHADER_FRAG)
				.link();
			
			let vao = gl.create_vertex_array().map_err(|e| format!("Failed to create vertex array: {}", e))?;
			let vbo = gl.create_named_buffer().map_err(|e| format!("Failed to create buffer object: {}", e))?;
			gl.bind_vertex_array(Some(vao));
			
			gl.named_buffer_data_size(vbo, (capacity * FLOAT_SIZE) as i32, glow::DYNAMIC_DRAW);
			gl.vertex_array_vertex_buffer(vao, 0, Some(vbo), 0, (FLOATS * FLOAT_SIZE) as i32);
			
			let locId = shader.getAttribLocation("i_shapeId").unwrap();
			let locPos = shader.getAttribLocation("i_position").unwrap();
			let locCol = shader.getAttribLocation("i_color").unwrap();
			let locSize = shader.getAttribLocation("i_size").unwrap();
			let locRot = shader.getAttribLocation("i_rotation").unwrap();
			let locOutline = shader.getAttribLocation("i_outline").unwrap();
			
			let mut offset: usize = 0;
			gl.vertex_array_attrib_format_f32(vao, locId, 1, glow::FLOAT, false, offset as u32);
			gl.vertex_array_attrib_binding_f32(vao, locId, 0);
			offset += FLOAT_SIZE;
			
			gl.vertex_array_attrib_format_f32(vao, locPos, 2, glow::FLOAT, false, offset as u32);
			gl.vertex_array_attrib_binding_f32(vao, locPos, 0);
			offset += 2 * FLOAT_SIZE;
			
			gl.vertex_array_attrib_format_f32(vao, locCol, 3, glow::FLOAT, false, offset as u32);
			gl.vertex_array_attrib_binding_f32(vao, locCol, 0);
			offset += 3 * FLOAT_SIZE;
			
			gl.vertex_array_attrib_format_f32(vao, locSize, 2, glow::FLOAT, false, offset as u32);
			gl.vertex_array_attrib_binding_f32(vao, locSize, 0);
			offset += 2 * FLOAT_SIZE;
			
			gl.vertex_array_attrib_format_f32(vao, locRot, 1, glow::FLOAT, false, offset as u32);
			gl.vertex_array_attrib_binding_f32(vao, locRot, 0);
			offset += FLOAT_SIZE;
			
			gl.vertex_array_attrib_format_f32(vao, locOutline, 1, glow::FLOAT, false, offset as u32);
			gl.vertex_array_attrib_binding_f32(vao, locOutline, 0);
			// offset += FLOAT_SIZE;
			
			gl.enable_vertex_array_attrib(vao, locId);
			gl.enable_vertex_array_attrib(vao, locPos);
			gl.enable_vertex_array_attrib(vao, locCol);
			gl.enable_vertex_array_attrib(vao, locSize);
			gl.enable_vertex_array_attrib(vao, locRot);
			gl.enable_vertex_array_attrib(vao, locOutline);
			
			gl.bind_vertex_array(None);
			
			Ok(ShapeRenderer {
				gl,
				vec,
				shader,
				vao,
				vbo,
				floatsPushed: 0,
				lastFloatsPushed: capacity,
				enabled: true,
				destroyed: false,
			})
		}
	}
	
	pub fn pushCircle(&mut self, pos: Vec2, color: Vec3, radius: f32, outline: f32) {
		if !self.enabled {
			return;
		}
		self.vec.push(ID_CIRCLE);
		
		self.vec.push(pos.x);
		self.vec.push(pos.y);
		
		self.vec.push(color.x);
		self.vec.push(color.y);
		self.vec.push(color.z);
		
		self.vec.push(radius);
		self.vec.push(radius);
		
		self.vec.push(0.0); // rotation
		
		self.vec.push(outline);
		
		self.floatsPushed += FLOATS;
	}
	
	pub fn pushBox(&mut self, pos: Vec2, color: Vec3, size: Vec2, rotation: f32, outline: f32) {
		if !self.enabled {
			return;
		}
		self.vec.push(ID_BOX);
		
		self.vec.push(pos.x);
		self.vec.push(pos.y);
		
		self.vec.push(color.x);
		self.vec.push(color.y);
		self.vec.push(color.z);
		
		self.vec.push(size.x);
		self.vec.push(size.y);
		
		self.vec.push(rotation);
		
		self.vec.push(outline);
		
		self.floatsPushed += FLOATS;
	}
	
	pub fn pushLineWithLength(&mut self, pos: Vec2, color: Vec3, length: f32, thickness: f32, rotation: f32, outline: f32) {
		if !self.enabled {
			return;
		}
		self.vec.push(ID_LINE);
		
		self.vec.push(pos.x);
		self.vec.push(pos.y);
		
		self.vec.push(color.x);
		self.vec.push(color.y);
		self.vec.push(color.z);
		
		self.vec.push(length);
		self.vec.push(thickness);
		
		self.vec.push(rotation);
		
		self.vec.push(outline);
		
		self.floatsPushed += FLOATS;
	}
	
	pub fn pushLineWithPoints(&mut self, p1: Vec2, p2: Vec2, color: Vec3, thickness: f32, outline: f32) {
		if !self.enabled {
			return;
		}
		let delta: Vec2 = p2 - p1;
		
		self.vec.push(ID_LINE);
		
		self.vec.push(p1.x);
		self.vec.push(p1.y);
		
		self.vec.push(color.x);
		self.vec.push(color.y);
		self.vec.push(color.z);
		
		self.vec.push(delta.length());
		self.vec.push(thickness);
		
		self.vec.push(delta.y.atan2(delta.x));
		
		self.vec.push(outline);
		
		self.floatsPushed += FLOATS;
	}
	
	pub fn drawFlush(&mut self, pvMatrix: &Mat4) {
		if !self.enabled || self.vec.len() < FLOATS || self.floatsPushed < FLOATS {
			return;
		}
		
		self.shader.bind();
		self.shader.setMatrix4f("u_pvm", pvMatrix);
		
		unsafe {
			self.gl.bind_vertex_array(Some(self.vao));
			
			if self.floatsPushed > self.lastFloatsPushed {
				self.gl.named_buffer_data_u8_slice(self.vbo, bytemuck::cast_slice(&self.vec), glow::DYNAMIC_DRAW);
			} else {
				self.gl.named_buffer_sub_data_u8_slice(self.vbo, 0, bytemuck::cast_slice(&self.vec));
			}
			
			let drawCount = self.vec.len() / FLOATS;
			// info!("drawCount: {}", drawCount);
			self.gl.draw_arrays(glow::POINTS, 0, drawCount as i32);
			
			self.gl.bind_vertex_array(None);
		}
		
		self.vec.clear();
		self.lastFloatsPushed = self.floatsPushed;
		self.floatsPushed = 0;
	}
	
	pub fn destroy(&mut self) {
		if self.destroyed {
			return;
		}
		info!("Destroying shape renderer");
		self.shader.delete();
		unsafe {
			self.gl.delete_buffer(self.vbo);
			self.gl.delete_vertex_array(self.vao);
		}
		self.destroyed = true;
	}
	
	pub fn getBufferCapacity(&self) -> usize {
		self.vec.capacity()
	}
	
	pub fn getLastFloatsPushed(&self) -> usize {
		self.lastFloatsPushed
	}
}

impl Drop for ShapeRenderer {
	fn drop(&mut self) {
		self.destroy();
	}
}
