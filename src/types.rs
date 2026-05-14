use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use glow::{Context as GlowContext};
use sdl3::video::Window as SdlWindow;
use crate::graphics::mesh::Mesh;
use crate::graphics::Renderable;
use crate::graphics::shader::Shader;
use crate::simulation::{Physical, Solver};

pub type SdlWindowRef = Rc<RefCell<SdlWindow>>;

pub type GlRef = Arc<GlowContext>;

pub type ShaderRef = Arc<RwLock<Shader>>;

pub type RenderableRef = Rc<RefCell<dyn Renderable>>;

pub type MeshRef = Rc<RefCell<Mesh>>;

pub type PhysicalRef = Arc<RwLock<dyn Physical>>;

pub type SolverRef = Rc<RefCell<Solver>>;

pub fn newSdlWindowRef(window: SdlWindow) -> SdlWindowRef {
	Rc::new(RefCell::new(window))
}

pub fn newGlRef(gl: GlowContext) -> GlRef {
	Arc::new(gl)
}

pub fn newShaderRef(shader: Shader) -> ShaderRef {
	Arc::new(RwLock::new(shader))
}

pub fn newRenderableRef<T: Renderable + 'static>(renderable: T) -> RenderableRef {
	Rc::new(RefCell::new(renderable))
}

pub fn newMeshRef(mesh: Mesh) -> MeshRef {
	Rc::new(RefCell::new(mesh))
}

pub fn newPhysicalRef<P: Physical + 'static>(physical: P) -> PhysicalRef {
	Arc::new(RwLock::new(physical))
}

pub fn newSolverRef(solver: Solver) -> SolverRef {
	Rc::new(RefCell::new(solver))
}