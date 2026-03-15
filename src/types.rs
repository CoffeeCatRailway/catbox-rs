use std::cell::RefCell;
use std::rc::Rc;
use glow::{Context as GlowContext};
use sdl3::video::Window as SdlWindow;
use crate::graphics::line_renderer::LineRenderer;
use crate::graphics::mesh::Mesh;
use crate::graphics::render_manager::Renderable;
use crate::graphics::shader::Shader;

pub type SdlWindowRef = Rc<RefCell<SdlWindow>>;

pub type GlRef = Rc<GlowContext>;

pub type ShaderRef = Rc<Shader>;

pub type LineRendererRef = Rc<RefCell<LineRenderer>>;

pub type RenderableRef = Rc<RefCell<dyn Renderable>>;

pub type MeshRef = Rc<RefCell<dyn Mesh>>;

pub fn newSdlWindowRef(window: SdlWindow) -> SdlWindowRef {
	Rc::new(RefCell::new(window))
}

pub fn newGlRef(gl: GlowContext) -> GlRef {
	Rc::new(gl)
}


pub fn newShaderRef(shader: Shader) -> ShaderRef {
	Rc::new(shader)
}

pub fn newLineRendererRef(renderer: LineRenderer) -> LineRendererRef {
	Rc::new(RefCell::new(renderer))
}

pub fn newRenderableRef<T: Renderable + 'static>(renderable: T) -> RenderableRef {
	Rc::new(RefCell::new(renderable))
}

pub fn newMeshRef<T: Mesh + 'static>(mesh: T) -> MeshRef {
	Rc::new(RefCell::new(mesh))
}