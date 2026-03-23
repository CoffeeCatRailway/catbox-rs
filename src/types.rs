use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use glow::{Context as GlowContext};
use sdl3::video::Window as SdlWindow;
use crate::graphics::line_renderer::LineRenderer;
use crate::graphics::mesh::Mesh;
use crate::graphics::render_manager::Renderable;
use crate::graphics::shader::Shader;
use crate::simulation::solver::{Physical, Solver};

pub type SdlWindowRef = Rc<RefCell<SdlWindow>>;

pub type GlRef = Arc<GlowContext>;

pub type ShaderRef = Arc<RwLock<Shader>>;

pub type LineRendererRef = Rc<RefCell<LineRenderer>>;

pub type RenderableRef = Rc<RefCell<dyn Renderable>>;

pub type MeshRef = Rc<RefCell<Mesh>>;

pub type PhysicalRef = Rc<RefCell<dyn Physical>>;

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

pub fn newLineRendererRef(renderer: LineRenderer) -> LineRendererRef {
	Rc::new(RefCell::new(renderer))
}

pub fn newRenderableRef<T: Renderable + 'static>(renderable: T) -> RenderableRef {
	Rc::new(RefCell::new(renderable))
}

pub fn newMeshRef(mesh: Mesh) -> MeshRef {
	Rc::new(RefCell::new(mesh))
}

pub fn newPhysicalRef<P: Physical + 'static>(physical: P) -> PhysicalRef {
	Rc::new(RefCell::new(physical))
}

pub fn newSolverRef(solver: Solver) -> SolverRef {
	Rc::new(RefCell::new(solver))
}