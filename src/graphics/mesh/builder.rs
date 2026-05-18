use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::types::GlRef;

pub struct MeshBuilder {
	vertices: HashMap<Vertex, Rc<Cell<Vertex>>>,
	indexMap: Vec<Rc<Cell<Vertex>>>,
	triangles: HashSet<Triangle>,
}

impl MeshBuilder {
	pub fn new() -> Self {
		Self {
			vertices: HashMap::with_capacity(3),
			indexMap: Vec::with_capacity(3),
			triangles: HashSet::with_capacity(1),
		}
	}
	
	pub fn vertices(&self) -> &Vec<Rc<Cell<Vertex>>> {
		&self.indexMap
	}
	
	pub fn triangles(&self) -> &HashSet<Triangle> {
		&self.triangles
	}
	
	pub fn vertex(&mut self, vertex: Vertex) -> &mut MeshBuilder {
		if !self.vertices.contains_key(&vertex) {
			let rc = Rc::new(Cell::new(vertex));
			self.vertices.insert(vertex, rc.clone());
			self.indexMap.push(rc.clone());
		}
		self
	}
	
	pub fn triangleFromIndices(&mut self, i0: usize, i1: usize, i2: usize) -> (&mut MeshBuilder, Triangle) {
		let v0 = self.indexMap[i0].clone();
		let v1 = self.indexMap[i1].clone();
		let v2 = self.indexMap[i2].clone();
		let triangle = Triangle(v0, v1, v2);
		self.triangles.insert(triangle.clone());
		(self, triangle)
	}
	
	pub fn triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) -> (&mut MeshBuilder, Triangle) {
		self.vertex(v0).vertex(v1).vertex(v2);
		let triangle = Triangle(self.vertices[&v0].clone(), self.vertices[&v1].clone(), self.vertices[&v2].clone());
		self.triangles.insert(triangle.clone());
		(self, triangle)
	}
	
	pub fn subdivideTriangle(&mut self, triangle: &Triangle) -> &mut MeshBuilder {
		self.triangles.remove(&triangle);
		let triangles = triangle.subdivide();
		for tri in triangles {
			self.triangle(tri.0.get(), tri.1.get(), tri.2.get());
		}
		self
	}
	
	fn buildData(&self) -> (Vec<Vertex>, Option<Vec<u32>>) {
		let indexMap: HashMap<Vertex, u32> = self.indexMap.iter().cloned().enumerate().map(|(i, v)| (v.get(), i as u32)).collect();
		
		let indices = if self.triangles.is_empty() {
			None
		} else {
			let mut indices: Vec<u32> = Vec::new();
			for triangle in self.triangles.iter() {
				indices.push(*indexMap.get(&triangle.0.get()).unwrap_or_else(|| &0));
				indices.push(*indexMap.get(&triangle.1.get()).unwrap_or_else(|| &0));
				indices.push(*indexMap.get(&triangle.2.get()).unwrap_or_else(|| &0));
			}
			Some(indices)
		};
		
		(self.indexMap.iter().map(|rc| rc.get()).collect(), indices)
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

#[derive(Debug, Clone, PartialEq)]
pub struct Triangle(pub Rc<Cell<Vertex>>, pub Rc<Cell<Vertex>>, pub Rc<Cell<Vertex>>);

impl Triangle {
    pub fn subdivide(&self) -> [Triangle; 4] {
        let a = self.0.clone().get();
        let b = self.1.clone().get();
        let c = self.2.clone().get();
        let ab = Rc::new(Cell::new(Vertex {
            position: (a.position + b.position) / 2.0,
            normal: (a.normal + b.normal) / 2.0,
            color: (a.color + b.color) / 2.0,
        }));
        let ac = Rc::new(Cell::new(Vertex {
            position: (a.position + c.position) / 2.0,
            normal: (a.normal + c.normal) / 2.0,
            color: (a.color + c.color) / 2.0,
        }));
        let bc = Rc::new(Cell::new(Vertex {
            position: (b.position + c.position) / 2.0,
            normal: (b.normal + c.normal) / 2.0,
            color: (b.color + c.color) / 2.0,
        }));
        [
            Triangle(self.0.clone(), ab.clone(), ac.clone()),
            Triangle(ac.clone(), bc.clone(), self.2.clone()),
            Triangle(ab.clone(), self.1.clone(), bc.clone()),
            Triangle(ab.clone(), bc.clone(), ac.clone()),
        ]
    }
}

impl Eq for Triangle {}

impl Hash for Triangle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.get().hash(state);
        self.1.get().hash(state);
        self.2.get().hash(state);
    }
}