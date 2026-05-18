use std::collections::{HashMap, HashSet};
use crate::graphics::mesh::{Mesh, Triangle, Vertex};
use crate::types::GlRef;

pub struct MeshBuilder {
	vertices: HashSet<Vertex>,
	triangles: HashSet<Triangle>,
}

impl MeshBuilder {
	pub fn new() -> Self {
		Self {
			vertices: HashSet::with_capacity(3),
			triangles: HashSet::with_capacity(1),
		}
	}
	
	pub fn vertexCount(&self) -> usize {
		self.vertices.len()
	}
	
	pub fn vertex(&mut self, vertex: Vertex) -> &mut MeshBuilder {
		self.vertices.insert(vertex);
		self
	}
	
	pub fn triangleFromIndices(&mut self, i0: usize, i1: usize, i2: usize) -> (&mut MeshBuilder, Triangle) {
		let (v0, v1, v2) = {
			let vec = self.vertices.iter().cloned().collect::<Vec<_>>();
			(vec[i0], vec[i1], vec[i2])
		};
		let triangle = Triangle(v0, v1, v2);
		self.triangles.insert(triangle);
		(self, triangle)
	}
	
	pub fn triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) -> &mut MeshBuilder {
		let triangle = Triangle(v0, v1, v2);
		self.triangles.insert(triangle);
		self.vertex(triangle.0).vertex(triangle.1).vertex(triangle.2)
	}
	
	pub fn subdivideTriangle(&mut self, triangle: &Triangle) -> &mut MeshBuilder {
		self.triangles.remove(&triangle);
		let triangles = triangle.subdivide();
		for tri in triangles {
			self.triangles.insert(tri);
			self.vertex(tri.0).vertex(tri.1).vertex(tri.2);
		}
		self
	}
	
	fn buildData(&self) -> (Vec<Vertex>, Vec<u32>) {
		let indexMap: HashMap<Vertex, u32> = self.vertices.iter().cloned().enumerate().map(|(i, v)| (v, i as u32)).collect();
		
		let mut indices: Vec<u32> = Vec::new();
		for triangle in self.triangles.iter() {
			indices.push(*indexMap.get(&triangle.0).unwrap_or_else(|| &0));
			indices.push(*indexMap.get(&triangle.1).unwrap_or_else(|| &0));
			indices.push(*indexMap.get(&triangle.2).unwrap_or_else(|| &0));
		}
		
		(self.vertices.iter().cloned().collect(), indices)
	}
	
	pub fn buildSimpleMesh(&self, gl: GlRef) -> Mesh {
		let data = self.buildData();
		Mesh::simple(gl, data.0, Some(data.1))
	}
	
	pub fn buildInstanceMesh(&self, gl: GlRef) -> Mesh {
		let data = self.buildData();
		Mesh::instance(gl, data.0, Some(data.1))
	}
}
