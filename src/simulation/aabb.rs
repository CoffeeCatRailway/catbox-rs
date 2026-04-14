use glam::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
	pub position: Vec3,
	pub size: Vec3,
}

impl AABB {
	pub fn new(position: Vec3, size: Vec3) -> Self {
		Self { position, size }
	}
	
	pub fn centered(position: Vec3, size: Vec3) -> Self {
		Self::new(position - size / 2.0, size)
	}
	
	pub fn center(&self) -> Vec3 {
		self.position + self.size / 2.0
	}
	
	pub fn grow(&self, extent: Vec3) -> Self {
		Self::centered(self.center(), self.size + extent)
	}
	
	pub fn shrink(&self, extent: Vec3) -> Self {
		Self::centered(self.center(), self.size - extent)
	}
	
	pub fn start(&self) -> Vec3 {
		self.position
	}
	
	pub fn end(&self) -> Vec3 {
		self.position + self.size
	}
	
	pub fn overlaps(&self, other: &AABB) -> bool
	{
		(self.start().x <= other.end().x && self.end().x >= other.start().x) &&
			(self.start().y <= other.end().y && self.end().y >= other.start().y) &&
			(self.start().z <= other.end().z && self.end().z >= other.start().z)
	}
	
	pub fn containsPoint(&self, point: Vec3) -> bool {
		(point.x >= self.start().x && point.x <= self.end().x) &&
			(point.y >= self.start().y && point.y <= self.end().y) &&
			(point.z >= self.start().z && point.z <= self.end().z)
	}
}
