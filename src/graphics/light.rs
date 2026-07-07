use glam::Vec3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Light {
	Directional(LightProperties),
	Point(LightProperties),
	// Spot(LightProperties),
}

impl Light {
	pub fn properties(&self) -> &LightProperties {
		match self {
			Light::Directional(properties) => properties,
			Light::Point(properties) => properties,
			// Light::Spot(properties) => properties,
		}
	}
	
	pub fn propertiesMut(&mut self) -> &mut LightProperties {
		match self {
			Light::Directional(properties) => properties,
			Light::Point(properties) => properties,
			// Light::Spot(properties) => properties,
		}
	}
	
	pub fn toU8(&self) -> u8 {
		match self {
			Light::Directional(_) => 0,
			Light::Point(_) => 1,
			// Light::Spot(_) => 2,
		}
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightProperties {
	pub position: Vec3,
	
	pub color: Vec3,
	pub ambient: f32,
	pub diffuse: f32,
	pub specular: f32,
	
	// attenuation
	pub intensity: f32,
	pub radius: f32,
}

impl Default for LightProperties {
	fn default() -> LightProperties {
		Self {
			position: Vec3::ZERO,
			
			color: Vec3::ONE,
			ambient: 1.0,
			diffuse: 1.0,
			specular: 1.0,
			
			intensity: 1.0,
			radius: 20.0,
		}
	}
}
