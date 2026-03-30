use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicUsize;
use std::time::Instant;
use bool_flags::Flags8;
use dear_imgui_rs::{Ui, WindowFlags};
use glam::{vec3, Mat4, Vec3};
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::render_manager::Renderable;
use crate::simulation::transform::Transform;
use crate::types::{newMeshRef, GlRef, MeshRef, PhysicalRef, ShaderRef};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn newId() -> usize {
	ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[allow(unused)]
pub trait Physical {
	fn transform(&self) -> &Transform;
	
	fn transformMut(&mut self) -> &mut Transform;
	
	fn lastTransform(&self) -> &Transform;
	
	fn lastTransformMut(&mut self) -> &mut Transform;
	
	fn fixed(&self) -> bool;
	
	fn elasticity(&self) -> f32; // todo: try moving properties to separate component
	
	fn update(&mut self, dt: f32);
	
	fn accelerate(&mut self, acceleration: Vec3);
	
	fn setVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn addVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn getVelocity(&self, dt: f32) -> Vec3;
	
	fn getColor(&self) -> Vec3; // todo: move shape and collision to separate components

	fn id(&self) -> usize;
}

enum Axis {
	X,
	Y,
	#[allow(unused)]
	Z,
}

struct Edge {
	physical: PhysicalRef,
	isMinimum: bool,
	axis: Axis,
}

impl Edge {
	pub fn coordinate(&self) -> f32 {
		let physical = self.physical.borrow();
  		let transform = physical.transform();
		match self.axis {
			Axis::X => {
				if self.isMinimum {
					transform.position.x - transform.scale.x * 0.5
				} else {
					transform.position.x + transform.scale.x * 0.5
				}
			}
			Axis::Y => {
				if self.isMinimum {
					transform.position.y - transform.scale.y * 0.5
				} else {
					transform.position.y + transform.scale.y * 0.5
				}
			}
			Axis::Z => {
				if self.isMinimum {
					transform.position.z - transform.scale.z * 0.5
				} else {
					transform.position.z + transform.scale.z * 0.5
				}
			}
		}
	}
}

const F_DESTROYED: u8 = 0;
const F_PAUSED: u8 = 1;
const F_FORCE_STEP: u8 = 2;

pub struct Solver {
	mesh: MeshRef,
	shader: ShaderRef,
	
	pub gravity: Vec3,
	pub worldSize: Vec3,

	edgesX: Vec<Edge>,
	edgesY: Vec<Edge>,
	physicals: HashMap<usize, PhysicalRef>,
	
	subSteps: u32,
	updatesDone: u32,
	
	flags: Flags8,
	
	sortTimeAccum: f32,
	sortTime: f32,
	sweepTimeAccum: f32,
	sweepTime: f32,
	subStepTimeAccum: f32,
	subStepTime: f32,
	stepTime: f32,
}

impl Solver {
	pub fn new(worldSize: Vec3, gl: GlRef, shader: ShaderRef) -> Result<Solver, String> {
		let mesh = {
			let vertices = vec![
				Vertex {
					position: vec3(-0.5, 0.5, 0.0),
					color: Vec3::splat(0.15),
				},
				Vertex {
					position: vec3(0.5, 0.5, 0.0),
					color: Vec3::splat(0.15),
				},
				Vertex {
					position: vec3(0.5, -0.5, 0.0),
					color: Vec3::splat(0.15),
				},
				Vertex {
					position: vec3(-0.5, -0.5, 0.0),
					color: Vec3::splat(0.15),
				}
			];
			let indices = vec![0, 1, 2, 2, 3, 0];
			let mut mesh = Mesh::simple(gl, vertices, Some(indices));
			mesh.upload(shader.clone())?;
			newMeshRef(mesh)
		};
		
		let mut flags = Flags8::none();
		flags.set(F_PAUSED);
		Ok(Self {
			mesh,
			shader,
			
			gravity: Vec3::ZERO,
			worldSize,

			edgesX: Vec::new(),
			edgesY: Vec::new(),
			physicals: HashMap::new(),
			
			subSteps: 8,
			updatesDone: 0,
			
			flags,

			sortTimeAccum: 0.0,
			sortTime: 0.0,
			sweepTimeAccum: 0.0,
			sweepTime: 0.0,
			subStepTimeAccum: 0.0,
			subStepTime: 0.0,
			stepTime: 0.0,
		})
	}
	
	pub fn isDestroyed(&self) -> bool {
		self.flags.get(F_DESTROYED)
	}
	
	pub fn isPaused(&self) -> bool {
		self.flags.get(F_PAUSED)
	}
	
	pub fn pause(&mut self, paused: bool) {
		if paused {
			self.flags.set(F_PAUSED);
		} else {
			self.flags.clear(F_PAUSED);
		}
	}
	
	fn isForceStep(&self) -> bool {
		self.flags.get(F_FORCE_STEP)
	}
	
	pub fn forceStep(&mut self, force: bool) {
		if force {
			self.flags.set(F_FORCE_STEP);
		} else {
			self.flags.clear(F_FORCE_STEP);
		}
	}
	
	pub fn addPhysical(&mut self, physical: PhysicalRef) {
		self.edgesX.push(Edge {
			physical: physical.clone(),
			isMinimum: true,
			axis: Axis::X,
		});
		self.edgesX.push(Edge {
			physical: physical.clone(),
			isMinimum: false,
			axis: Axis::X,
		});
		self.edgesY.push(Edge {
			physical: physical.clone(),
			isMinimum: true,
			axis: Axis::Y,
		});
		self.edgesY.push(Edge {
			physical: physical.clone(),
			isMinimum: false,
			axis: Axis::Y,
		});
		let id = physical.borrow().id();
		self.physicals.insert(id, physical);
	}

	fn insertionSort<T, F>(vec: &mut Vec<T>, mut compare: F)
	where
		F: FnMut(&T, &T) -> bool,
	{
		// Taken with 800 physicals and 1,-400 gravity
		// ~10.930447ms first sort
		// ~0.15062812ms second sort and onwards
		let n = 1..vec.len();
		for i in n {
			let mut j = i;
			while j > 0 && compare(&vec[j], &vec[j - 1]) {
				vec.swap(j, j - 1);
				j -= 1;
			}
		}

		// Taken with 800 physicals and 1,-400 gravity
		// ~10.008063ms first
		// ~2.7993727ms second-fifth
		// lowers afterwards
		// for i in 1..vec.len() {
		// 	for j in (0..i).rev() {// j=i-1; j >= 0; j--
		// 		if compare(&vec[j], &vec[j + 1]) {
		// 			break;
		// 		}
		// 		vec.swap(j, j + 1);
		// 	}
		// }
	}

	
	fn sortEdges(&mut self) {
		let now = Instant::now();
		
		// ~0.269025ms, average from 8 substeps with 800 physicals, done until stable state
		// default rust sort
		// self.edges.sort_by(|a, b| {
		// 	let ax = {
		// 		let ap = a.physical.borrow();
		// 		if a.isLeft {
		// 			ap.transform().position.x - ap.transform().scale.x * 0.5
		// 		} else {
		// 			ap.transform().position.x + ap.transform().scale.x * 0.5
		// 		}
		// 	};
		// 	let bx = {
		// 		let bp = b.physical.borrow();
		// 		if b.isLeft {
		// 			bp.transform().position.x - bp.transform().scale.x * 0.5
		// 		} else {
		// 			bp.transform().position.x + bp.transform().scale.x * 0.5
		// 		}
		// 	};
		// 	ax.total_cmp(&bx)
		// });
		
		// ~0.14825ms, average from 8 substeps with 800 physicals, done until stable state
		// insertion sort
		// for i in 1..self.edges.len() {
		// 	for j in (0..i).rev() { // j=i-1; j >= 0; j--
		// 		let ax = {
		// 			let a = &self.edges[j];
		// 			let ap = a.physical.borrow();
		// 			if a.isMinimum {
		// 				ap.transform().position.x - ap.transform().scale.x * 0.5
		// 			} else {
		// 				ap.transform().position.x + ap.transform().scale.x * 0.5
		// 			}
		// 		};
		//
		// 		let bx = {
		// 			let b = &self.edges[j + 1];
		// 			let bp = b.physical.borrow();
		// 			if b.isMinimum {
		// 				bp.transform().position.x - bp.transform().scale.x * 0.5
		// 			} else {
		// 				bp.transform().position.x + bp.transform().scale.x * 0.5
		// 			}
		// 		};
		//
		// 		if ax < bx {
		// 			break;
		// 		}
		// 		self.edges.swap(j, j + 1);
		// 	}
		// }

		Self::insertionSort(&mut self.edgesX, |a, b| {
			a.coordinate() < b.coordinate()
		});
		Self::insertionSort(&mut self.edgesY, |a, b| {
			a.coordinate() < b.coordinate()
		});
		
		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.sortTimeAccum += end;
	}
	
	fn collideWithPhysical(&self, physical1: PhysicalRef, physical2: PhysicalRef) {
		if let Ok(mut physical1) = physical1.try_borrow_mut() {
			if let Ok(mut physical2) = physical2.try_borrow_mut() {
				let r1 = physical1.transform().scale.x * 0.5;
				let r2 = physical2.transform().scale.x * 0.5;
				
				let dir = physical1.transform().position - physical2.transform().position;
				let dist = dir.length();
				let minDist = r1 + r2;
				if dist < minDist {
					let mut dir = dir.normalize();
					if dist <= f32::EPSILON {
						dir = Vec3::X;
					}
					
					let massRatio1 = r1 / minDist;
					let massRatio2 = r2 / minDist;
					let force = 0.5 * ((physical1.elasticity() + physical2.elasticity()) * 0.5) * (dist - minDist);
					
					if !physical1.fixed() {
						physical1.transformMut().position -= dir * massRatio2 * force;
					}
					if !physical2.fixed() {
						physical2.transformMut().position += dir * massRatio1 * force;
					}
				}
			}
		}
	}
	
	fn collideWithBoundary(&self, _dt: f32, physical: PhysicalRef) {
		if let Ok(mut physical) = physical.try_borrow_mut() {
			let halfSize = (self.worldSize - physical.transform().scale.x) * 0.5;
			let velocity = physical.getVelocity(1.0) * physical.elasticity();
			
			if physical.transform().position.x < -halfSize.x {
				physical.transformMut().position.x = -halfSize.x;
				physical.lastTransformMut().position.x = -halfSize.x + velocity.x;
			} else if physical.transform().position.x > halfSize.x {
				physical.transformMut().position.x = halfSize.x;
				physical.lastTransformMut().position.x = halfSize.x + velocity.x;
			}
			
			if physical.transform().position.y < -halfSize.y {
				physical.transformMut().position.y = -halfSize.y;
				physical.lastTransformMut().position.y = -halfSize.y + velocity.y;
			} else if physical.transform().position.y > halfSize.y {
				physical.transformMut().position.y = halfSize.y;
				physical.lastTransformMut().position.y = halfSize.y + velocity.y;
			}
		}
	}

	fn sweepEdges(&self, edges: &Vec<Edge>) -> Vec<(usize, usize)> {
		let mut pairs = Vec::<(usize, usize)>::new();

		let mut touching = Vec::<usize>::new();
		for edge in edges.iter() {
			let edgeId = edge.physical.borrow().id();
			if edge.isMinimum {
				for other in touching.iter() {
					let other = *other;
					pairs.push((other.min(edgeId), edgeId.max(other)));
				}
				touching.push(edgeId);
			} else {
				if let Some(index) = touching.iter().position(|x| *x == edgeId) {
					touching.remove(index);
				}
				// touching.retain(|x| *x != edgeId);
			}
		}

		pairs
	}
	
	fn collide(&mut self, _dt: f32) {
		let now = Instant::now();

		// let mut touching: Vec<usize> = Vec::new();
		// for edge in self.edgesX.iter() {
		// 	let edgeId = edge.physical.borrow().id();
		// 	if edge.isMinimum {
		// 		for other in touching.iter() {
		// 			self.collideWithPhysical(self.physicals.get(&other).unwrap().clone(), edge.physical.clone());
		// 		}
		// 		// self.collideWithBoundary(dt, edge.physical.clone());
		// 		touching.push(edgeId);
		// 	} else {
		// 		// if let Some(index) = touching.iter().position(|x| x.borrow().transform().position == edge.physical.borrow().transform().position) {
		// 		// 	touching.remove(index);
		// 		// }
		// 		touching.retain(|x| *x != edgeId);
		// 	}
		// }

		let pairsX = self.sweepEdges(&self.edgesX).into_iter().collect::<HashSet<_>>();
		let pairsY = self.sweepEdges(&self.edgesY);//.into_iter().collect::<HashSet<_>>();
		// let pairs = pairsX.intersection(&pairsY).collect::<HashSet<_>>();
		let pairs = pairsY.into_iter().filter(|x| pairsX.contains(x)).collect::<Vec<_>>();
		// println!("{} {} {}", pairsMin.len(), pairsMax.len(), pairs.len());
		for (a, b) in pairs.iter() {
			let physical1 = self.physicals.get(a).unwrap().clone();
			let physical2 = self.physicals.get(b).unwrap().clone();
			self.collideWithPhysical(physical1, physical2);
		}

		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.sweepTimeAccum += end;
	}
	
	fn updatePhysicals(&self, dt: f32) {
		for (_, physical) in self.physicals.iter() {
			let physical = physical.clone();
			{
				let mut physicalMut = physical.borrow_mut();
				physicalMut.accelerate(self.gravity);
				physicalMut.update(dt);
			}
			self.collideWithBoundary(dt, physical);
		}
	}
	
	fn subStep(&mut self, dt: f32) {
		let now = Instant::now();

		self.sortEdges();
		self.collide(dt);
		self.updatePhysicals(dt);

		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.subStepTimeAccum += end;
	}
	
	pub fn update(&mut self, dt: f32) {
		if self.isDestroyed() {
			return;
		}

		if !self.isPaused() || self.isForceStep() {
			let now = Instant::now();

			let subStepDt = dt / self.subSteps as f32;
			for _ in 0..self.subSteps {
				self.subStep(subStepDt);
			}

			let end = now.elapsed().as_secs_f32() * 1000.0;
			self.stepTime = end;

			let timeRecip = 1.0 / self.subSteps as f32;
			self.sortTime = self.sortTimeAccum * timeRecip;
			self.sortTimeAccum = 0.0;
			self.sweepTime = self.sweepTimeAccum * timeRecip;
			self.sweepTimeAccum = 0.0;
			self.subStepTime = self.subStepTimeAccum * timeRecip;
			self.subStepTimeAccum = 0.0;
			
			self.updatesDone += 1;
			self.forceStep(false);
		}
	}
	
	pub fn gui(&mut self, ui: &mut Ui, dt: f32) {
		ui.window("Verlet Solver")
			.flags(WindowFlags::ALWAYS_AUTO_RESIZE)
			.build(|| {
				ui.input_float3("Gravity", self.gravity.as_mut()).build();
				ui.separator();
				
				ui.text(format!("Physicals: {}", self.physicals.len()));
				ui.separator();

				ui.text(format!("Sub steps: {}", self.subSteps));
				ui.text(format!("Updates: {}", self.updatesDone));
				ui.text(format!("Update dt: {}", dt));
				ui.text(format!("Sub step dt: {}", dt / self.subSteps as f32));
				
				let mut pause = self.isPaused();
				ui.checkbox("Pause", &mut pause);
				self.pause(pause);
				if pause {
					ui.same_line();
					if ui.small_button("Step") {
						self.forceStep(true);
					}
				}
				ui.separator();

				ui.text("Keys: (*) = Averaged over sub steps");
				ui.text(format!("Sort time*: {}ms", self.sortTime));
				ui.text(format!("Sweep time*: {}ms", self.sweepTime));
				ui.text(format!("Sub step time*: {}ms", self.subStepTime));
				ui.text(format!("Step time: {}ms", self.stepTime));
			});
	}
	
	pub fn getPhysicals(&self) -> &HashMap<usize, PhysicalRef> {
		&self.physicals
	}

	// Semi-Redundant since render manager destroys all renderables, here in case of multiple solvers
	pub fn destroy(&mut self) {
		self.flags.set(F_DESTROYED);
		self.mesh.borrow_mut().destroy();
	}
}

impl Renderable for Solver {
	fn meshRef(&self) -> &MeshRef {
		&self.mesh
	}
	
	fn shaderRef(&self) -> &ShaderRef {
		&self.shader
	}
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::from_scale(self.worldSize)
	}
}