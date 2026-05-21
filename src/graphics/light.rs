use glam::Vec3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Light {
	Directional(LightProperties),
	Point(LightProperties),
}

impl Light {
	pub fn properties(&self) -> &LightProperties {
		match self {
			Light::Directional(properties) => &properties,
			Light::Point(properties) => &properties,
		}
	}
	
	pub fn propertiesMut(&mut self) -> &mut LightProperties {
		match self {
			Light::Directional(properties) => properties,
			Light::Point(properties) => properties,
		}
	}
	
	pub fn toU32(&self) -> u32 {
		match self {
			Light::Directional(_) => 0,
			Light::Point(_) => 1,
		}
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightProperties {
	pub position: Vec3,
	pub ambient: Vec3,
	pub ambientStrength: f32,
	pub diffuseStrength: f32,
	pub specularStrength: f32,
}

impl Default for LightProperties {
	fn default() -> LightProperties {
		Self {
			position: Vec3::ZERO,
			ambient: Vec3::ONE,
			ambientStrength: 1.0,
			diffuseStrength: 1.0,
			specularStrength: 1.0,
		}
	}
}
