use std::sync::OnceLock;
use bool_flags::Flags8;
use glow::{HasContext, PixelUnpackData, Texture as GlowTexture};
use image::ImageReader;
use tracing::{info, warn};
use crate::{gl_check_error, LogError};
use crate::types::{newTextureRef, GlRef, TextureRef};

static DEFAULT_TEXTURE_REF: OnceLock<TextureRef> = OnceLock::new();

const F_DELETED: u8 = 0;

#[derive(Debug, Clone)]
pub struct Texture {
	gl: GlRef,
	flags: Flags8,
	pub handle: Option<GlowTexture>,
	pub width: u32,
	pub height: u32,
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
	
	pub fn wrap(mut self, wrap: WrapMode) -> Self {
		self.wrap = wrap;
		self
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
			
			let filter = match self.filter {
				FilterMode::Nearest => glow::NEAREST as i32,
				FilterMode::Linear => glow::LINEAR as i32,
			};
			let wrap = match self.wrap {
				WrapMode::Repeat => glow::REPEAT as i32,
				WrapMode::ClampToEdge => glow::CLAMP_TO_EDGE as i32,
				WrapMode::MirroredRepeat => glow::MIRRORED_REPEAT as i32,
			};
			
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap);
			self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap);
			gl_check_error!(self.gl);
			
			Ok(Texture {
				gl: self.gl,
				flags: Flags8::none(),
				handle: Some(texture),
				width,
				height,
			})
		}
	}
}

impl Texture {
	/// Quick load with default settings (unit 0, nearest filter, repeat wrap)
	pub fn fromFile(gl: GlRef, path: &str) -> Result<Texture, String> {
		TextureBuilder::new(gl).loadFile(path)
	}
	
	/// Quick load from embedded bytes with default settings
	pub fn fromBytes(gl: GlRef, bytes: &[u8]) -> Result<Texture, String> {
		TextureBuilder::new(gl).loadBytes(bytes)
	}
	
	pub fn builder(gl: GlRef) -> TextureBuilder {
		TextureBuilder::new(gl)
	}
	
	pub fn defaultTexture(gl: &GlRef) -> TextureRef {
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
			self.gl.bind_texture(glow::TEXTURE_2D, self.handle);
			gl_check_error!(self.gl);
		}
	}
	
	pub fn delete(&mut self) {
		if self.flags.get(F_DELETED) {
			return;
		}
		unsafe {
			let handle = self.handle.take().unwrap();
			warn!("Deleting texture {:?}", handle);
			self.gl.delete_texture(handle);
			self.flags.set(F_DELETED);
		}
	}
}

impl Drop for Texture {
	fn drop(&mut self) {
		self.delete();
	}
}
