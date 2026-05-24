use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::types::GlRef;

#[derive(Debug, Clone, PartialEq)]
pub struct VertexRef(Rc<RefCell<Vertex>>);

impl From<Vertex> for VertexRef {
	fn from(vertex: Vertex) -> Self {
		VertexRef(Rc::new(RefCell::new(vertex)))
	}
}

impl Deref for VertexRef {
	type Target = Rc<RefCell<Vertex>>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Hash for VertexRef {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.borrow().hash(state);
	}
}

#[derive(Debug, Clone)]
pub struct MeshBuilder {
	vertices: Vec<VertexRef>,
	triangles: HashSet<Triangle>,
}

#[allow(unused)]
impl MeshBuilder {
	pub fn new() -> Self {
		Self {
			vertices: Vec::with_capacity(3),
			triangles: HashSet::with_capacity(1),
		}
	}
	
	pub fn vertices(&self) -> &Vec<VertexRef> {
		&self.vertices
	}
	
	pub fn triangles(&self) -> &HashSet<Triangle> {
		&self.triangles
	}
	
	pub fn findVertex(&self, vertex: &Vertex) -> Option<VertexRef> {
		self.vertices.iter().find(|v| *v.borrow() == *vertex).cloned()
	}
	
	pub fn findVertexIndex(&self, vertex: &Vertex) -> Option<usize> {
		self.vertices.iter().rposition(|v| *v.borrow() == *vertex)
	}
	
	pub fn vertex(&mut self, vertex: Vertex) -> VertexRef {
		let found = self.findVertex(&vertex);
		if found.is_none() {
			let vertex = VertexRef::from(vertex);
			self.vertices.push(vertex.clone());
			return vertex;
		}
		found.unwrap()
	}
	
	pub fn findTriangle(&self, triangle: &Triangle) -> Option<Triangle> {
		self.triangles.get(triangle).cloned()
	}
	
	pub fn triangleIndices(&mut self, i0: usize, i1: usize, i2: usize) -> Triangle {
		let triangle = Triangle(i0, i1, i2);
		let found = self.findTriangle(&triangle);
		if found.is_none() {
			self.triangles.insert(triangle.clone());
			return triangle;
		}
		found.unwrap()
	}
	
	pub fn triangleVertices(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) -> Triangle {
		self.vertex(v0);
		self.vertex(v1);
		self.vertex(v2);
		self.triangleIndices(self.findVertexIndex(&v0).unwrap(), self.findVertexIndex(&v1).unwrap(), self.findVertexIndex(&v2).unwrap())
	}
	
	pub fn subdivideTriangle(&mut self, triangle: &Triangle) -> MeshBuilder {
		self.triangles.remove(&triangle);
		let a = self.vertices[triangle.0].borrow().clone();
		let b = self.vertices[triangle.1].borrow().clone();
		let c = self.vertices[triangle.2].borrow().clone();
		
		let ab = self.vertex(Vertex {
			position: (a.position + b.position) / 2.0,
			normal: (a.normal + b.normal) / 2.0,
			color: (a.color + b.color) / 2.0,
			uv: (a.uv + b.uv) / 2.0,
		});
		let bc = self.vertex(Vertex {
			position: (b.position + c.position) / 2.0,
			normal: (b.normal + c.normal) / 2.0,
			color: (b.color + c.color) / 2.0,
			uv: (b.uv + c.uv) / 2.0,
		});
		let ac = self.vertex(Vertex {
			position: (a.position + c.position) / 2.0,
			normal: (a.normal + c.normal) / 2.0,
			color: (a.color + c.color) / 2.0,
			uv: (a.uv + c.uv) / 2.0,
		});
		
		let ab = self.findVertexIndex(&*ab.borrow()).unwrap();
		let bc = self.findVertexIndex(&*bc.borrow()).unwrap();
		let ac = self.findVertexIndex(&*ac.borrow()).unwrap();
		
		let a = triangle.0;
		let b = triangle.1;
		let c = triangle.2;
		
		self.triangleIndices(a, ab, ac);
		self.triangleIndices(b, bc, ab);
		self.triangleIndices(c, ac, bc);
		self.triangleIndices(ab, bc, ac);
		self.to_owned()
	}
	
	pub fn subdivide(&mut self) -> MeshBuilder {
		let triangles = self.triangles.clone();
		for tri in triangles.iter() {
			self.subdivideTriangle(tri);
		}
		self.to_owned()
	}
	
	pub fn dual(&mut self) -> MeshBuilder {
		// todo https://danielsieger.com/blog/2021/01/03/generating-platonic-solids.html
		self.to_owned()
	}
	
	pub fn centroid(&mut self, triangle: &Triangle) -> MeshBuilder {
		// todo
		// need to find a way to calculate for triangles on same plane
		self.to_owned()
	}
	
	pub fn projectToSphere(&mut self, radius: f32) -> MeshBuilder {
		for v in self.vertices.iter() {
			let mut borrow = v.borrow_mut();
			let p = borrow.position;
			let n = p.length();
			borrow.position = (1.0 / n) * p * radius;
			borrow.normal = borrow.position.normalize_or_zero();
		}
		self.to_owned()
	}
	
	fn buildData(&self) -> (Vec<Vertex>, Option<Vec<u32>>) {
		let indices = if self.triangles.is_empty() {
			None
		} else {
			let mut indices: Vec<u32> = Vec::new();
			for triangle in self.triangles.iter() {
				indices.push(triangle.0 as u32);
				indices.push(triangle.1 as u32);
				indices.push(triangle.2 as u32);
			}
			Some(indices)
		};
		
		(self.vertices.iter().map(|v| *v.borrow()).collect(), indices)
	}
	
	pub fn buildSimpleMesh(&self, gl: GlRef) -> Mesh {
		let data = self.buildData();
		Mesh::simple(gl, data.0, data.1)
	}
	
	pub fn buildInstanceMesh(&self, gl: GlRef) -> Mesh {
		let data = self.buildData();
		Mesh::instance(gl, data.0, data.1)
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle(pub usize, pub usize, pub usize);

impl Eq for Triangle {}

impl Hash for Triangle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}
