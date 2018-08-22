use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::ops::Deref;

// Information
// -----------
// Utility wrappers for types that require worker threads that
// exist throughout their lifetime (note: if a thread is not tied
// to the lifetime of the type, a job should be used instead)
//
// See below for an example of usage in the test section

pub trait Managed: Send + Sync + Sized + 'static {
    fn init_workers(&self, manager: &mut Manager<Self>);
}

pub struct Manager<T: Managed> {
    internal: Arc<T>,

    running: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
}

impl<T: Managed> Manager<T> {
    pub fn init(internal: T) -> Manager<T> {
        let internal = Arc::new(internal);
        let running = Arc::new(AtomicBool::new(true));

        let mut manager = Manager { internal, running, workers: vec!() };

        // Start workers
        manager.internal.clone().init_workers(&mut manager);

        manager
    }

    pub fn add_worker<F: FnOnce(&T, &AtomicBool) + Send + Sync + 'static>(this: &mut Self, f: F) {
        let internal = this.internal.clone();
        let running = this.running.clone();
        this.workers.push(thread::spawn(move || {
            f(&internal, &running)
        }));
    }
}

impl<T: Managed> Deref for Manager<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.internal
    }
}

impl<T: Managed> Drop for Manager<T> {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.workers.drain(..).for_each(|w| w.join().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use super::{Managed, Manager};

    struct Server;

    impl Server {
        fn new() -> Server {
            Server
        }
    }

    impl Managed for Server {
        fn init_workers(&self, manager: &mut Manager<Self>) {
            Manager::add_worker(manager, |_, running| while running.load(Ordering::Relaxed) {
                println!("Hello, world!")
            });
            Manager::add_worker(manager, |_, running| while running.load(Ordering::Relaxed) {
                println!("Hi, planet!")
            });
        }
    }

    #[test]
    fn test_manager() {
        let server_mgr = Manager::init(Server::new());
    }
}
