use common::log::{self, Log, Payload};

use crate::println;

struct Logger;

impl Log for Logger {
    fn log(&self, payload: &Payload) {
        println!("{}: {}", payload.level(), payload.msg());
    }
}

pub fn init_logger() {
    log::set_logger(&Logger).unwrap();
}
