// https://github.com/dgerrells/how-fast-is-it/blob/main/rust-land/src/thread_pool.rs
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, warn};

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

enum Message {
	NewJob(Job),
	Stop,
}

pub struct ThreadPool {
	#[allow(unused)]
	workers: Vec<Worker>,
	sender: Sender<Message>,
	jobCount: Arc<(Mutex<usize>, Condvar)>,
}

impl ThreadPool {
	pub fn new(size: usize) -> Self {
		assert!(size > 0);
		
		let (sender, receiver) = mpsc::channel();
		let receiver = Arc::new(Mutex::new(receiver));
		
		let jobCount = Arc::new((Mutex::new(0), Condvar::new()));
		
		let mut workers = Vec::with_capacity(size);
		for id in 0..size {
			workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&jobCount)));
		}
		
		ThreadPool {
			workers,
			sender,
			jobCount,
		}
	}
	
	pub fn execute<F: FnOnce(usize) + Send + 'static>(&self, f: F) {
		let (lock, _) = &*self.jobCount;
		*lock.lock().unwrap() += 1;
		
		let job = Box::new(f);
		self.sender.send(Message::NewJob(job)).unwrap();
	}
	
	pub fn waitForCompletion(&self) {
		let (lock, cvar) = &*self.jobCount;
		let mut count = lock.lock().unwrap();
		while *count > 0 {
			count = cvar.wait(count).unwrap();
		}
	}
	
	pub fn stopAll(&self) {
		for _ in 0..self.workers.len() {
			self.sender.send(Message::Stop).unwrap();
		}
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