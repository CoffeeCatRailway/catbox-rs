#![allow(non_snake_case)]

use glam::{Mat4, Quat, Vec3};
use crate::simulation::Transform;

#[allow(unused)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
	Forward,
	Backward,
}

impl Direction {
	pub fn unitVec(&self) -> Vec3 {
		match self {
			Direction::Up => Vec3::Y,
			Direction::Down => Vec3::NEG_Y,
			Direction::Left => Vec3::NEG_X,
			Direction::Right => Vec3::X,
			Direction::Forward => Vec3::Z,
			Direction::Backward => Vec3::NEG_Z,
		}
	}
}

#[allow(unused)]
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
	pub turnSensitivity: f32,
}

impl Default for Camera {
	fn default() -> Self {
		let mut camera = Camera {
			frustum: Frustum {
				near: 0.1,
				far: 100.0,
				fov: 45.0,
				fovMin: 1.0,
				fovMax: 45.0
			}, // FOV depends on Projection
			transform: Transform::default(),
			yaw: -90.0,
			pitch: 0.0,
			turnSensitivity: 0.1,
		};
		// camera.transform.rotation = Quat::from_axis_angle(Vec3::Y, -90.0_f32.to_radians());
		camera.updateLocalVectors();
		camera
	}
}

#[allow(unused)]
impl Camera {
	fn updateLocalVectors(&mut self) {
		let front = Vec3 {
			x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
			y: self.pitch.to_radians().sin(),
			z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
		};
		self.transform.front = front;
		self.transform.right = self.transform.front.cross(Vec3::Y).normalize();
		self.transform.up = self.transform.right.cross(self.transform.front).normalize();
		self.transform.rotation = Quat::look_to_rh(self.transform.front, self.transform.up).normalize().inverse();
	}
	
	pub fn walk(&mut self, direction: Direction, local: bool, dt: f32) {
		if local {
			match direction {
				Direction::Up => { self.transform.moveUp(dt); }
				Direction::Down => { self.transform.moveDown(dt); }
				Direction::Left => { self.transform.moveLeft(dt); }
				Direction::Right => { self.transform.moveRight(dt); }
				Direction::Forward => { self.transform.moveForward(dt); }
				Direction::Backward => { self.transform.moveBackward(dt); }
			}
		} else {
			self.transform.position += direction.unitVec() * dt;
		}
	}
	
	pub fn turn(&mut self, mut xo: f32, mut yo: f32, constrainPitch: f32) {
		xo *= self.turnSensitivity;
		yo *= self.turnSensitivity;
		
		self.yaw += xo;
		self.pitch += yo;

		self.yaw = self.yaw % 360.0;
		self.pitch = self.pitch.clamp(-constrainPitch, constrainPitch) % 360.0;
		
		self.updateLocalVectors();
	}
	
	pub fn getProjectionMatrix(&mut self, projection: Projection) -> Mat4 {
		match projection {
			Projection::Perspective(aspect) => { Mat4::perspective_rh(self.frustum.fov.to_radians(), aspect, self.frustum.near, self.frustum.far) }
			Projection::Orthographic(left, right, bottom, top) => {
				let zoom = self.frustum.fov;
				Mat4::orthographic_rh(left * zoom, right * zoom, bottom * zoom, top * zoom, self.frustum.near, self.frustum.far)
			}
		}
	}
	
	pub fn getViewMatrix(&self) -> Mat4 {
		Mat4::look_at_rh(self.transform.position, self.transform.position + self.transform.front, self.transform.up)
	}
}

// pub trait Camera {
// 	fn getFrustum(&mut self) -> &mut Frustum;
//
// 	/**
// 	Camera2D: local is unused.
// 	*/
// 	fn walk(&mut self, direction: Direction, local: bool, dt: f32);
//
// 	/**
// 	Camera2D:
// 		Only x is used from deg and is used as 'roll'.
// 	*/
// 	fn turn(&mut self, deg: &Vec3);
//
// 	fn getProjectionMatrix(&mut self, projection: Projection) -> Mat4 {
// 		match projection {
// 			Projection::Perspective(aspect) => { Mat4::perspective_rh(self.getFrustum().fov.to_radians(), aspect, self.getFrustum().near, self.getFrustum().far) }
// 			Projection::Orthographic(left, right, bottom, top) => {
// 				let zoom = self.getFrustum().fov;
// 				Mat4::orthographic_rh(left * zoom, right * zoom, bottom * zoom, top * zoom, self.getFrustum().near, self.getFrustum().far)
// 			}
// 		}
// 	}
//
// 	fn getViewMatrix(&self) -> Mat4;
// }
//
// // TODO: Fix rotation and locals
// // TODO: More robust transform trait
// pub struct Camera2D {
// 	pub frustum: Frustum,
// 	pub pos: Vec2,
// 	pub rot: f32,
// }
//
// impl Default for Camera2D {
// 	fn default() -> Self {
// 		Camera2D {
// 			frustum: Frustum::new(0.0, 100.0, 1.0, 1.0, 10.0),
// 			pos: Vec2::ZERO,
// 			rot: 0.0,
// 		}
// 	}
// }
//
// impl Camera for Camera2D {
// 	fn getFrustum(&mut self) -> &mut Frustum {
// 		&mut self.frustum
// 	}
//
// 	fn walk(&mut self, direction: Direction, _local: bool, dt: f32) {
// 		match direction {
// 			Direction::Up | Direction::Down | Direction::Left | Direction::Right => {
// 				self.pos += direction.unitVec().xy() * dt;
// 			}
// 			Direction::Forward => { self.pos.y += dt; }
// 			Direction::Backward => { self.pos.y -= dt; }
// 		}
// 	}
//
// 	fn turn(&mut self, deg: &Vec3) {
// 		self.rot += deg.x;
// 		self.rot = self.rot % 360.0;
// 	}
//
// 	fn getViewMatrix(&self) -> Mat4 {
// 		Mat4::from_rotation_translation(Quat::from_rotation_z(self.rot), vec3(self.pos.x, self.pos.y, 0.0).neg())
// 	}
// }
//
// pub struct Camera3D {
// 	pub frustum: Frustum,
// 	pub pos: Vec3,
// 	pub front: Vec3,
// 	pub up: Vec3,
// 	pub right: Vec3,
// 	pub rot: Vec3,
// 	pub sensitivity: f32,
// }
//
// impl Default for Camera3D {
// 	fn default() -> Self {
// 		let mut camera = Camera3D {
// 			frustum: Frustum::new(0.0, 100.0, 45.0, 1.0, 45.0),
// 			pos: Vec3::ZERO,
// 			front: Vec3::ZERO,
// 			up: Vec3::ZERO,
// 			right: Vec3::ZERO,
// 			rot: Vec3::ZERO,
// 			sensitivity: 0.1,
// 		};
// 		camera.updateVectors();
// 		camera
// 	}
// }
//
// impl Camera3D {
// 	fn updateVectors(&mut self) {
// 		// let rotMat = Mat3::from_quat(Quat::from_euler(EulerRot::XYZ, self.rot.x, self.rot.y, self.rot.z)).transpose();
// 		let rotMat = Mat3::from_euler(EulerRot::XYZ, self.rot.x, self.rot.y, self.rot.z).transpose();
// 		let front = -rotMat.row(2);
// 		// let front = Vec3 {
// 		// 	x: self.rot.x.to_radians().cos() * self.rot.y.to_radians().cos(),
// 		// 	y: self.rot.y.to_radians().sin(),
// 		// 	z: self.rot.x.to_radians().sin() * self.rot.y.to_radians().cos(),
// 		// };
// 		self.front = front;
// 		self.right = self.front.cross(Vec3::Y).normalize();
// 		self.up = self.right.cross(self.front).normalize();
// 	}
// }
//
// impl Camera for Camera3D {
// 	fn getFrustum(&mut self) -> &mut Frustum {
// 		&mut self.frustum
// 	}
//
// 	fn walk(&mut self, direction: Direction, local: bool, dt: f32) {
// 		if local {
// 			match direction {
// 				Direction::Up => { self.pos += self.up * dt; }
// 				Direction::Down => { self.pos -= self.up * dt; }
// 				Direction::Left => { self.pos -= self.right * dt; }
// 				Direction::Right => { self.pos += self.right * dt; }
// 				Direction::Forward => { self.pos += self.front * dt; }
// 				Direction::Backward => { self.pos -= self.front * dt; }
// 			}
// 		} else {
// 			self.pos += direction.unitVec() * dt;
// 		}
// 	}
//
// 	fn turn(&mut self, deg: &Vec3) {
// 		self.rot += deg * self.sensitivity;
// 		self.rot = self.rot % 360.0;
//
// 		self.updateVectors();
// 	}
//
// 	fn getViewMatrix(&self) -> Mat4 {
// 		Mat4::look_at_rh(self.pos, self.pos + self.front, self.up)
// 	}
// }
