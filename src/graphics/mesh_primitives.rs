use std::f32::consts::TAU;
use glam::Vec3;
use crate::graphics::mesh::Vertex;

// todo: Chose XYZ plane for 2d
pub struct Primitives2D {}
pub struct Primitives3D {}

impl Primitives2D {
	pub fn circleXY(segments: usize, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
		let mut vertices = Vec::with_capacity(segments + 1);
		let mut indices = Vec::with_capacity(segments * 3);
		
		vertices.push(Default::default());
		for i in 0..segments {
			let angle = i as f32 * TAU / segments as f32;
			vertices.push(Vertex {
				position: Vec3::new(angle.cos(), angle.sin(), 0.0) / 2.0 * radius,
				..Default::default()
			});
			
			indices.push(0);
			indices.push(i as u32 + 1);
			let i2 = i as u32 + 2;
			if i2 <= segments as u32 {
				indices.push(i2);
			} else {
				indices.push(i2 - segments as u32);
			}
		}
		
		(vertices, indices)
	}
	
	// todo: box
}

impl Primitives3D {
	pub fn ball(resolution: usize, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
		let mut vertices = Vec::new(); // todo: calc capacity
		
		for longitudeI in 0..(resolution / 2) {
			let longitudeAngle = longitudeI as f32 / resolution as f32 * TAU;
			for latitudeI in 0..resolution {
				let latitudeAngle = latitudeI as f32 / resolution as f32 * TAU;
				
				let x = latitudeAngle.cos() * longitudeAngle.cos();
				let y = latitudeAngle.sin() * longitudeAngle.sin();
				let z = longitudeAngle.sin();
				
				vertices.push(Vertex {
					position: Vec3::new(x, y, z) * radius,
					..Default::default()
				});
			}
		}
		
		(vertices, Vec::new())
	}
}
