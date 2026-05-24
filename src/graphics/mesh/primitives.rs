#![allow(unused)]

use std::f32::consts::{PI, TAU};
use glam::Vec3;
use crate::graphics::mesh::{MeshBuilder, Vertex};

// todo: Chose XYZ plane for 2d
pub struct Primitives2D();
pub struct Primitives3D();

impl Primitives2D {
	pub fn circleXY(segments: usize, diameter: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		let vertMid = Vertex {
			normal: Vec3::Z,
			..Default::default()
		};
		for i in 0..segments {
			let angle = i as f32 * TAU / segments as f32;
			let vertex = Vertex {
				position: Vec3::new(angle.cos(), angle.sin(), 0.0) / 2.0 * diameter,
				normal: Vec3::Z,
				..Default::default()
			};
			
			let angle = ((i + 1) % segments) as f32 * TAU / segments as f32;
			let vertexNext = Vertex {
				position: Vec3::new(angle.cos(), angle.sin(), 0.0) / 2.0 * diameter,
				normal: Vec3::Z,
				..Default::default()
			};
			
			builder.triangleVertices(vertMid, vertex, vertexNext);
		}
		builder
	}
	
	pub fn squareXY(width: f32, height: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		let width = width / 2.0;
		let height = height / 2.0;
		let vertA = Vertex {
			position: Vec3::new(-width, height, 0.0),
			normal: Vec3::Z,
			..Default::default()
		};
		let vertB = Vertex {
			position: Vec3::new(width, height, 0.0),
			normal: Vec3::Z,
			..Default::default()
		};
		let vertC = Vertex {
			position: Vec3::new(width, -height, 0.0),
			normal: Vec3::Z,
			..Default::default()
		};
		let vertD = Vertex {
			position: Vec3::new(-width, -height, 0.0),
			normal: Vec3::Z,
			..Default::default()
		};
		builder.triangleVertices(vertA, vertC, vertB);
		builder.triangleVertices(vertA, vertD, vertC);
		builder
	}
}

impl Primitives3D {
	pub fn tetrahedron(radius: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		builder.vertex(Vertex::withColor(Vec3::new(radius, radius, radius), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(radius, -radius, -radius), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-radius, radius, -radius), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-radius, -radius, radius), Vec3::ZERO, Vec3::ONE).normalPos());
		
		builder.triangleIndices(0, 1, 2);
		builder.triangleIndices(0, 2, 3);
		builder.triangleIndices(0, 3, 1);
		builder.triangleIndices(3, 2, 1);
		
		builder
	}
	
	pub fn cube(w: f32, h: f32, d: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		builder.vertex(Vertex::withColor(Vec3::new(-w, -h, -d), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(w, -h, -d), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(w, h, -d), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-w, h, -d), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-w, -h, d), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(w, -h, d), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(w, h, d), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-w, h, d), Vec3::ZERO, Vec3::ONE).normalPos());
		
		builder.triangleIndices(3, 2, 1); // back
		builder.triangleIndices(3, 1, 0);
		builder.triangleIndices(2, 6, 5); // right
		builder.triangleIndices(2, 5, 1);
		builder.triangleIndices(5, 6, 7); // front
		builder.triangleIndices(5, 7, 4);
		builder.triangleIndices(0, 4, 7); // left
		builder.triangleIndices(0, 7, 3);
		builder.triangleIndices(3, 7, 6); // top
		builder.triangleIndices(3, 6, 2);
		builder.triangleIndices(1, 5, 4); // bottom
		builder.triangleIndices(1, 4, 0);
		
		builder
	}
	
	pub fn sphereCube(radius: f32) -> MeshBuilder {
		Self::cube(1.0, 1.0, 1.0).subdivide().subdivide().subdivide().projectToSphere(radius)
	}
	
	pub fn sphereUV(stacks: usize, slices: usize, radius: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		builder.vertex(Vertex {
			position: Vec3::Y * radius,
			normal: Vec3::Y,
			..Default::default()
		});
		
		for i in 0..(stacks - 1) {
			let phi = PI * ((i + 1) % stacks) as f32 / stacks as f32;
			for j in 0..slices {
				let theta = 2.0 * PI * j as f32 / slices as f32;
				let pos = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
				builder.vertex(Vertex {
					position: pos * radius,
					normal: pos.normalize_or_zero(),
					..Default::default()
				});
			}
		}
		
		builder.vertex(Vertex {
			position: Vec3::NEG_Y * radius,
			normal: Vec3::NEG_Y,
			..Default::default()
		});
		
		let lastIndex = builder.vertices().len() - 1;
		for i in 0..slices {
			let i0 = i + 1;
			let i1 = (i + 1) % slices + 1;
			builder.triangleIndices(0, i1, i0);
			let i0 = i + slices * (stacks - 2) + 1;
			let i1 = (i + 1) % slices + slices * (stacks - 2) + 1;
			builder.triangleIndices(lastIndex, i0, i1);
		}
		
		for j in 0..(stacks - 2) {
			let j0 = j * slices + 1;
			let j1 = (j + 1) * slices + 1;
			for i in 0..slices {
				let i0 = j0 + i;
				let i1 = j0 + (i + 1) % slices;
				let i2 = j1 + (i + 1) % slices;
				let i3 = j1 + i;
				builder.triangleIndices(i0, i1, i2);
				builder.triangleIndices(i0, i2, i3);
			}
		}
		
		builder
	}
	
	// order >= 6 is +1m
	pub fn icosphere(radius: f32, order: u32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
		let (a, b) = (1.0, 1.0 / phi);
		
		builder.vertex(Vertex::withColor(Vec3::new(0.0, b, -a), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(b, a, 0.0), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-b, a, 0.0), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(0.0, b, a), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(0.0, -b, a), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-a, 0.0, b), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(0.0, -b, -a), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(a, 0.0, -b), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(a, 0.0, b), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-a, 0.0, -b), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(b, -a, 0.0), Vec3::ZERO, Vec3::ONE).normalPos());
		builder.vertex(Vertex::withColor(Vec3::new(-b, -a, 0.0), Vec3::ZERO, Vec3::ONE).normalPos());
		
		builder.triangleIndices(2, 1, 0);
		builder.triangleIndices(1, 2, 3);
		builder.triangleIndices(5, 4, 3);
		builder.triangleIndices(4, 8, 3);
		builder.triangleIndices(7, 6, 0);
		builder.triangleIndices(6, 9, 0);
		builder.triangleIndices(11, 10, 4);
		builder.triangleIndices(10, 11, 6);
		builder.triangleIndices(9, 5, 2);
		builder.triangleIndices(5, 9, 11);
		builder.triangleIndices(8, 7, 1);
		builder.triangleIndices(7, 8, 10);
		builder.triangleIndices(2, 5, 3);
		builder.triangleIndices(8, 1, 3);
		builder.triangleIndices(9, 2, 0);
		builder.triangleIndices(1, 7, 0);
		builder.triangleIndices(11, 9, 6);
		builder.triangleIndices(7, 10, 6);
		builder.triangleIndices(5, 11, 4);
		builder.triangleIndices(10, 8, 4);
		
		for _ in 0..order {
			builder.subdivide();
		}
		
		builder.projectToSphere(radius);
		builder
	}
}
