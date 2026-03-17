use glam::{Mat4, Quat, Vec3};

#[derive(Copy, Clone)]
pub struct Transform {
	pub position: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
	pub localFront: Vec3,
	#[allow(unused)]
	pub localRight: Vec3,
	pub localUp: Vec3,
}

impl Default for Transform {
	fn default() -> Self {
		Transform {
			position: Vec3::ZERO,
			rotation: Quat::IDENTITY,
			scale: Vec3::ONE,
			localFront: Vec3::NEG_Z,
			localRight: Vec3::X,
			localUp: Vec3::Y,
		}
	}
}

#[allow(unused)]
impl Transform {
	pub fn getPositionMatrix(&self) -> Mat4 {
		Mat4::from_translation(self.position)
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
	
	pub fn getModelMatrix(&self) -> Mat4 {
		self.getPositionMatrix() * self.getRotationMatrix() * self.getScaleMatrix()
	}
	
	pub fn calculateLocalFront(&self) -> Vec3 {
		self.rotation.mul_vec3(Vec3::NEG_Z)
	}
	
	pub fn calculateLocalRight(&self) -> Vec3 {
		self.rotation.mul_vec3(Vec3::X)
	}
	
	pub fn calculateLocalUp(&self) -> Vec3 {
		self.rotation.mul_vec3(Vec3::Y)
	}
	
	pub fn calculateLocalVectors(&mut self) {
		self.localFront = self.calculateLocalFront();
		self.localRight = self.calculateLocalRight();
		self.localUp = self.calculateLocalUp();
	}
	
	pub fn translateGlobal(&mut self, translation: Vec3) {
		self.position += translation;
	}
	
	pub fn translateLocalForward(&mut self, delta: Vec3) {
		self.position += self.localFront * delta;
	}
	
	pub fn translateLocalRight(&mut self, delta: Vec3) {
		self.position += self.localRight * delta;
	}
	
	pub fn translateLocalUp(&mut self, delta: Vec3) {
		self.position += self.localUp * delta;
	}
}