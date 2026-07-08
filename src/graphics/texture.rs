use std::sync::OnceLock;
use bool_flags::Flags8;
use glow::{Framebuffer, HasContext, PixelUnpackData, Texture as GlowTexture};
use image::ImageReader;
use tracing::{info, warn};
use crate::{gl_check_error, LogError};
use crate::types::{newTextureRef, GlRef, TextureRef};

static DEFAULT_TEXTURE_REF: OnceLock<TextureRef> = OnceLock::new();

const F_DELETED: u8 = 0;
const F_DEPTH_MAP: u8 = 1;

#[derive(Debug, Clone)]
pub struct Texture {
	gl: GlRef,
	flags: Flags8,
	pub handleTex: Option<GlowTexture>,
	pub handleFBO: Option<Framebuffer>,
	pub width: u32,
	pub height: u32,
}

#[derive(Debug, Copy, Clone, Default)]
pub enum DepthComponent {
	#[default]
	U16,
	U24,
	U32,
	F32,
}

#[derive(Debug, Copy, Clone, Default)]
pub enum FilterMode {
	#[default]
	Nearest,
	Linear,
}

#[derive(Debug, Copy, Clone, Default)]
pub enum WrapMode {
	#[default]
	Repeat,
	ClampToEdge,
	ClampToBorder,
	MirroredRepeat,
}

pub struct TextureBuilder {
	gl: GlRef,
	filter: FilterMode,
	wrap: WrapMode,
}

impl TextureBuilder {
	pub fn new(gl: GlRef) -> Self {
		Self {
			gl,
			filter: FilterMode::default(),
			wrap: WrapMode::default(),
		}
	}
	
	pub fn filter(mut self, filter: FilterMode) -> Self {
		self.filter = filter;
		self
	}
	
	fn filterValue(&self) -> i32 {
		match self.filter {
			FilterMode::Nearest => glow::NEAREST as i32,
			FilterMode::Linear => glow::LINEAR as i32,
		}
	}
	
	pub fn wrap(mut self, wrap: WrapMode) -> Self {
		self.wrap = wrap;
		self
	}
	
	fn wrapValue(&self) -> i32 {
		match self.wrap {
			WrapMode::Repeat => glow::REPEAT as i32,
			WrapMode::ClampToEdge => glow::CLAMP_TO_EDGE as i32,
			WrapMode::ClampToBorder => glow::CLAMP_TO_BORDER as i32,
			WrapMode::MirroredRepeat => glow::MIRRORED_REPEAT as i32,
		}
	}
	
	/// Load texture from file path (not supported on WASM - Unclear if I'll implement WASM yet)
	pub fn loadFile(self, path: &str) -> Result<Texture, String> {
		let img = ImageReader::open(path).map_err(|e| format!("Failed to open image '{}': {}", path, e)).logErr()?
			.decode().map_err(|e| format!("Failed to decode image '{}': {}", path, e)).logErr()?
			.to_rgba8();
		self.loadRGBA(&img.as_raw(), img.width(), img.height())
	}
	
	/// Load texture from embedded bytes
	pub fn loadBytes(self, bytes: &[u8]) -> Result<Texture, String> {
		let img = image::load_from_memory(bytes).map_err(|e| format!("Failed to decode image: {}", e)).logErr()?
			.to_rgba8();
		self.loadRGBA(&img.as_raw(), img.width(), img.height())
	}
	
	/// Load texture from raw RGBA bytes
	pub fn loadRGBA(self, data: &[u8], width: u32, height: u32) -> Result<Texture, String> {
		unsafe {
			let texture = self.gl.create_texture().logErr()?;
			info!("Loading rgba texture {}", texture.0);
			let pixels = PixelUnpackData::Slice(Some(data));
			
			self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			gl_check_error!(self.gl);
			
			self.gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGBA8 as i32,
				width as i32,
				height as i32,
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				pixels,
			);
			gl_check_error!(self.gl);
			
			let filter = self.filterValue();
			let wrap = self.wrapValue();
			
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap);
			gl_check_error!(self.gl);
			
			Ok(Texture {
				gl: self.gl,
				flags: Flags8::none(),
				handleTex: Some(texture),
				handleFBO: None,
				width,
				height,
			})
		}
	}
	
	pub fn createDepthMap(self, width: u32, height: u32, depthComponent: DepthComponent) -> Result<Texture, String> {
		unsafe {
			let fbo = self.gl.create_framebuffer().logErr()?;
			self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
			gl_check_error!(self.gl);
			
			let texture = self.gl.create_texture().logErr()?;
			self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			gl_check_error!(self.gl);
			info!("Creating depth map, fbo: {}, texture: {}", fbo.0, texture.0);
			
			let (depthValue, depthType) = match depthComponent {
				DepthComponent::U16 => (glow::DEPTH_COMPONENT16 as i32, glow::UNSIGNED_INT),
				DepthComponent::U24 => (glow::DEPTH_COMPONENT24 as i32, glow::UNSIGNED_INT),
				DepthComponent::U32 => (glow::DEPTH_COMPONENT32 as i32, glow::UNSIGNED_INT),
				DepthComponent::F32 => (glow::DEPTH_COMPONENT32F as i32, glow::FLOAT),
			};
			self.gl.tex_image_2d(glow::TEXTURE_2D, 0, depthValue, width as i32, height as i32, 0, glow::DEPTH_COMPONENT, depthType, PixelUnpackData::Slice(None));
			gl_check_error!(self.gl);
			
			let filter = self.filterValue();
			let wrap = self.wrapValue();
			
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_COMPARE_FUNC, glow::LEQUAL as i32);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_COMPARE_MODE, glow::COMPARE_REF_TO_TEXTURE as i32);
			self.gl.tex_parameter_f32_slice(glow::TEXTURE_2D, glow::TEXTURE_BORDER_COLOR, &[1.0, 1.0, 1.0, 1.0]);
			gl_check_error!(self.gl);
			
			self.gl.framebuffer_texture(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, Some(texture), 0);
			gl_check_error!(self.gl);
			
			if self.gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
				return Err(String::from("Depth map fbo incomplete!"));
			}
			
			self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
			gl_check_error!(self.gl);
			
			let mut flags = Flags8::none();
			flags.set(F_DEPTH_MAP);
			Ok(Texture {
				gl: self.gl,
				flags,
				handleTex: Some(texture),
				handleFBO: Some(fbo),
				width,
				height,
			})
		}
	}
}

impl Texture {
	pub fn isDepthMap(&self) -> bool {
		self.flags.get(F_DEPTH_MAP)
	}
	
	/// Quick load with default settings (unit 0, nearest filter, repeat wrap)
	pub fn fromFile(gl: GlRef, path: &str) -> Result<Texture, String> {
		TextureBuilder::new(gl).loadFile(path)
	}
	
	/// Quick load from embedded bytes with default settings
	pub fn fromBytes(gl: GlRef, bytes: &[u8]) -> Result<Texture, String> {
		TextureBuilder::new(gl).loadBytes(bytes)
	}
	
	/// Quick create depth map
	pub fn createDepthMap(gl: GlRef, width: u32, height: u32, depthComponent: DepthComponent) -> Result<Texture, String> {
		TextureBuilder::new(gl).filter(FilterMode::Nearest).wrap(WrapMode::ClampToBorder).createDepthMap(width, height, depthComponent)
	}
	
	pub fn builder(gl: GlRef) -> TextureBuilder {
		TextureBuilder::new(gl)
	}
	
	pub fn default1x1White(gl: &GlRef) -> TextureRef {
		DEFAULT_TEXTURE_REF.get_or_init(|| {
			info!("Building default texture");
			let data = [255, 255, 255, 255];
			newTextureRef(TextureBuilder::new(gl.clone())
				.filter(FilterMode::Nearest)
				.wrap(WrapMode::Repeat)
				.loadRGBA(&data, 1, 1).expect("Failed to load default texture"))
		}).clone()
	}
	
	pub fn bind(&self, active: u32) {
		if self.flags.get(F_DELETED) {
			return;
		}
		unsafe {
			self.gl.active_texture(glow::TEXTURE0 + active);
			self.gl.bind_texture(glow::TEXTURE_2D, self.handleTex);
			gl_check_error!(self.gl);
		}
	}
	
	pub fn bindDepthMapFBO(&self) {
		if self.flags.get(F_DELETED) || !self.isDepthMap() {
			warn!("Trying to bind depth map fbo for non-depth map texture!");
			return;
		}
		unsafe {
			self.gl.bind_framebuffer(glow::FRAMEBUFFER, self.handleFBO);
			gl_check_error!(self.gl);
		}
	}
	
	pub fn delete(&mut self) {
		if self.flags.get(F_DELETED) {
			return;
		}
		unsafe {
			if let Some(handle) = self.handleTex {
				warn!("Deleting texture {}", handle.0);
				self.gl.delete_texture(handle);
			}
			if let Some(handle) = self.handleFBO {
				warn!("Deleting depth map fbo {}", handle.0);
				self.gl.delete_framebuffer(handle);
			}
			self.flags.set(F_DELETED);
		}
	}
}

impl Drop for Texture {
	fn drop(&mut self) {
		// self.delete(); // todo: fix, causes seg fault on exit
	}
}
