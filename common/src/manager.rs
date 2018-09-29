// Standard
use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

// Information
// -----------
// Utility wrappers for types that require worker threads that
// exist throughout their lifetime (note: if a thread is not tied
// to the lifetime of the type, a job should be used instead)
//
// See below for an example of usage in the test section

pub trait Managed: Send + Sync + Sized + 'static {
    fn init_workers(&self, manager: &mut Manager<Self>);
    fn on_drop(&self, _manager: &mut Manager<Self>) {}
}

#[derive(Debug)]
pub struct Manager<T: Managed> {
    internal: Arc<T>,
    root: bool,

    running: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
}

impl<T: Managed> Manager<T> {
    pub fn init(internal: T) -> Manager<T> {
        let mut manager = Manager {
            internal: Arc::new(internal),
            root: true,
            running: Arc::new(AtomicBool::new(true)),
            workers: vec![],
        };

        // Start workers
        manager.internal.clone().init_workers(&mut manager);

        manager
    }

    fn new_child(&self) -> Manager<T> {
        Manager {
            internal: self.internal.clone(),
            root: false,
            running: Arc::new(AtomicBool::new(true)),
            workers: vec![],
        }
    }

    pub fn add_worker<F: FnOnce(&T, &AtomicBool, Manager<T>) + Send + Sync + 'static>(this: &mut Self, f: F) {
        let internal = this.internal.clone();
        let running = this.running.clone();

        let child_mgr = this.new_child();
        this.workers
            .push(thread::spawn(move || f(&internal, &running, child_mgr)));
    }

    pub fn shutdown(this: &mut Self) {
        if this.root {
            this.internal.clone().on_drop(this);
            this.running.store(false, Ordering::Relaxed);
        }
    }

    pub fn await_shutdown(mut this: Self) { let _ = this.workers.drain(..).for_each(|w| w.join().unwrap()); }

    pub fn internal(this: &Self) -> &Arc<T> { &this.internal }
}

impl<T: Managed> Deref for Manager<T> {
    type Target = T;

    fn deref(&self) -> &T { &self.internal }
}

impl<T: Managed> Drop for Manager<T> {
    fn drop(&mut self) {
        Manager::shutdown(self);
        let _ = self.workers.drain(..).for_each(|w| w.join().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::{Managed, Manager};
    use std::{sync::atomic::Ordering, thread, time::Duration};

    struct Server;

    impl Server {
        fn new() -> Server { Server }
    }

    impl Managed for Server {
        fn init_workers(&self, manager: &mut Manager<Self>) {
            Manager::add_worker(manager, |_, running, _| {
                while running.load(Ordering::Relaxed) {
                    //println!("Hello, world!")
                }
            });
            Manager::add_worker(manager, |_, running, mut mgr| {
                while running.load(Ordering::Relaxed) {
                    //println!("Hi, planet!");

                    Manager::add_worker(&mut mgr, |_, _, _| {})
                }
            });
        }
    }

    #[test]
    fn test_manager() {
        let _server_mgr = Manager::init(Server::new());
        thread::sleep(Duration::from_millis(50));
    }
}
