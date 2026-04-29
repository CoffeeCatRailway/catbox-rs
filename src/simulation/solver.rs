use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
use bool_flags::Flags8;
use dear_imgui_rs::{TreeNodeFlags, Ui, WindowFlags};
use glam::{vec3, Mat4, Vec3};
use tracing::info;
use crate::graphics::{LineRenderer, Renderable};
use crate::graphics::mesh::{Mesh, Vertex};
use crate::simulation::region::{BSPGrid, AABB};
use crate::simulation::Transform;
use crate::thread_pool::ThreadPool;
use crate::types::{newMeshRef, GlRef, MeshRef, PhysicalRef, ShaderRef};

static U64_ATOMIC_BUFFER: AtomicU64 = AtomicU64::new(0);
static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn newId() -> usize {
	ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[allow(unused)]
pub trait Physical: Debug + Send + Sync {
	fn id(&self) -> usize;
	
	fn transform(&self) -> &Transform;
	
	fn transformMut(&mut self) -> &mut Transform;
	
	fn lastTransform(&self) -> &Transform;
	
	fn lastTransformMut(&mut self) -> &mut Transform;
	
	fn fixed(&self) -> bool;
	
	fn update(&mut self, dt: f32);
	
	fn accelerate(&mut self, acceleration: Vec3);
	
	fn setVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn addVelocity(&mut self, velocity: Vec3, dt: f32);
	
	fn getVelocity(&self, dt: f32) -> Vec3;
	
	fn elasticity(&self) -> f32; // todo: try moving properties to separate component
	
	fn color(&self) -> Vec3; // todo: move shape and collision to separate components
	
	fn bounds(&self) -> AABB;
}

struct Edge {
	id: usize,
	isMinimum: bool,
	coord: f32,
}

struct Chunk {
	tree: BSPGrid<PhysicalRef>,
	physicals: Vec<PhysicalRef>,
	neighbours: Vec<ChunkRef>,
}

type ChunkRef = Arc<RwLock<Chunk>>;

const F_DESTROYED: u8 = 0;
const F_PAUSED: u8 = 1;
const F_FORCE_STEP: u8 = 2;

const F_COLLISION_MODE: u8 = 3;
const F_THREAD_MODE: u8 = 4;

const GRID_CAPACITY: usize = 7;
const THREAD_COUNT: usize = 6;

pub struct Solver {
	mesh: MeshRef,
	shader: ShaderRef,
	
	pub gravity: Vec3,
	pub worldSize: Vec3,
	
	threadPool: ThreadPool,
	chunks: Vec<ChunkRef>,

	edgesX: Vec<Edge>,
	edgesY: Vec<Edge>,
	quadTree: BSPGrid<PhysicalRef>,
	physicals: HashMap<usize, PhysicalRef>,
	
	subSteps: u32,
	updatesDone: u32,
	
	flags: Flags8,

	sweepAxis: String,
	collisionChecks: usize,

	calcEdgeCoordsAccum: f32,
	calcEdgeCoordsTime: f32,
	sortTimeAccum: f32,
	sortTime: f32,
	sweepTimeAccum: f32,
	sweepTime: f32,
	
	subStepTimeAccum: f32,
	subStepTime: f32,
	
	chunkBuildTime: f32,
	
	stepTime: f32,
}

impl Solver {
	pub fn new(worldSize: Vec3, gl: GlRef, shader: ShaderRef) -> Result<Solver, String> {
		let worldSize = worldSize.truncate().extend(0.0); // Only simulating 2d for now
		
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
		
		info!("Creating solver chunks");
		let mut chunks = Vec::with_capacity(THREAD_COUNT * THREAD_COUNT);
		let chunkSize = worldSize / THREAD_COUNT as f32;
		let worldSizeHalf = worldSize / 2.0;
		
		for y in 0..THREAD_COUNT {
			for x in 0..THREAD_COUNT {
				let pos = Vec3::new(x as f32 * chunkSize.x, (THREAD_COUNT - 1 - y) as f32 * chunkSize.y, 0.0) - worldSizeHalf;
				chunks.push(Arc::new(RwLock::new(Chunk {
					tree: BSPGrid::new(GRID_CAPACITY, AABB::new(pos, chunkSize)),
					physicals: Vec::new(),
					neighbours: Vec::new(),
				})));
			}
		}
		
		info!("Finding chunk neighbours");
		for x in 0..THREAD_COUNT {
			for y in 0..THREAD_COUNT {
				let chunk = chunks[x + y * THREAD_COUNT].clone();
				for dy in -1..2 {
					for dx in -1..2 {
						let x = dx + x as i32;
						let y = dy + y as i32;
						if (dx == 0 && dy == 0) || (x < 0 || x >= THREAD_COUNT as i32 || y < 0 || y >= THREAD_COUNT as i32) {
							continue;
						}
						chunk.write().unwrap().neighbours.push(chunks[y as usize + x as usize * THREAD_COUNT].clone());
					}
				}
			}
		}
		
		let mut flags = Flags8::none();
		flags.set(F_PAUSED);
		flags.set(F_COLLISION_MODE);
		flags.set(F_THREAD_MODE);
		Ok(Self {
			mesh,
			shader,
			
			gravity: Vec3::ZERO,
			worldSize,
			
			threadPool: ThreadPool::new(THREAD_COUNT),
			chunks,

			edgesX: Vec::new(),
			edgesY: Vec::new(),
			quadTree: BSPGrid::new(GRID_CAPACITY, AABB::centered(Vec3::ZERO, worldSize)), // todo: fix vec3 issue with aabb/quadtree
			physicals: HashMap::new(),
			
			subSteps: 8,
			updatesDone: 0,
			
			flags,

			sweepAxis: "X".to_string(),
			collisionChecks: 0,

			calcEdgeCoordsAccum: 0.0,
			calcEdgeCoordsTime: 0.0,
			sortTimeAccum: 0.0,
			sortTime: 0.0,
			sweepTimeAccum: 0.0,
			sweepTime: 0.0,
			
			subStepTimeAccum: 0.0,
			subStepTime: 0.0,
			
			chunkBuildTime: 0.0,
			
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
		let id = {
			let borrow = physical.read().unwrap();
			let id = borrow.id();
			let transform = borrow.transform();
			self.edgesX.push(Edge {
				id,
				isMinimum: true,
				coord: transform.position.x - transform.scale.x / 2.0,
			});
			self.edgesX.push(Edge {
				id,
				isMinimum: false,
				coord: transform.position.x + transform.scale.x / 2.0,
			});
			self.edgesY.push(Edge {
				id,
				isMinimum: true,
				coord: transform.position.y - transform.scale.y / 2.0,
			});
			self.edgesY.push(Edge {
				id,
				isMinimum: false,
				coord: transform.position.y + transform.scale.y / 2.0,
			});
			id
		};
		self.physicals.insert(id, physical);
	}
	
	// todo: try collision checks with rays
	fn collideWithPhysical(physical1: PhysicalRef, physical2: PhysicalRef) {
		if let Ok(mut physical1) = physical1.try_write() {
			if let Ok(mut physical2) = physical2.try_write() {
				let r1 = physical1.transform().scale.x / 2.0;
				let r2 = physical2.transform().scale.x / 2.0;
				
				let dir = physical1.transform().position - physical2.transform().position;
				let dist = dir.length();
				let minDist = r1 + r2;
				if dist < minDist {
					let mut dir = dir.normalize_or_zero();
					if dist <= f32::EPSILON {
						dir = Vec3::X;
					}
					
					let massRatio1 = r1 / minDist;
					let massRatio2 = r2 / minDist;
					let force = ((physical1.elasticity() + physical2.elasticity()) / 2.0) / 2.0 * (dist - minDist);
					
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
	
	fn collideWithBoundary(_dt: f32, physical: PhysicalRef, worldSize: Vec3) {
		if let Ok(mut physical) = physical.try_write() {
			let halfSize = (worldSize - physical.transform().scale.x) / 2.0;
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
	
	fn sweepEdges(edges: &Vec<Edge>) -> Vec<(usize, usize)> {
		let mut pairs = Vec::<(usize, usize)>::new();
	
		let mut touching = Vec::<usize>::new();
		for edge in edges.iter() {
			let edgeId = edge.id;
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
	
	fn calcVarianceForEdges(edges: &Vec<Edge>) -> f32 {
		let mut sum = 0.0;
		for edge in edges.iter() {
			sum += edge.coord;
		}
		let mean = sum / edges.len() as f32;
	
		let mut squaredDiffSum = 0.0;
		for edge in edges.iter() {
			let diff = edge.coord - mean;
			squaredDiffSum += diff * diff;
		}
	
		squaredDiffSum / edges.len() as f32
	}
	
	fn calcEdgeCoords(&mut self) {
		let now = Instant::now();
	
		for i in 0..self.edgesX.len() {
			let edgeX = &mut self.edgesX[i];
			let (px, sx) = {
				let borrow = self.physicals[&edgeX.id].read().unwrap();
				let transform = borrow.transform();
				(transform.position.x, transform.scale.x / 2.0)
			};
			if edgeX.isMinimum {
				edgeX.coord = px - sx;
			} else {
				edgeX.coord = px + sx;
			}
	
			let edgeY = &mut self.edgesY[i];
			let (py, sy) = {
				let borrow = self.physicals[&edgeY.id].read().unwrap();
				let transform = borrow.transform();
				(transform.position.y, transform.scale.y / 2.0)
			};
			if edgeY.isMinimum {
				edgeY.coord = py - sy;
			} else {
				edgeY.coord = py + sy;
			}
		}
	
		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.calcEdgeCoordsAccum += end;
	}
	
	fn broadPhaseCollisionCheck(&mut self) {
		let now = Instant::now();
	
		Self::insertionSort(&mut self.edgesX, |a, b| a.coord < b.coord);
		Self::insertionSort(&mut self.edgesY, |a, b| a.coord < b.coord);
	
		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.sortTimeAccum += end;
	
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
	
		let pairs = {
			let varianceX = Self::calcVarianceForEdges(&self.edgesX);
			let varianceY = Self::calcVarianceForEdges(&self.edgesY);
			if varianceX > varianceY {
				self.sweepAxis = "X".to_string();
				Self::sweepEdges(&self.edgesX)
			} else {
				self.sweepAxis = "Y".to_string();
				Self::sweepEdges(&self.edgesY)
			}
		};
		self.collisionChecks = pairs.len();
	
		// let pairsX = Self::sweepEdges(&self.edgesX);//.into_iter().collect::<HashSet<_>>();
		// let pairsY = self.sweepEdges(&self.edgesY);//.into_iter().collect::<HashSet<_>>();
		// let pairs = pairsY.into_iter().filter(|x| pairsX.contains(x)).collect::<Vec<_>>();
		// let pairs = pairsX.intersection(&pairsY).collect::<HashSet<_>>();
	
		for (a, b) in pairs.into_iter() {
			let physical1 = self.physicals[&a].clone();
			let physical2 = self.physicals[&b].clone();
			Self::collideWithPhysical(physical1, physical2);
		}
	
		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.sweepTimeAccum += end;
	}
	
	fn updatePhysicals(&self, dt: f32) {
		for (_, physical) in self.physicals.iter() {
			let physical = physical.clone();
			{
				let mut physicalMut = physical.write().unwrap();
				physicalMut.accelerate(self.gravity);
				// let gravity = (Vec3::ZERO - physicalMut.transform().position).normalize_or_zero() * self.gravity.x;
				// physicalMut.accelerate(gravity);
				physicalMut.update(dt);
			}
			Self::collideWithBoundary(dt, physical, self.worldSize);
		}
	}
	
	fn populateQuadTree(&mut self) {
		// 17+ fps
		// ~3.5ms
		self.quadTree.clear();
		for (_, physical) in self.physicals.iter() {
			self.quadTree.insert(physical.clone(), &|physical, bounds| {
				bounds.containsPoint(physical.read().unwrap().transform().position)
			});
		}
	}
	
	fn subStep(&mut self, dt: f32) {
		let now = Instant::now();
	
		if self.flags.get(F_COLLISION_MODE) {
			self.populateQuadTree();
	
			// ~5ms
			// ~50ms (full step)
			for (id, physical) in self.physicals.iter() {
				let found = {
					let area = physical.read().unwrap().bounds();
					self.quadTree.findInArea(&area, &|physical, bounds| {
						bounds.overlaps(&physical.read().unwrap().bounds())
					})
				};
				for physical2 in found.into_iter() {
					if *id == physical2.read().unwrap().id() {
						continue;
					}
					Self::collideWithPhysical(physical.clone(), physical2.clone());
				}
			}
		} else {
			// 5-18 fps
			// ~27ms
			// ~150ms (full step)
			self.calcEdgeCoords();
			self.broadPhaseCollisionCheck();
		}
		self.updatePhysicals(dt);
	
		let end = now.elapsed().as_secs_f32() * 1000.0;
		self.subStepTimeAccum += end;
	}
	
	fn collideBroadPhaseChunk(dt: f32, chunk: ChunkRef, worldSize: Vec3) {
		if let Ok(chunk) = chunk.try_read() {
			// let now = Instant::now();
			for physical1 in chunk.physicals.iter() {
				let (id1, bounds) = {
					let read = physical1.read().unwrap();
					(read.id(), read.bounds())
				};
				
				let mut found = chunk.tree.findInArea(&bounds, &|physical, bounds| {
					bounds.overlaps(&physical.read().unwrap().bounds())
				});
				
				// for neighbour in chunk.neighbours.iter() {
				// 	if let Ok(neighbour) = neighbour.try_read() {
				// 		let mut extra = neighbour.tree.findInArea(&bounds, &|physical, bounds| {
				// 			bounds.overlaps(&physical.read().unwrap().bounds())
				// 		});
				// 		found.append(&mut extra);
				// 	}
				// }
				
				for physical2 in found.into_iter() {
					let id2 = { physical2.read().unwrap().id() };
					if id1 == id2 {
						continue;
					}
					Self::collideWithPhysical(physical1.clone(), physical2.clone());
				}
				
				Self::collideWithBoundary(dt, physical1.clone(), worldSize); // todo: fix physicals 'outside' world
			}
			// let end = now.elapsed().as_secs_f32() * 1000.0;
			// info!("Chunk collision took {}ms", end);
		}
	}
	
	fn updatePhysicalsChunk(dt: f32, chunk: ChunkRef, gravity: Vec3) {
		if let Ok(chunk) = chunk.try_read() {
			for physical in chunk.physicals.iter() {
				if let Ok(mut physical) = physical.try_write() {
					physical.accelerate(gravity);
					// let gravity = (Vec3::ZERO - physical.transform().position).normalize_or_zero() * gravity.length();
					// physical.accelerate(gravity);
					physical.update(dt);
				}
			}
		}
	}
	
	pub fn update(&mut self, dt: f32) {
		if self.isDestroyed() {
			return;
		}

		if !self.isPaused() || self.isForceStep() {
			let now = Instant::now();
			
			let subSteps = self.subSteps;
			let subStepDt = dt / subSteps as f32;
			
			if self.flags.get(F_THREAD_MODE) {
				// 30+ fps
				// ~10-20ms (full step)
				
				let gravity = self.gravity;
				let worldSize = self.worldSize;
				
				U64_ATOMIC_BUFFER.store(0, Ordering::Relaxed);
				for x in 0..THREAD_COUNT {
					for y in 0..THREAD_COUNT {
						let chunk = self.chunks[x + y * THREAD_COUNT].clone();
						let physicals = self.physicals.clone();
						self.threadPool.execute(move |_| {
							let now = Instant::now();
							
							if let Ok(mut chunk) = chunk.try_write() {
								chunk.tree.clear();
								chunk.physicals.clear();
								
								for (_, physical) in physicals.iter() {
									if chunk.tree.insert(physical.clone(), &|physical, bounds| {
										bounds.containsPoint(physical.read().unwrap().transform().position)
									}) {
										chunk.physicals.push(physical.clone());
									}
								}
							}
							
							let end = now.elapsed().as_micros();
							U64_ATOMIC_BUFFER.fetch_add(end as u64, Ordering::Relaxed);
							// info!("{}", end as f32 / 1000.0);
						});
					}
					self.threadPool.waitForCompletion();
				}
				self.chunkBuildTime = (U64_ATOMIC_BUFFER.load(Ordering::Relaxed) / (THREAD_COUNT * THREAD_COUNT) as u64) as f32 / 1000.0;
				
				U64_ATOMIC_BUFFER.store(0, Ordering::Relaxed);
				for x in 0..THREAD_COUNT {
					for y in 0..THREAD_COUNT {
						let x = (x + y) % THREAD_COUNT; // Stagger x so no neighboring threads update simultaneously
						let chunk = self.chunks[x + y * THREAD_COUNT].clone();
						
						self.threadPool.execute(move |_| {
							for _ in 0..subSteps {
								let now = Instant::now();
								
								Self::collideBroadPhaseChunk(subStepDt, chunk.clone(), worldSize);
								Self::updatePhysicalsChunk(subStepDt, chunk.clone(), gravity);
								
								let end = now.elapsed().as_micros();
								U64_ATOMIC_BUFFER.fetch_add(end as u64, Ordering::Relaxed);
								// info!("{}", end as f32 / 1000.0);
							}
						});
					}
					self.threadPool.waitForCompletion();
				}
				self.subStepTime = (U64_ATOMIC_BUFFER.load(Ordering::Relaxed) / (THREAD_COUNT * THREAD_COUNT * self.subSteps as usize) as u64) as f32 / 1000.0;
			} else {
				self.populateQuadTree();
				for _ in 0..subSteps {
					self.subStep(subStepDt);
				}
				
				let timeRecip = 1.0 / self.subSteps as f32;
				self.calcEdgeCoordsTime = self.calcEdgeCoordsAccum * timeRecip;
				self.calcEdgeCoordsAccum = 0.0;
				self.sortTime = self.sortTimeAccum * timeRecip;
				self.sortTimeAccum = 0.0;
				self.sweepTime = self.sweepTimeAccum * timeRecip;
				self.sweepTimeAccum = 0.0;
				
				self.subStepTime = self.subStepTimeAccum * timeRecip;
				self.subStepTimeAccum = 0.0;
			}

			let end = now.elapsed().as_secs_f32() * 1000.0;
			self.stepTime = end;
			
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
				
				let mut collisionMode = self.flags.get(F_COLLISION_MODE);
				let mut threadMode = self.flags.get(F_THREAD_MODE);
				ui.checkbox("Use threads", &mut threadMode);
				if threadMode {
					self.flags.set(F_THREAD_MODE);
					ui.text(format!("Thread count: {}", THREAD_COUNT));
				} else {
					self.flags.clear(F_THREAD_MODE);
					ui.checkbox("Space partition/Sweep n' prune", &mut collisionMode);
					
					if collisionMode {
						self.flags.set(F_COLLISION_MODE);
						ui.text(format!("Partition depth: {}", self.quadTree.depth()));
					} else {
						self.flags.clear(F_COLLISION_MODE);
						ui.text(format!("Sweep axis: {}", self.sweepAxis));
						ui.text(format!("Collision checks: {}", self.collisionChecks));
					}
				}
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
				
				if ui.collapsing_header("Times", TreeNodeFlags::COLLAPSING_HEADER) {
					if threadMode {
						ui.text("(*) = Averaged over sub steps and threads");
						
						ui.text(format!("Chunk build time*: {}ms", self.chunkBuildTime));
					} else {
						ui.text("(*) = Averaged over sub steps");
						
						if !collisionMode {
							ui.text(format!("Calc edge coords time*: {}ms", self.calcEdgeCoordsTime));
							ui.text(format!("Sort time*: {}ms", self.sortTime));
							ui.text(format!("Sweep time*: {}ms", self.sweepTime));
						}
					}
					ui.text(format!("Sub step time*: {}ms", self.subStepTime));
					ui.text(format!("Step time: {}ms", self.stepTime));
				}
			});
	}
	
	pub fn getPhysicals(&self) -> &HashMap<usize, PhysicalRef> {
		&self.physicals
	}

	// Semi-Redundant since render manager destroys all renderables, here in case of multiple solvers
	pub fn destroy(&mut self) {
		self.flags.set(F_DESTROYED);
		self.mesh.borrow_mut().destroy();
		self.threadPool.stopAll();
	}
}

impl Renderable for Solver {
	fn meshRef(&self) -> Option<&MeshRef> {
		Some(&self.mesh)
	}
	
	fn shaderRef(&self) -> Option<&ShaderRef> {
		Some(&self.shader)
	}
	
	fn renderPost(&self, projViewMat: &Mat4, dt: f32, lineRenderer: &mut LineRenderer) -> Result<(), String> {
		let collisionMode = self.flags.get(F_COLLISION_MODE);
		let threadMode = self.flags.get(F_THREAD_MODE);
		
		if threadMode {
			for chunk in self.chunks.iter() {
				chunk.read().unwrap().tree.render(projViewMat, dt, lineRenderer)?;
				lineRenderer.pushAABB(chunk.read().unwrap().tree.bounds(), Vec3::Z);
			}
		} else if collisionMode {
			self.quadTree.render(projViewMat, dt, lineRenderer)?;
		}
		Ok(())
	}
	
	fn modelMatrix(&self) -> Mat4 {
		Mat4::from_scale(self.worldSize)
	}
}