use glam::{vec2, vec4, Mat4, Quat, Vec2, Vec3};
use crate::simulation::transform::Transform;

pub fn screenToWorldSpace(cursor: Vec2, width: u32, height: u32, projectionMatrix: Mat4, viewMatrix: Mat4) -> Vec3 {
	// https://antongerdelan.net/opengl/raycasting.html
	let ndc = vec2(
		(2.0 * cursor.x) / width as f32 - 1.0,
		1.0 - (2.0 * cursor.y) / height as f32,
	);
	let clip = vec4(ndc.x, ndc.y, -1.0, 1.0);
	let mut eye = projectionMatrix.inverse() * clip;
	eye.z = -1.0;
	eye.w = 0.0;
	(viewMatrix.inverse() * eye).truncate() //.normalize()
}

pub enum Projection {
	Perspective(f32),
	Orthographic(f32, f32, f32, f32),
}

pub struct Frustum {
	pub near: f32,
	pub far: f32,
	pub fov: f32,
	pub fovMin: f32,
	pub fovMax: f32,
}

impl Default for Frustum {
	fn default() -> Self {
		Frustum {
			near: 0.1,
			far: 100.0,
			fov: 45.0,
			fovMin: 1.0,
			fovMax: 45.0,
		}
	}
}

impl Frustum {
	pub fn zoom(&mut self, dt: f32) {
		self.fov += dt;
		self.fov = self.fov.clamp(self.fovMin, self.fovMax);
	}
}

pub struct Camera {
	pub frustum: Frustum,
	pub transform: Transform,
	pub yaw: f32,
	pub pitch: f32,
	pub pitchConstraint: f32,
	pub sensitivity: f32,
}

impl Default for Camera {
	fn default() -> Self {
		Camera {
			frustum: Default::default(),
			transform: Default::default(),
			yaw: -90.0,
			pitch: 0.0,
			pitchConstraint: 89.0,
			sensitivity: 0.1,
		}
	}
}

impl Camera {
	pub fn updateLocalVectors(&mut self) {
		let front = Vec3 {
			x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
			y: self.pitch.to_radians().sin(),
			z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
		};
		self.transform.frontLocal = front;
		self.transform.rightLocal = self.transform.frontLocal.cross(Vec3::Y).normalize();
		self.transform.upLocal = self.transform.rightLocal.cross(self.transform.frontLocal).normalize();
		self.transform.rotation = Quat::look_to_rh(self.transform.frontLocal, self.transform.upLocal).normalize().inverse();
	}
	
	pub fn turn(&mut self, xo: f32, yo: f32) {
		self.yaw += xo * self.sensitivity;
		self.pitch += yo * self.sensitivity;
		
		self.yaw = self.yaw % 360.0;
		self.pitch = self.pitch.clamp(-self.pitchConstraint, self.pitchConstraint);// % 360.0;
		
		self.updateLocalVectors();
	}
	
	pub fn getProjectionMatrix(&mut self, projection: Projection) -> Mat4 {
		match projection {
			Projection::Perspective(aspect) => {
				Mat4::perspective_rh(self.frustum.fov.to_radians(), aspect, self.frustum.near, self.frustum.far)
			},
			Projection::Orthographic(left, right, bottom, top) => {
				let zoom = self.frustum.fov;
				Mat4::orthographic_rh(left * zoom, right * zoom, bottom * zoom, top * zoom, self.frustum.near, self.frustum.far)
			},
		}
	}
	
	pub fn getViewMatrix(&self) -> Mat4 {
		Mat4::look_at_rh(self.transform.position, self.transform.position + self.transform.frontLocal, self.transform.upLocal)
	}
}