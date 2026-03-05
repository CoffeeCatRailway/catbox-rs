// use glam::Vec3;

pub mod transform;
pub mod camera;

// pub enum Direction {
// 	Up,
// 	Down,
// 	Left,
// 	Right,
// 	Forward,
// 	Backward,
// }
//
// impl Direction {
// 	pub fn unitVec(&self) -> Vec3 {
// 		match self {
// 			Direction::Up => Vec3::Y,
// 			Direction::Down => Vec3::NEG_Y,
// 			Direction::Left => Vec3::NEG_X,
// 			Direction::Right => Vec3::X,
// 			Direction::Forward => Vec3::Z,
// 			Direction::Backward => Vec3::NEG_Z,
// 		}
// 	}
//
// 	pub fn opposite(&self) -> Direction {
// 		match self {
// 			Direction::Up => Direction::Down,
// 			Direction::Down => Direction::Up,
// 			Direction::Left => Direction::Right,
// 			Direction::Right => Direction::Left,
// 			Direction::Forward => Direction::Forward,
// 			Direction::Backward => Direction::Backward,
// 		}
// 	}
// }
