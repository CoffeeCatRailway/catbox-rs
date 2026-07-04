use glam::{vec2, vec4, Mat4, Quat, Vec2, Vec3};
use crate::simulation::Transform;

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
	(viewMatrix.inverse() * eye).truncate() //.normalize_or_zero()
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

impl Default for Frustum {
	fn default() -> Self {
		Frustum {
			near: 0.01,
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
	#[allow(unused)]
	pub yaw: f32,
	#[allow(unused)]
	pub pitch: f32,
	#[allow(unused)]
	pub pitchConstraint: f32,
	#[allow(unused)]
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

#[allow(unused)]
impl Camera {
	pub fn updateLocalVectors(&mut self) {
		let front = Vec3 {
			x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
			y: self.pitch.to_radians().sin(),
			z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
		};
		self.transform.localFront = front;
		self.transform.localRight = self.transform.localFront.cross(Vec3::Y).normalize_or_zero();
		self.transform.localUp = self.transform.localRight.cross(self.transform.localFront).normalize_or_zero();
		self.transform.rotation = Quat::look_to_rh(self.transform.localFront, self.transform.localUp).normalize().inverse();
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
		Mat4::look_at_rh(self.transform.position, self.transform.position + self.transform.localFront, self.transform.localUp)
	}
	
	pub fn calcFrustumBoundsForView(&self, winWidth: u32, winHeight: u32, view: Mat4) -> (Vec3, Vec3) {
		let aspect = winWidth as f32 / winHeight as f32;
		let fovRad = self.frustum.fov.to_radians();
		let near = self.frustum.near;
		let far = self.frustum.far;
		let hNear = 2.0 * (fovRad / 2.0).tan() * near;
		let wNear = hNear * aspect;
		let hFar = 2.0 * (fovRad / 2.0).tan() * far;
		let wFar = hFar * aspect;
		
		let centerFar = self.transform.position + self.transform.localFront * far;
		let topLeftFar = centerFar + (self.transform.localUp * hFar / 2.0) - (self.transform.localRight * wFar / 2.0);
		let topRightFar = centerFar + (self.transform.localUp * hFar / 2.0) + (self.transform.localRight * wFar / 2.0);
		let bottomLeftFar = centerFar - (self.transform.localUp * hFar / 2.0) - (self.transform.localRight * wFar / 2.0);
		let bottomRightFar = centerFar - (self.transform.localUp * hFar / 2.0) + (self.transform.localRight * wFar / 2.0);
		
		let centerNear = self.transform.position + self.transform.localFront * near;
		let topLeftNear = centerNear + (self.transform.localUp * hNear / 2.0) - (self.transform.localRight * wNear / 2.0);
		let topRightNear = centerNear + (self.transform.localUp * hNear / 2.0) + (self.transform.localRight * wNear / 2.0);
		let bottomLeftNear = centerNear - (self.transform.localUp * hNear / 2.0) - (self.transform.localRight * wNear / 2.0);
		let bottomRightNear = centerNear - (self.transform.localUp * hNear / 2.0) + (self.transform.localRight * wNear / 2.0);
		
		let frustumCenter = (centerFar - centerNear) / 2.0;
		let frustumToView = [
			view * bottomRightNear.extend(1.0),
			view * topRightNear.extend(1.0),
			view * bottomLeftNear.extend(1.0),
			view * topLeftNear.extend(1.0),
			view * bottomRightFar.extend(1.0),
			view * topRightFar.extend(1.0),
			view * bottomLeftFar.extend(1.0),
			view * topLeftFar.extend(1.0),
		];
		
		let (mut min, mut max) = (Vec3::MAX, Vec3::MIN);
		for i in 0..frustumToView.len() {
			min.x = min.x.min(frustumToView[i].x);
			min.y = min.y.min(frustumToView[i].y);
			min.z = min.z.min(frustumToView[i].z);
			max.x = max.x.max(frustumToView[i].x);
			max.y = max.y.max(frustumToView[i].y);
			max.z = max.z.max(frustumToView[i].z);
		}
		(min, max)
		
		// Other way to calculate
		// let (lightProj, lightView) = {
		// 	let invCameraProjViewMatrix = (projectMat * camera.getViewMatrix()).inverse();
		// 	let mut frustumCorners = Vec::new();
		// 	for x in 0..2 {
		// 		for y in 0..2 {
		// 			for z in 0..2 {
		// 				let p = invCameraProjViewMatrix * Vec4::new(
		// 					2.0 * x as f32 - 1.0,
		// 					2.0 * y as f32 - 1.0,
		// 					2.0 * z as f32 - 1.0,
		// 					1.0
		// 				);
		// 				frustumCorners.push(p / p.w);
		// 			}
		// 		}
		// 	}
		//
		// 	let mut frustumCenter = Vec3::ZERO;
		// 	for corner in frustumCorners.iter() {
		// 		frustumCenter += corner.truncate();
		// 	}
		// 	frustumCenter /= frustumCorners.len() as f32;
		//
		// 	let sunPosRelCenter = frustumCenter - self.sunLight.borrow().properties().position;
		// 	let lightView = Mat4::look_at_rh(sunPosRelCenter, frustumCenter, Vec3::Y);
		//
		// 	let mut minX = f32::MAX;
		// 	let mut maxX = f32::MIN;
		// 	let mut minY = f32::MAX;
		// 	let mut maxY = f32::MIN;
		// 	let mut minZ = f32::MAX;
		// 	let mut maxZ = f32::MIN;
		// 	for corner in frustumCorners.iter() {
		// 		let p = lightView * corner;
		// 		minX = minX.min(p.x);
		// 		maxX = maxX.max(p.x);
		// 		minY = minY.min(p.y);
		// 		maxY = maxY.max(p.y);
		// 		minZ = minZ.min(p.z);
		// 		maxZ = maxZ.max(p.z);
		// 	}
		//
		// 	// tune z-bounds to catch tall geometry outside of frustum
		// 	let zMult = 10.0;
		// 	if minZ < 0.0 {
		// 		minZ *= zMult;
		// 	} else {
		// 		minZ /= zMult;
		// 	}
		// 	if maxZ < 0.0 {
		// 		maxZ /= zMult;
		// 	} else {
		// 		maxZ *= zMult;
		// 	}
		//
		// 	let worldTexelSizeX = (maxX - minX) / SHADOW_MAP_RES as f32;
		// 	let worldTexelSizeY = (maxY - minY) / SHADOW_MAP_RES as f32;
		// 	minX = (minX / worldTexelSizeX).floor() * worldTexelSizeX;
		// 	maxX = (maxX / worldTexelSizeX).floor() * worldTexelSizeX;
		// 	minY = (minY / worldTexelSizeY).floor() * worldTexelSizeY;
		// 	maxY = (maxY / worldTexelSizeY).floor() * worldTexelSizeY;
		// 	let lightProj = Mat4::orthographic_rh_gl(minX, maxX, minY, maxY, minZ, maxZ);
		//
		// 	(lightProj, lightView)
		// };
	}
}
