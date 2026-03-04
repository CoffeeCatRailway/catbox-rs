use std::cell::RefCell;
use std::rc::Rc;
use glow::{Context as GlowContext};
use sdl3::video::Window as SdlWindow;
use crate::graphics::line_renderer::LineRenderer;
// use sdl3::video::{GLContext, Window as SdlWindow};
// use crate::graphics::shader::Shader;

pub type SdlWindowRef = Rc<RefCell<SdlWindow>>;

pub type GlRef = Rc<GlowContext>;

// pub type GlContextRef = Rc<GLContext>;
//
// pub type ShaderRef = Rc<Shader>;

pub type LineRendererRef = Rc<RefCell<LineRenderer>>;

pub fn newSdlWindowRef(window: SdlWindow) -> SdlWindowRef {
	Rc::new(RefCell::new(window))
}

// pub fn newGlRef(gl: GlowContext) -> GlRef {
// 	Rc::new(gl)
// }

// pub fn newGlContextRef(glContext: GLContext) -> GlContextRef {
// 	Rc::new(glContext)
// }
//
// pub fn newShaderRef(shader: Shader) -> ShaderRef {
// 	Rc::new(shader)
// }

pub fn newLineRendererRef(renderer: LineRenderer) -> LineRendererRef {
	Rc::new(RefCell::new(renderer))
}
