#![allow(non_snake_case)]

mod graphics;
mod simulation;
mod types;
mod window;
mod thread_pool;

use std::error::Error;
use glow::HasContext;
use tracing::{error, info};
use crate::types::GlRef;
use crate::window::CatBox;

// Helper to check for GL errors at runtime. Mirrors the behavior of the
// C-style `glCheckError()` helper: it polls `gl.get_error()` and prints
// any found errors with the source file and line number.
pub fn gl_check_error_impl(gl: &GlRef, file: &'static str, line: u32) -> u32 {
	let mut last_error = glow::NO_ERROR;
	#[cfg(debug_assertions)] // Only compiles in dev
	unsafe {
		loop {
			let err = gl.get_error();
			if err == glow::NO_ERROR {
				break;
			}
			last_error = err;
			let error_str = match err {
				glow::INVALID_ENUM => "INVALID_ENUM",
				glow::INVALID_VALUE => "INVALID_VALUE",
				glow::INVALID_OPERATION => "INVALID_OPERATION",
				glow::STACK_OVERFLOW => "STACK_OVERFLOW",
				glow::STACK_UNDERFLOW => "STACK_UNDERFLOW",
				glow::OUT_OF_MEMORY => "OUT_OF_MEMORY",
				glow::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
				_ => &*format!("UNKNOWN_ERROR ({})", err),
			};
			error!("GL error: {} | {} ({})", error_str, file, line);
		}
	}
	last_error
}

// Macro wrapper so callers can write `gl_check_error!(gl)` and get file/line.
#[macro_export]
macro_rules! gl_check_error {
    ($gl:expr) => {
        $crate::gl_check_error_impl(&$gl, file!(), line!())
    };
}

fn initializeTracing() -> Result<(), Box<dyn Error>> {
	use std::fs::File;
	use std::sync::Arc;
	use tracing_subscriber::{fmt, filter, prelude::*};
	
	// console
	let stdoutLog = fmt::layer()
		.with_ansi(true)
		.with_target(false)
		.with_file(true)
		.with_line_number(true)
		.with_thread_names(true)
		.with_thread_ids(false)
		.with_filter(filter::LevelFilter::INFO);
	
	// file
	let file = File::create("latest.log")?;
	let fileLog = fmt::layer()
		.with_writer(Arc::new(file))
		.with_ansi(false)
		.with_target(true)
		.with_file(true)
		.with_line_number(true)
		.with_thread_names(true)
		.with_thread_ids(true)
		.with_filter(filter::LevelFilter::INFO);
	
	// combine
	tracing_subscriber::registry().with(stdoutLog).with(fileLog).init();
	
	info!("Hello world from tracing!");
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	initializeTracing()?;

	let mut catbox = CatBox::new()?;
	catbox.run()?;
	catbox.destroy();
	
	Ok(())
}