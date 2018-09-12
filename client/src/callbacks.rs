use std::sync::Mutex;

pub struct Callbacks {
    recv_chat_msg: Mutex<Option<Box<Fn(&str) + Send>>>,
}

impl Callbacks {
    pub fn new() -> Callbacks {
        Callbacks {
            recv_chat_msg: Mutex::new(None),
        }
    }

    pub fn call_recv_chat_msg(&self, msg: &str) {
        match *self.recv_chat_msg.lock().unwrap() {
            Some(ref f) => f(msg),
            None => {},
        }
    }

    pub fn set_recv_chat_msg<F: 'static + Fn(&str) + Send>(&self, f: F) {
        *self.recv_chat_msg.lock().unwrap() = Some(Box::new(f));
    }
}
