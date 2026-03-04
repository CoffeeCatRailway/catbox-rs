#![allow(non_snake_case)]

mod graphics;
mod catbox;
mod types;

use std::error::Error;
use tracing::info;
use crate::catbox::CatBox;

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

	let mut catBox = CatBox::new()?;
	catBox.run()?;
	catBox.destroy();
	
	Ok(())
}
