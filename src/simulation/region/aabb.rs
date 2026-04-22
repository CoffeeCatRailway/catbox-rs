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
	
	#[allow(unused)]
	pub fn center(&self) -> Vec3 {
		self.position + self.size / 2.0
	}
	
	#[allow(unused)]
	pub fn grow(&self, extent: Vec3) -> Self {
		Self::centered(self.center(), self.size + extent)
	}
	
	#[allow(unused)]
	pub fn shrink(&self, extent: Vec3) -> Self {
		Self::centered(self.center(), self.size - extent)
	}
	
	pub fn start(&self) -> Vec3 {
		self.position
	}
	
	pub fn end(&self) -> Vec3 {
		self.position + self.size
	}
	
	pub fn overlaps(&self, other: &AABB) -> bool {
		let start1 = self.start();
		let end1 = self.end();
		
		let start2 = other.start();
		let end2 = other.end();
		
		start1.x <= end2.x && end1.x >= start2.x
			&& start1.y <= end2.y && end1.y >= start2.y
			&& start1.z <= end2.z && end1.z >= start2.z
	}
	
	pub fn containsPoint(&self, point: Vec3) -> bool {
		let start = self.start();
		let end = self.end();
		
		point.x >= start.x && point.x <= end.x
			&& point.y >= start.y && point.y <= end.y
			&& point.z >= start.z && point.z <= end.z
	}
}
