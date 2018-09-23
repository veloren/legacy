#![feature(nll, integer_atomics)]

extern crate bincode;
extern crate get_if_addrs;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
extern crate rand;
extern crate time;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate vek;

// Reexports
pub use clock::Clock;

pub mod clock;
pub mod manager;
pub mod msg;
pub mod names;
pub mod net;
pub mod post;

// Standard
use parking_lot::Mutex;

pub type Uid = u64;

const CARGO_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub fn get_version() -> String { CARGO_VERSION.unwrap_or("UNKNOWN VERSION").to_string() }

pub struct TestPorts {
    next: Mutex<u32>,
}

impl TestPorts {
    pub fn new() -> TestPorts {
        TestPorts {
            next: Mutex::new(50000),
        }
    }

    pub fn next(&self) -> String {
        let mut n = self.next.lock();
        *n += 1;
        format!("127.0.0.1:{}", *n)
    }
}

lazy_static! {
    pub static ref PORTS: TestPorts = TestPorts::new();
}
