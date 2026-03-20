use std::sync::{Arc, RwLock};
use glow::{Context as GlowContext};
use sdl3::video::Window as SdlWindow;
use crate::graphics::line_renderer::LineRenderer;
use crate::graphics::mesh::Mesh;
use crate::graphics::render_manager::Renderable;
use crate::graphics::shader::Shader;
use crate::simulation::verlet_solver::{Physical, VerletSolver};

pub type SdlWindowRef = Arc<RwLock<SdlWindow>>;

pub type GlRef = Arc<GlowContext>;

pub type ShaderRef = Arc<RwLock<Shader>>;

pub type LineRendererRef = Arc<RwLock<LineRenderer>>;

pub type RenderableRef = Arc<RwLock<dyn Renderable>>;

pub type MeshRef = Arc<RwLock<Mesh>>;

pub type PhysicalRef = Arc<RwLock<dyn Physical>>;

pub type VerletSolverRef = Arc<RwLock<VerletSolver>>;

pub fn newSdlWindowRef(window: SdlWindow) -> SdlWindowRef {
	Arc::new(RwLock::new(window))
}

pub fn newGlRef(gl: GlowContext) -> GlRef {
	Arc::new(gl)
}

pub fn newShaderRef(shader: Shader) -> ShaderRef {
	Arc::new(RwLock::new(shader))
}

pub fn newLineRendererRef(renderer: LineRenderer) -> LineRendererRef {
	Arc::new(RwLock::new(renderer))
}

pub fn newRenderableRef<T: Renderable + 'static>(renderable: T) -> RenderableRef {
	Arc::new(RwLock::new(renderable))
}

pub fn newMeshRef(mesh: Mesh) -> MeshRef {
	Arc::new(RwLock::new(mesh))
}

pub fn newPhysicalRef<P: Physical + 'static>(physical: P) -> PhysicalRef {
	Arc::new(RwLock::new(physical))
}

pub fn newVerletSolverRef(solver: VerletSolver) -> VerletSolverRef {
	Arc::new(RwLock::new(solver))
}