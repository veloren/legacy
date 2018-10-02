// Standard
use parking_lot::Mutex;

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
