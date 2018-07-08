use std::iter::IntoIterator;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Weak, RwLock, Mutex};

pub struct Jobs<T: 'static + Sync + Send> {
	root_ref: RwLock<Weak<T>>,
}

impl<T: 'static + Sync + Send> Jobs<T> {
	pub fn new() -> Jobs<T> {
		Jobs {
			root_ref: RwLock::new(Weak::new()),
		}
	}

	pub fn set_root(&self, root: Arc<T>) {
		*self.root_ref.write().unwrap() = Arc::downgrade(&root);
	}

	pub fn do_once<F, U: 'static + Send>(&self, job_func: F) -> JobHandle<U>
		where F: FnOnce(&Arc<T>) -> U + Send + 'static
	{
		let root = self.root_ref
			.read()
			.unwrap()
			.upgrade()
			.expect("Root no longer exists");

		JobHandle::new(thread::spawn(move || {
			let root_ref = root;
			job_func(&root_ref)
		}))
	}

	pub fn do_loop<F>(&self, job_func: F) -> JobHandle<()>
			where F: Fn(&Arc<T>) -> bool + Copy + Send + 'static
	{
		let root = self.root_ref
			.read()
			.unwrap()
			.upgrade()
			.expect("Root no longer exists");

		JobHandle::new(thread::spawn(move || {
			let root_ref = root;
			while job_func(&root_ref) {}
		}))
	}
}

pub struct JobHandle<T> {
	handle: JoinHandle<T>,
}

impl<T> JobHandle<T> {
	pub fn new(handle: JoinHandle<T>) -> JobHandle<T> {
		JobHandle { handle }
	}

    pub fn reify(self) -> T {
        self.handle.join().expect("Could not yield job")
    }

	pub fn await(self) {
		self.handle.join().expect("Could not await job");
	}

	pub fn ignore(self) {}
}

pub trait JobMultiHandle: Sized {
	fn await(self);
    fn ignore(self: Self) {}
}

impl<I, T> JobMultiHandle for I
	where I: IntoIterator<Item = JobHandle<T>> + Sized
{
	fn await(self: Self) {
		for job in self {
			job.await();
		}
	}
}
