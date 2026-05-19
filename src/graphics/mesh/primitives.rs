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
		
		builder.vertex(Vertex::autoNormal(Vec3::new(radius, radius, radius), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(radius, -radius, -radius), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(-radius, radius, -radius), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(-radius, -radius, radius), Vec3::ONE));
		
		builder.triangleIndices(0, 1, 2);
		builder.triangleIndices(0, 2, 3);
		builder.triangleIndices(0, 3, 1);
		builder.triangleIndices(3, 2, 1);
		
		builder
	}
	
	pub fn cube(w: f32, h: f32, d: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		builder.vertex(Vertex::autoNormal(Vec3::new(-w, -h, -d), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(w, -h, -d), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(w, h, -d), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(-w, h, -d), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(-w, -h, d), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(w, -h, d), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(w, h, d), Vec3::ONE));
		builder.vertex(Vertex::autoNormal(Vec3::new(-w, h, d), Vec3::ONE));
		
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
}
