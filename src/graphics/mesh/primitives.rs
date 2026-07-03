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
		let vertMid = Vertex::new(
			Vec3::ZERO,
			Vec3::Z,
			Vec2::ONE / 2.0,
		);
		for i in 0..segments {
			let angle = i as f32 * TAU / segments as f32;
			let pos = Vec3::new(angle.cos(), angle.sin(), 0.0) / 2.0;
			let vertex = Vertex::new(
				pos * diameter,
				Vec3::Z,
				pos.truncate() * Vec2::new(1.0, -1.0) + 0.5,
			);
			
			let angle = ((i + 1) % segments) as f32 * TAU / segments as f32;
			let pos = Vec3::new(angle.cos(), angle.sin(), 0.0) / 2.0;
			let vertexNext = Vertex::new(
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
		let vertA = Vertex::new(
			Vec3::new(-width, height, 0.0),
			Vec3::Z,
			Vec2::new(0.0, 0.0),
		);
		let vertB = Vertex::new(
			Vec3::new(width, height, 0.0),
			Vec3::Z,
			Vec2::new(1.0, 0.0),
		);
		let vertC = Vertex::new(
			Vec3::new(width, -height, 0.0),
			Vec3::Z,
			Vec2::new(1.0, 1.0),
		);
		let vertD = Vertex::new(
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
	pub fn tetrahedron(diameter: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		let radius = diameter / 2.0;
		
		builder.vertex(Vertex::new(Vec3::new(radius, radius, radius), Vec3::ZERO, Vec2::ZERO).normalFromPosition());
		builder.vertex(Vertex::new(Vec3::new(radius, -radius, -radius), Vec3::ZERO, Vec2::new(1.0, 0.0)).normalFromPosition());
		builder.vertex(Vertex::new(Vec3::new(-radius, radius, -radius), Vec3::ZERO, Vec2::new(0.0, 1.0)).normalFromPosition());
		builder.vertex(Vertex::new(Vec3::new(-radius, -radius, radius), Vec3::ZERO, Vec2::ONE).normalFromPosition());
		
		builder.triangleIndices(0, 1, 2);
		builder.triangleIndices(0, 2, 3);
		builder.triangleIndices(0, 3, 1);
		builder.triangleIndices(3, 2, 1);
		
		builder
	}
	
	pub fn cube(w: f32, h: f32, d: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		let w = w / 2.0;
		let h = h / 2.0;
		let d = d / 2.0;
		
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
			Vertex::new(p3, Vec3::NEG_Z, uv10),//.normalFromPosition(),
			Vertex::new(p2, Vec3::NEG_Z, uv00),//.normalFromPosition(),
			Vertex::new(p1, Vec3::NEG_Z, uv01),//.normalFromPosition(),
		);
		builder.triangleVertices(
			Vertex::new(p3, Vec3::NEG_Z, uv10),//.normalFromPosition(),
			Vertex::new(p1, Vec3::NEG_Z, uv01),//.normalFromPosition(),
			Vertex::new(p0, Vec3::NEG_Z, uv11),//.normalFromPosition(),
		);
		// right
		builder.triangleVertices(
			Vertex::new(p2, Vec3::X, uv10),//.normalFromPosition(),
			Vertex::new(p6, Vec3::X, uv00),//.normalFromPosition(),
			Vertex::new(p5, Vec3::X, uv01),//.normalFromPosition(),
		);
		builder.triangleVertices(
			Vertex::new(p2, Vec3::X, uv10),//.normalFromPosition(),
			Vertex::new(p5, Vec3::X, uv01),//.normalFromPosition(),
			Vertex::new(p1, Vec3::X, uv11),//.normalFromPosition(),
		);
		// front
		builder.triangleVertices(
			Vertex::new(p6, Vec3::Z, uv10),//.normalFromPosition(),
			Vertex::new(p7, Vec3::Z, uv00),//.normalFromPosition(),
			Vertex::new(p4, Vec3::Z, uv01),//.normalFromPosition(),
		);
		builder.triangleVertices(
			Vertex::new(p6, Vec3::Z, uv10),//.normalFromPosition(),
			Vertex::new(p4, Vec3::Z, uv01),//.normalFromPosition(),
			Vertex::new(p5, Vec3::Z, uv11),//.normalFromPosition(),
		);
		// left
		builder.triangleVertices(
			Vertex::new(p0, Vec3::NEG_X, uv01),//.normalFromPosition(),
			Vertex::new(p4, Vec3::NEG_X, uv11),//.normalFromPosition(),
			Vertex::new(p7, Vec3::NEG_X, uv10),//.normalFromPosition(),
		);
		builder.triangleVertices(
			Vertex::new(p0, Vec3::NEG_X, uv01),//.normalFromPosition(),
			Vertex::new(p7, Vec3::NEG_X, uv10),//.normalFromPosition(),
			Vertex::new(p3, Vec3::NEG_X, uv00),//.normalFromPosition(),
		);
		// top
		builder.triangleVertices(
			Vertex::new(p7, Vec3::Y, uv01),//.normalFromPosition(),
			Vertex::new(p2, Vec3::Y, uv10),//.normalFromPosition(),
			Vertex::new(p3, Vec3::Y, uv00),//.normalFromPosition(),
		);
		builder.triangleVertices(
			Vertex::new(p7, Vec3::Y, uv01),//.normalFromPosition(),
			Vertex::new(p6, Vec3::Y, uv11),//.normalFromPosition(),
			Vertex::new(p2, Vec3::Y, uv10),//.normalFromPosition(),
		);
		// bottom
		builder.triangleVertices(
			Vertex::new(p5, Vec3::NEG_Y, uv10),//.normalFromPosition(),
			Vertex::new(p0, Vec3::NEG_Y, uv01),//.normalFromPosition(),
			Vertex::new(p1, Vec3::NEG_Y, uv11),//.normalFromPosition(),
		);
		builder.triangleVertices(
			Vertex::new(p5, Vec3::NEG_Y, uv10),//.normalFromPosition(),
			Vertex::new(p4, Vec3::NEG_Y, uv00),//.normalFromPosition(),
			Vertex::new(p0, Vec3::NEG_Y, uv01),//.normalFromPosition(),
		);
		
		builder
	}
	
	pub fn sphereCube(diameter: f32, order: u32) -> MeshBuilder {
		let mut builder = Self::cube(1.0, 1.0, 1.0);
		
		let radius = diameter / 2.0;
		
		for _ in 0..order {
			builder.subdivide();
		}
		
		builder.projectToSphere(radius);
		builder
	}
	
	// https://danielsieger.com/blog/2021/03/27/generating-spheres.html
	pub fn sphereUV(stacks: usize, slices: usize, diameter: f32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		let radius = diameter / 2.0;
		
		builder.vertex(Vertex::new(
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
				
				builder.vertex(Vertex::new(
					pos * radius,
					pos.normalize_or_zero(),
					uv,
				));
			}
		}
		
		builder.vertex(Vertex::new(
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
	pub fn icosphere(diameter: f32, order: u32) -> MeshBuilder {
		let mut builder = MeshBuilder::new();
		
		let radius = diameter / 2.0;
		
		let H_ANGLE = PI / 180.0 * 72.0;
		let V_ANGLE = (1.0_f32 / 2.0).atan();
		
		let mut hAngle1 = -PI / 2.0 - H_ANGLE / 2.0;
		let mut hAngle2 = -PI / 2.0;
		
		let mut tmpVertices = Vec::new();
		tmpVertices.push(Vec3::new(0.0, radius, 0.0));
		
		for _ in 0..6 {
			let y = radius * V_ANGLE.sin();
			let xz = radius * V_ANGLE.cos();
			
			tmpVertices.push(Vec3::new(xz * hAngle1.sin(), y, xz * hAngle1.cos()));
			tmpVertices.push(Vec3::new(xz * hAngle2.sin(), -y, xz * hAngle2.cos()));
			
			hAngle1 += H_ANGLE;
			hAngle2 += H_ANGLE;
		}
		
		tmpVertices.push(Vec3::new(0.0, -radius, 0.0));
		
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
		builder.vertex(Vertex::new(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP, 0.0)).normalFromPosition()); // v0
		builder.vertex(Vertex::new(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 3.0, 0.0)).normalFromPosition()); // v1
		builder.vertex(Vertex::new(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 5.0, 0.0)).normalFromPosition()); // v2
		builder.vertex(Vertex::new(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 7.0, 0.0)).normalFromPosition()); // v3
		builder.vertex(Vertex::new(tmpVertices[0], Vec3::ZERO, Vec2::new(S_STEP * 9.0, 0.0)).normalFromPosition()); // v4
		
		// bottom
		builder.vertex(Vertex::new(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 2.0, T_STEP * 3.0)).normalFromPosition()); // v5
		builder.vertex(Vertex::new(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 4.0, T_STEP * 3.0)).normalFromPosition()); // v6
		builder.vertex(Vertex::new(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 6.0, T_STEP * 3.0)).normalFromPosition()); // v7
		builder.vertex(Vertex::new(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 8.0, T_STEP * 3.0)).normalFromPosition()); // v8
		builder.vertex(Vertex::new(tmpVertices[13], Vec3::ZERO, Vec2::new(S_STEP * 10.0, T_STEP * 3.0)).normalFromPosition()); // v9
		
		builder.vertex(Vertex::new(tmpVertices[1], Vec3::ZERO, Vec2::new(0.0, T_STEP)).normalFromPosition()); // v10 (left)
		builder.vertex(Vertex::new(tmpVertices[11], Vec3::ZERO, Vec2::new(S_STEP * 10.0, T_STEP)).normalFromPosition()); // v11 (right)
		
		builder.vertex(Vertex::new(tmpVertices[2], Vec3::ZERO, Vec2::new(S_STEP, T_STEP * 2.0)).normalFromPosition()); // v12 (left)
		builder.vertex(Vertex::new(tmpVertices[12], Vec3::ZERO, Vec2::new(S_STEP * 11.0, T_STEP * 2.0)).normalFromPosition()); // v13 (right)
		
		builder.vertex(Vertex::new(tmpVertices[3], Vec3::ZERO, Vec2::new(S_STEP * 2.0, T_STEP)).normalFromPosition()); // v14 (shared)
		builder.vertex(Vertex::new(tmpVertices[5], Vec3::ZERO, Vec2::new(S_STEP * 4.0, T_STEP)).normalFromPosition()); // v15 (shared)
		builder.vertex(Vertex::new(tmpVertices[7], Vec3::ZERO, Vec2::new(S_STEP * 6.0, T_STEP)).normalFromPosition()); // v16 (shared)
		builder.vertex(Vertex::new(tmpVertices[9], Vec3::ZERO, Vec2::new(S_STEP * 8.0, T_STEP)).normalFromPosition()); // v17 (shared)
		
		builder.vertex(Vertex::new(tmpVertices[4], Vec3::ZERO, Vec2::new(S_STEP * 3.0, T_STEP * 2.0)).normalFromPosition()); // v18 (shared)
		builder.vertex(Vertex::new(tmpVertices[6], Vec3::ZERO, Vec2::new(S_STEP * 5.0, T_STEP * 2.0)).normalFromPosition()); // v19 (shared)
		builder.vertex(Vertex::new(tmpVertices[8], Vec3::ZERO, Vec2::new(S_STEP * 7.0, T_STEP * 2.0)).normalFromPosition()); // 20 (shared)
		builder.vertex(Vertex::new(tmpVertices[10], Vec3::ZERO, Vec2::new(S_STEP * 9.0, T_STEP * 2.0)).normalFromPosition()); // 21 (shared)
		
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
