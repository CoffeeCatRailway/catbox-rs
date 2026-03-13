use glam::{Mat4, Quat, Vec3};

#[derive(Copy, Clone)]
pub struct Transform {
	pub position: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
	pub frontLocal: Vec3,
	pub rightLocal: Vec3,
	pub upLocal: Vec3,
}

impl Default for Transform {
	fn default() -> Self {
		Transform {
			position: Vec3::ZERO,
			rotation: Quat::IDENTITY,
			scale: Vec3::ONE,
			frontLocal: Vec3::NEG_Z,
			rightLocal: Vec3::X,
			upLocal: Vec3::Y,
		}
	}
}

impl Transform {
	pub fn getPositionMatrix(&self) -> Mat4 {
		Mat4::from_translation(-self.position)
	}
	
	pub fn getRotationMatrix(&self) -> Mat4 {
		Mat4::from_quat(self.rotation)
	}
	
	pub fn getScaleMatrix(&self) -> Mat4 {
		Mat4::from_scale(self.scale)
	}
	
	pub fn getWorldMatrix(&self) -> Mat4 {
		self.getPositionMatrix() * self.getRotationMatrix()
	}
	
	pub fn getViewMatrix(&self) -> Mat4 {
		self.getRotationMatrix() * self.getPositionMatrix()
	}
	
	pub fn calcFront(&self) -> Vec3 {
		self.rotation.mul_vec3(Vec3::NEG_Z)
	}
	
	pub fn calcRight(&self) -> Vec3 {
		self.rotation.mul_vec3(Vec3::X)
	}
	
	pub fn calcUp(&self) -> Vec3 {
		self.rotation.mul_vec3(Vec3::Y)
	}
	
	pub fn updateLocalVectors(&mut self) {
		self.frontLocal = self.calcFront();
		self.rightLocal = self.calcRight();
		self.upLocal = self.calcUp();
	}
	
	pub fn translateGlobal(&mut self, translation: Vec3) {
		self.position += translation;
	}
	
	pub fn forward(&mut self, delta: f32) {
		self.position += self.frontLocal * delta;
	}
	
	pub fn right(&mut self, delta: f32) {
		self.position += self.rightLocal * delta;
	}
	
	pub fn up(&mut self, delta: f32) {
		self.position += self.upLocal * delta;
	}
}
