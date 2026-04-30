// https://github.com/dgerrells/how-fast-is-it/blob/main/rust-land/src/thread_pool.rs
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use bool_flags::Flags8;
use tracing::{error, info, warn};

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

enum Message {
	NewJob(Job),
	Stop,
}

const F_STOPPED: u8 = 0;

pub struct ThreadPool {
	#[allow(unused)]
	workers: Vec<Worker>,
	sender: Sender<Message>,
	jobCount: Arc<(Mutex<usize>, Condvar)>,
	flags: Flags8,
}

impl ThreadPool {
	pub fn getAvailableMaxThreads() -> usize {
		match thread::available_parallelism() {
			Ok(t) => t.get(),
			Err(e) => {
				error!("{}", e);
				0
			},
		}
	}
	
	pub fn withMaxWorkers() -> Self {
		Self::withNWorkers(Self::getAvailableMaxThreads())
	}
	
	pub fn withNWorkers(workers: usize) -> Self {
		assert!(workers > 0);
		
		let maxWorkers = Self::getAvailableMaxThreads();
		assert!(maxWorkers > 0);
		
		let workers = if workers > maxWorkers {
			warn!("{} workers exceeded maximum of {}!", workers, maxWorkers);
			maxWorkers
		} else {
			workers
		};
		info!("Starting {} worker threads", workers);
		
		let (sender, receiver) = mpsc::channel();
		let receiver = Arc::new(Mutex::new(receiver));
		
		let jobCount = Arc::new((Mutex::new(0), Condvar::new()));
		
		let mut pool = Vec::with_capacity(workers);
		for id in 0..workers {
			pool.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&jobCount)));
		}
		
		ThreadPool {
			workers: pool,
			sender,
			jobCount,
			flags: Flags8::none(),
		}
	}
	
	pub fn execute<F: FnOnce(usize) + Send + 'static>(&self, f: F) {
		if self.flags.get(F_STOPPED) {
			return;
		}
		
		let (lock, _) = &*self.jobCount;
		*lock.lock().unwrap() += 1;
		
		let job = Box::new(f);
		self.sender.send(Message::NewJob(job)).unwrap();
	}
	
	pub fn waitForCompletion(&self) {
		if self.flags.get(F_STOPPED) {
			return;
		}
		
		let (lock, cvar) = &*self.jobCount;
		let mut count = lock.lock().unwrap();
		while *count > 0 {
			count = cvar.wait(count).unwrap();
		}
	}
	
	pub fn stopAll(&mut self) {
		if self.flags.get(F_STOPPED) {
			return;
		}
		
		for _ in 0..self.workers.len() {
			self.sender.send(Message::Stop).unwrap();
		}
		self.flags.set(F_STOPPED);
	}
	
	// pub fn getActive(&self) -> usize {
	// 	let (lock, _) = &*self.jobCount;
	// 	*lock.lock().unwrap()
	// }
	
	pub fn getTotal(&self) -> usize {
		self.workers.len()
	}
}

impl Drop for ThreadPool {
	fn drop(&mut self) {
		self.stopAll();
	}
}

struct Worker {
	#[allow(unused)]
	id: usize,
	#[allow(unused)]
	handle: Option<JoinHandle<()>>,
}

impl Worker {
	fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>, jobCount: Arc<(Mutex<usize>, Condvar)>) -> Self {
		let handle = thread::spawn(move || loop {
			let message = receiver.lock().unwrap().recv();
			match message {
				Ok(Message::NewJob(job)) => {
					job(id);
					let (lock, cvar) = &*jobCount;
					let mut count = lock.lock().unwrap();
					*count -= 1;
					cvar.notify_all();
				},
				Ok(Message::Stop) => {
					warn!("Stopping worker thread {}", id);
					break;
				},
				Err(e) => {
					error!("Worker thread panicked: {}", e);
					break;
				},
			}
		});
		
		Worker {
			id,
			handle: Some(handle),
		}
	}
}