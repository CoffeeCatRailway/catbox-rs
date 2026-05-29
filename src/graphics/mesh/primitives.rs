#![allow(unused)]

use std::f32::consts::{PI, TAU};
use glam::{Vec2, Vec3};
use crate::graphics::mesh::{MeshBuilder, Vertex};

// todo: Chose XYZ plane for 2d
pub struct Primitives2D();
pub struct Primitives3D();

impl Primitives2D {
	pub fn circleXY(segments: usize, diameter: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		let vertMid = Vertex::withUV(
			Vec3::ZERO,
			Vec3::Z,
			Vec2::ONE / 2.0,
		);
		for i in 0..segments {
			let angle = i as f32 * TAU / segments as f32;
			let pos = Vec3::new(angle.cos(), angle.sin(), 0.0) / 2.0;
			let vertex = Vertex::withUV(
				pos * diameter,
				Vec3::Z,
				pos.truncate() * Vec2::new(1.0, -1.0) + 0.5,
			);
			
			let angle = ((i + 1) % segments) as f32 * TAU / segments as f32;
			let pos = Vec3::new(angle.cos(), angle.sin(), 0.0) / 2.0;
			let vertexNext = Vertex::withUV(
				pos * diameter,
				Vec3::Z,
				pos.truncate() * Vec2::new(1.0, -1.0) + 0.5,
			);
			
			builder.triangleVertices(vertMid, vertex, vertexNext);
		}
		builder
	}
	
	pub fn squareXY(width: f32, height: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		let width = width / 2.0;
		let height = height / 2.0;
		let vertA = Vertex::withUV(
			Vec3::new(-width, height, 0.0),
			Vec3::Z,
			Vec2::new(0.0, 0.0),
		);
		let vertB = Vertex::withUV(
			Vec3::new(width, height, 0.0),
			Vec3::Z,
			Vec2::new(1.0, 0.0),
		);
		let vertC = Vertex::withUV(
			Vec3::new(width, -height, 0.0),
			Vec3::Z,
			Vec2::new(1.0, 1.0),
		);
		let vertD = Vertex::withUV(
			Vec3::new(-width, -height, 0.0),
			Vec3::Z,
			Vec2::new(0.0, 1.0),
		);
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
		
		let p0 = Vec3::new(-w, -h, -d);
		let p1 = Vec3::new(w, -h, -d);
		let p2 = Vec3::new(w, h, -d);
		let p3 = Vec3::new(-w, h, -d);
		let p4 = Vec3::new(-w, -h, d);
		let p5 = Vec3::new(w, -h, d);
		let p6 = Vec3::new(w, h, d);
		let p7 = Vec3::new(-w, h, d);
		
		let uv00 = Vec2::new(0.0, 0.0);
		let uv10 = Vec2::new(1.0, 0.0);
		let uv11 = Vec2::new(1.0, 1.0);
		let uv01 = Vec2::new(0.0, 1.0);
		
		// back
		builder.triangleVertices(
			Vertex::withUV(p3, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p2, Vec3::ZERO, uv00).normalPos(),
			Vertex::withUV(p1, Vec3::ZERO, uv01).normalPos(),
		);
		builder.triangleVertices(
			Vertex::withUV(p3, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p1, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p0, Vec3::ZERO, uv11).normalPos(),
		);
		// right
		builder.triangleVertices(
			Vertex::withUV(p2, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p6, Vec3::ZERO, uv00).normalPos(),
			Vertex::withUV(p5, Vec3::ZERO, uv01).normalPos(),
		);
		builder.triangleVertices(
			Vertex::withUV(p2, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p5, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p1, Vec3::ZERO, uv11).normalPos(),
		);
		// front
		builder.triangleVertices(
			Vertex::withUV(p6, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p7, Vec3::ZERO, uv00).normalPos(),
			Vertex::withUV(p4, Vec3::ZERO, uv01).normalPos(),
		);
		builder.triangleVertices(
			Vertex::withUV(p6, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p4, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p5, Vec3::ZERO, uv11).normalPos(),
		);
		// left
		builder.triangleVertices(
			Vertex::withUV(p0, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p4, Vec3::ZERO, uv11).normalPos(),
			Vertex::withUV(p7, Vec3::ZERO, uv10).normalPos(),
		);
		builder.triangleVertices(
			Vertex::withUV(p0, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p7, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p3, Vec3::ZERO, uv00).normalPos(),
		);
		// top
		builder.triangleVertices(
			Vertex::withUV(p7, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p2, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p3, Vec3::ZERO, uv00).normalPos(),
		);
		builder.triangleVertices(
			Vertex::withUV(p7, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p6, Vec3::ZERO, uv11).normalPos(),
			Vertex::withUV(p2, Vec3::ZERO, uv10).normalPos(),
		);
		// bottom
		builder.triangleVertices(
			Vertex::withUV(p5, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p0, Vec3::ZERO, uv01).normalPos(),
			Vertex::withUV(p1, Vec3::ZERO, uv11).normalPos(),
		);
		builder.triangleVertices(
			Vertex::withUV(p5, Vec3::ZERO, uv10).normalPos(),
			Vertex::withUV(p4, Vec3::ZERO, uv00).normalPos(),
			Vertex::withUV(p0, Vec3::ZERO, uv01).normalPos(),
		);
		
		builder
	}
	
	pub fn sphereCube(radius: f32, order: u32) -> MeshBuilder {
		let mut builder = Self::cube(1.0, 1.0, 1.0);
		
		for _ in 0..order {
			builder.subdivide();
		}
		
		builder.projectToSphere(radius);
		builder
	}
	
	// https://danielsieger.com/blog/2021/03/27/generating-spheres.html
	pub fn sphereUV(stacks: usize, slices: usize, radius: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		builder.vertex(Vertex::withUV(
			Vec3::Y * radius,
			Vec3::Y,
			Vec2::new(0.0, 0.0),
		));
		
		for i in 0..(stacks - 1) {
			let phi = PI * ((i + 1) % stacks) as f32 / stacks as f32;
			for j in 0..slices + 1 {
				let theta = 2.0 * PI * j as f32 / slices as f32;
				let pos = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
				
				let mut uv = Vec2::ZERO;
				uv.x = j as f32 / slices as f32;
				uv.y = (i + 1) as f32 / stacks as f32;
				if j == slices {
					uv.x = 1.0;
				}
				
				builder.vertex(Vertex::withUV(
					pos * radius,
					pos.normalize_or_zero(),
					uv,
				));
			}
		}
		
		builder.vertex(Vertex::withUV(
			Vec3::NEG_Y * radius,
			Vec3::NEG_Y,
			Vec2::new(0.0, 1.0),
		));
		
		let slices = slices + 1;
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
		
		let H_ANGLE = PI / 180.0 * 72.0;
		let V_ANGLE = (1.0_f32 / 2.0).atan();
		
		let mut hAngle1 = -PI / 2.0 - H_ANGLE / 2.0;
		let mut hAngle2 = -PI / 2.0;
		
		let mut tmpVertices = Vec::new();
		tmpVertices.push(Vec3::new(0.0, 0.0, radius));
		
		for _ in 0..6 {
			let z = radius * V_ANGLE.sin();
			let xy = radius * V_ANGLE.cos();
			
			tmpVertices.push(Vec3::new(xy * hAngle1.cos(), xy * hAngle1.sin(), z));
			tmpVertices.push(Vec3::new(xy * hAngle2.cos(), xy * hAngle2.sin(), -z));
			
			hAngle1 += H_ANGLE;
			hAngle2 += H_ANGLE;
		}
		
		tmpVertices.push(Vec3::new(0.0, 0.0, -radius));
		
		let S_STEP: f32 = 186.0 / 2048.0;
		let T_STEP: f32 = 322.0 / 1024.0;
		
		// smooth icosahedron has 14 non-shared (0 to 13) and
		// 8 shared vertices (14 to 21) (total 22 vertices)
		//  00  01  02  03  04          //
		//  /\  /\  /\  /\  /\          //
		// /  \/  \/  \/  \/  \         //
		//10--14--15--16--17--11        //
		// \  /\  /\  /\  /\  /\        //
		//  \/  \/  \/  \/  \/  \       //
		//  12--18--19--20--21--13      //
		//   \  /\  /\  /\  /\  /       //
		//    \/  \/  \/  \/  \/        //
		//    05  06  07  08  09        //
		// add 14 non-shared vertices first (index from 0 to 13)
		
		// top
		builder.vertex(Vertex::withUV(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP, 0.0)).normalPos()); // v0
		builder.vertex(Vertex::withUV(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 3.0, 0.0)).normalPos()); // v1
		builder.vertex(Vertex::withUV(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 5.0, 0.0)).normalPos()); // v2
		builder.vertex(Vertex::withUV(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 7.0, 0.0)).normalPos()); // v3
		builder.vertex(Vertex::withUV(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 9.0, 0.0)).normalPos()); // v4
		
		// bottom
		builder.vertex(Vertex::withUV(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 2.0, T_STEP * 3.0)).normalPos()); // v5
		builder.vertex(Vertex::withUV(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 4.0, T_STEP * 3.0)).normalPos()); // v6
		builder.vertex(Vertex::withUV(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 6.0, T_STEP * 3.0)).normalPos()); // v7
		builder.vertex(Vertex::withUV(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 8.0, T_STEP * 3.0)).normalPos()); // v8
		builder.vertex(Vertex::withUV(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 10.0, T_STEP * 3.0)).normalPos()); // v9
		
		builder.vertex(Vertex::withUV(tmpVertices[1], Vec3::ZERO, Vec2::new(0.0, T_STEP)).normalPos()); // v10 (left)
		builder.vertex(Vertex::withUV(tmpVertices[11], Vec3::ZERO, Vec2::new(S_STEP * 10.0, T_STEP)).normalPos()); // v11 (right)
		
		builder.vertex(Vertex::withUV(tmpVertices[2], Vec3::ZERO, Vec2::new(S_STEP, T_STEP * 2.0)).normalPos()); // v12 (left)
		builder.vertex(Vertex::withUV(tmpVertices[12], Vec3::ZERO, Vec2::new(S_STEP * 11.0, T_STEP * 2.0)).normalPos()); // v13 (right)
		
		builder.vertex(Vertex::withUV(tmpVertices[3], Vec3::ZERO, Vec2::new(S_STEP * 2.0, T_STEP)).normalPos()); // v14 (shared)
		builder.vertex(Vertex::withUV(tmpVertices[5], Vec3::ZERO, Vec2::new(S_STEP * 4.0, T_STEP)).normalPos()); // v15 (shared)
		builder.vertex(Vertex::withUV(tmpVertices[7], Vec3::ZERO, Vec2::new(S_STEP * 6.0, T_STEP)).normalPos()); // v16 (shared)
		builder.vertex(Vertex::withUV(tmpVertices[9], Vec3::ZERO, Vec2::new(S_STEP * 8.0, T_STEP)).normalPos()); // v17 (shared)
		
		builder.vertex(Vertex::withUV(tmpVertices[4], Vec3::ZERO, Vec2::new(S_STEP * 3.0, T_STEP * 2.0)).normalPos()); // v18 (shared)
		builder.vertex(Vertex::withUV(tmpVertices[6], Vec3::ZERO, Vec2::new(S_STEP * 5.0, T_STEP * 2.0)).normalPos()); // v19 (shared)
		builder.vertex(Vertex::withUV(tmpVertices[8], Vec3::ZERO, Vec2::new(S_STEP * 7.0, T_STEP * 2.0)).normalPos()); // 20 (shared)
		builder.vertex(Vertex::withUV(tmpVertices[10], Vec3::ZERO, Vec2::new(S_STEP * 9.0, T_STEP * 2.0)).normalPos()); // 21 (shared)
		
		// 1st row, 5 tris
		builder.triangleIndices(0, 10, 14);
		builder.triangleIndices(1, 14, 15);
		builder.triangleIndices(2, 15, 16);
		builder.triangleIndices(3, 16, 17);
		builder.triangleIndices(4, 17, 11);
		// 2nd row, 10 tris
		builder.triangleIndices(10, 12, 14);
		builder.triangleIndices(12, 18, 14);
		builder.triangleIndices(14, 18, 15);
		builder.triangleIndices(18, 19, 15);
		builder.triangleIndices(15, 19, 16);
		builder.triangleIndices(19, 20, 16);
		builder.triangleIndices(16, 20, 17);
		builder.triangleIndices(20, 21, 17);
		builder.triangleIndices(17, 21, 11);
		builder.triangleIndices(21, 13, 11);
		// 3rd row, 5 tris
		builder.triangleIndices(5, 18, 12);
		builder.triangleIndices(6, 19, 18);
		builder.triangleIndices(7, 20, 19);
		builder.triangleIndices(8, 21, 20);
		builder.triangleIndices(9, 13, 21);
		
		for _ in 0..order {
			builder.subdivide();
		}
		
		builder.projectToSphere(radius);
		builder
	}
}
